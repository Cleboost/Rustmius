use crate::service::SshServer;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LayoutItem {
    Server {
        name: String,
    },
    Folder {
        #[serde(default)]
        id: String,
        name: String,
        items: Vec<LayoutItem>,
    },
}

fn is_folder_by_id_or_name(item: &LayoutItem, key: &str) -> bool {
    matches!(
        item,
        LayoutItem::Folder { id, name, .. } if id == key || name == key
    )
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Layout {
    pub items: Vec<LayoutItem>,
}

fn layout_config_path_internal() -> PathBuf {
    let base = config_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Path::new(&home).join(".config")
    });
    base.join("rustmius").join("layout.json")
}

pub fn ensure_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn load_layout(existing_servers: &[SshServer]) -> Layout {
    let path = layout_config_path_internal();
    let mut layout: Layout = match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Layout::default(),
    };
    ensure_folder_ids(&mut layout);
    sync_layout_with_servers(&mut layout, existing_servers);
    layout
}

fn ensure_folder_ids(layout: &mut Layout) {
    fn ensure_in_items(items: &mut [LayoutItem]) {
        for item in items.iter_mut() {
            if let LayoutItem::Folder { id, items, .. } = item {
                if id.is_empty() {
                    *id = Uuid::new_v4().to_string();
                }
                ensure_in_items(items);
            }
        }
    }
    ensure_in_items(&mut layout.items);
}

pub fn save_layout(layout: &Layout) -> Result<(), Box<dyn Error>> {
    let path = layout_config_path_internal();
    ensure_parent_dir(&path)?;
    let content = serde_json::to_string_pretty(layout)?;
    fs::write(path, content)?;
    Ok(())
}

fn collect_server_names(item: &LayoutItem) -> Vec<String> {
    match item {
        LayoutItem::Server { name } => vec![name.clone()],
        LayoutItem::Folder { items, .. } => {
            let mut names = Vec::new();
            for sub_item in items {
                names.extend(collect_server_names(sub_item));
            }
            names
        }
    }
}

pub fn sync_layout_with_servers(layout: &mut Layout, existing_servers: &[SshServer]) {
    let mut known: Vec<String> = Vec::new();
    for item in &layout.items {
        known.extend(collect_server_names(item));
    }

    for s in existing_servers {
        if !known.iter().any(|n| n == &s.name) {
            layout.items.push(LayoutItem::Server {
                name: s.name.clone(),
            });
        }
    }

    let existing_names: Vec<String> = existing_servers.iter().map(|s| s.name.clone()).collect();
    layout.items = layout
        .items
        .iter()
        .filter_map(|item| clean_layout_item(item, &existing_names))
        .collect();
}

fn clean_layout_item(item: &LayoutItem, existing_names: &[String]) -> Option<LayoutItem> {
    match item {
        LayoutItem::Server { name } => {
            if existing_names.contains(name) {
                Some(item.clone())
            } else {
                None
            }
        }
        LayoutItem::Folder { id, name, items } => {
            let cleaned_items: Vec<LayoutItem> = items
                .iter()
                .filter_map(|sub_item| clean_layout_item(sub_item, existing_names))
                .collect();

            if cleaned_items.is_empty() {
                None
            } else {
                Some(LayoutItem::Folder {
                    id: id.clone(),
                    name: name.clone(),
                    items: cleaned_items,
                })
            }
        }
    }
}

pub fn server_exists_anywhere(layout: &Layout, server_name: &str) -> bool {
    for item in &layout.items {
        match item {
            LayoutItem::Server { name } => {
                if name == server_name {
                    return true;
                }
            }
            LayoutItem::Folder { items, .. } => {
                if server_exists_in_folder_items(items, server_name) {
                    return true;
                }
            }
        }
    }
    false
}

fn server_exists_in_folder_items(items: &[LayoutItem], server_name: &str) -> bool {
    for item in items {
        match item {
            LayoutItem::Server { name } => {
                if name == server_name {
                    return true;
                }
            }
            LayoutItem::Folder {
                items: sub_items, ..
            } => {
                if server_exists_in_folder_items(sub_items, server_name) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn remove_server_from_anywhere_except_folder(
    layout: &mut Layout,
    server_name: &str,
    except_folder_id_or_name: &str,
) -> bool {
    let mut removed = false;

    fn clean_items(
        items: &[LayoutItem],
        server_name: &str,
        except_key: &str,
        removed: &mut bool,
    ) -> Vec<LayoutItem> {
        items
            .iter()
            .filter_map(|item| match item {
                LayoutItem::Server { name } => {
                    if name == server_name {
                        *removed = true;
                        None
                    } else {
                        Some(item.clone())
                    }
                }
                LayoutItem::Folder {
                    id, name, items, ..
                } => {
                    if id == except_key || name == except_key {
                        Some(item.clone())
                    } else {
                        let cleaned_items = clean_items(items, server_name, except_key, removed);
                        Some(LayoutItem::Folder {
                            id: id.clone(),
                            name: name.clone(),
                            items: cleaned_items,
                        })
                    }
                }
            })
            .collect()
    }

    layout.items = clean_items(
        &layout.items,
        server_name,
        except_folder_id_or_name,
        &mut removed,
    );

    fn purge_empty(items: &mut Vec<LayoutItem>) {
        let mut i = 0;
        while i < items.len() {
            match &mut items[i] {
                LayoutItem::Folder { items: sub, .. } => {
                    purge_empty(sub);
                    if sub.is_empty() {
                        items.remove(i);
                        continue;
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
    purge_empty(&mut layout.items);

    removed
}

pub fn remove_server_from_anywhere(layout: &mut Layout, server_name: &str) -> bool {
    let mut removed = false;

    layout.items = layout
        .items
        .iter()
        .filter_map(|item| {
            let cleaned = clean_item_remove_server(item, server_name);
            if let Some((item, was_removed)) = cleaned {
                if was_removed {
                    removed = true;
                }
                Some(item)
            } else {
                None
            }
        })
        .collect();

    fn purge_empty(items: &mut Vec<LayoutItem>) {
        let mut i = 0;
        while i < items.len() {
            match &mut items[i] {
                LayoutItem::Folder { items: sub, .. } => {
                    purge_empty(sub);
                    if sub.is_empty() {
                        items.remove(i);
                        continue;
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
    purge_empty(&mut layout.items);

    removed
}

fn clean_item_remove_server(item: &LayoutItem, server_name: &str) -> Option<(LayoutItem, bool)> {
    match item {
        LayoutItem::Server { name } => {
            if name == server_name {
                None
            } else {
                Some((item.clone(), false))
            }
        }
        LayoutItem::Folder { name, items, .. } => {
            let mut cleaned_items = Vec::new();
            let mut removed_any = false;

            for sub_item in items {
                if let Some((cleaned_item, was_removed)) =
                    clean_item_remove_server(sub_item, server_name)
                {
                    cleaned_items.push(cleaned_item);
                    if was_removed {
                        removed_any = true;
                    }
                } else {
                    removed_any = true;
                }
            }

            if cleaned_items.is_empty() {
                None
            } else {
                Some((
                    LayoutItem::Folder {
                        id: match item {
                            LayoutItem::Folder { id, .. } => id.clone(),
                            _ => String::new(),
                        },
                        name: name.clone(),
                        items: cleaned_items,
                    },
                    removed_any,
                ))
            }
        }
    }
}

fn find_item_index(layout: &Layout, predicate: impl Fn(&LayoutItem) -> bool) -> Option<usize> {
    layout.items.iter().position(predicate)
}

fn find_folder_mut<'a>(
    layout: &'a mut Layout,
    folder_id_or_name: &'a str,
) -> Option<&'a mut Vec<LayoutItem>> {
    fn find_in<'a>(items: &'a mut [LayoutItem], key: &'a str) -> Option<&'a mut Vec<LayoutItem>> {
        for item in items {
            match item {
                LayoutItem::Folder { id, name, items } => {
                    if id == key || name == key {
                        return Some(items);
                    }
                    if let Some(found) = find_in(items, key) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }
    find_in(&mut layout.items, folder_id_or_name)
}

pub fn move_into_folder(
    layout: &mut Layout,
    source: &str,
    folder_id_or_name: &str,
) -> Result<(), String> {
    if source == folder_id_or_name {
        return Ok(());
    }
    remove_server_from_anywhere(layout, source);
    if let Some(items) = find_folder_mut(layout, folder_id_or_name) {
        if !items
            .iter()
            .any(|item| matches!(item, LayoutItem::Server { name } if name == source))
        {
            items.push(LayoutItem::Server {
                name: source.to_string(),
            });
        }
        Ok(())
    } else {
        Err(format!("Folder '{}' not found", folder_id_or_name))
    }
}

fn unique_folder_name(layout: &Layout, base: &str) -> String {
    if !layout
        .items
        .iter()
        .any(|i| matches!(i, LayoutItem::Folder { name, .. } if name == base))
    {
        return base.to_string();
    }
    let mut idx = 1u32;
    loop {
        let candidate = format!("{} ({})", base, idx);
        if !layout
            .items
            .iter()
            .any(|i| matches!(i, LayoutItem::Folder { name, .. } if name == &candidate))
        {
            return candidate;
        }
        idx += 1;
    }
}

pub fn drop_onto_server_into(
    layout: &mut Layout,
    source: &str,
    target_server: &str,
) -> Result<(), String> {
    if source == target_server {
        return Ok(());
    }

    let mut target_folder_name: Option<String> = None;
    for item in &layout.items {
        if let LayoutItem::Folder { id: _, name, items } = item {
            if collect_server_names(&LayoutItem::Folder {
                id: String::new(),
                name: name.clone(),
                items: items.clone(),
            })
            .iter()
            .any(|n| n == target_server)
            {
                target_folder_name = Some(name.clone());
                break;
            }
        }
    }

    if let Some(folder_name) = target_folder_name {
        move_into_folder(layout, source, &folder_name)
    } else {
        let target_index = find_item_index(
            layout,
            |item| matches!(item, LayoutItem::Server { name } if name == target_server),
        )
        .ok_or_else(|| format!("Target server '{}' not found", target_server))?;

        remove_server_from_anywhere(layout, source);
        layout.items.remove(target_index);

        let folder_name = unique_folder_name(layout, &format!("Group: {}", target_server));
        let mut items = vec![LayoutItem::Server {
            name: target_server.to_string(),
        }];
        if !items
            .iter()
            .any(|item| matches!(item, LayoutItem::Server { name } if name == source))
        {
            items.push(LayoutItem::Server {
                name: source.to_string(),
            });
        }
        layout.items.insert(
            target_index,
            LayoutItem::Folder {
                id: Uuid::new_v4().to_string(),
                name: folder_name,
                items,
            },
        );
        Ok(())
    }
}

pub fn drop_onto_folder_into(
    layout: &mut Layout,
    source: &str,
    target_folder: &str,
) -> Result<(), String> {
    move_into_folder(layout, source, target_folder)
}

pub fn drop_onto_server_into_folder(
    layout: &mut Layout,
    source: &str,
    target_server: &str,
    parent_folder: &str,
) -> Result<(), String> {
    if source == target_server {
        return Ok(());
    }

    let new_folder_id = Uuid::new_v4().to_string();
    let folder_name = unique_folder_name(layout, &format!("Group: {}", target_server));

    let parent_items: &mut Vec<LayoutItem> = find_folder_mut(layout, parent_folder)
        .ok_or_else(|| format!("Parent folder '{}' not found", parent_folder))?;

    let target_index = find_item_index_in_vec(parent_items, |item| match item {
        LayoutItem::Server { name } => name == target_server,
        _ => false,
    })
    .ok_or_else(|| {
        format!(
            "Target server '{}' not found in folder '{}'",
            target_server, parent_folder
        )
    })?;

    parent_items.remove(target_index);

    let mut subfolder_items = vec![LayoutItem::Server {
        name: target_server.to_string(),
    }];
    if !subfolder_items
        .iter()
        .any(|item| matches!(item, LayoutItem::Server { name } if name == source))
    {
        subfolder_items.push(LayoutItem::Server {
            name: source.to_string(),
        });
    }

    parent_items.insert(
        target_index,
        LayoutItem::Folder {
            id: new_folder_id.clone(),
            name: folder_name,
            items: subfolder_items,
        },
    );

    remove_server_from_anywhere_except_folder(layout, source, &new_folder_id);

    Ok(())
}

pub fn drop_folder_onto_folder(
    layout: &mut Layout,
    source_folder: &str,
    target_folder: &str,
) -> Result<(), String> {
    if source_folder == target_folder {
        return Ok(());
    }

    let source_item = find_and_remove_folder(layout, source_folder)
        .ok_or_else(|| format!("Source folder '{}' not found", source_folder))?;

    let target_index = find_item_index(layout, |item| is_folder_by_id_or_name(item, target_folder))
        .ok_or_else(|| format!("Target folder '{}' not found", target_folder))?;

    if let LayoutItem::Folder { name: _, items, .. } = &mut layout.items[target_index] {
        items.push(source_item);
        Ok(())
    } else {
        Err(format!("Target '{}' is not a folder", target_folder))
    }
}

fn find_and_remove_folder(layout: &mut Layout, folder_id_or_name: &str) -> Option<LayoutItem> {
    let index = find_item_index(layout, |item| is_folder_by_id_or_name(item, folder_id_or_name))?;
    Some(layout.items.remove(index))
}

fn find_item_index_in_vec<F>(items: &[LayoutItem], predicate: F) -> Option<usize>
where
    F: Fn(&LayoutItem) -> bool,
{
    items.iter().position(predicate)
}

pub fn get_folder_path(layout: &Layout, target_folder: &str) -> Vec<String> {
    fn find_folder_path_recursive(
        items: &[LayoutItem],
        target: &str,
        current_path: &mut Vec<String>,
    ) -> bool {
        for item in items {
            match item {
                LayoutItem::Folder {
                    name,
                    items: sub_items,
                    ..
                } => {
                    current_path.push(name.clone());
                    if name == target {
                        return true;
                    }
                    if find_folder_path_recursive(sub_items, target, current_path) {
                        return true;
                    }
                    current_path.pop();
                }
                LayoutItem::Server { .. } => {}
            }
        }
        false
    }

    let mut path = Vec::new();
    find_folder_path_recursive(&layout.items, target_folder, &mut path);
    path
}

pub fn get_servers_in_folder(layout: &Layout, folder_name: &str) -> Vec<String> {
    fn collect_servers_recursive(items: &[LayoutItem], target: &str, servers: &mut Vec<String>) {
        for item in items {
            match item {
                LayoutItem::Folder {
                    name,
                    items: sub_items,
                    ..
                } => {
                    if name == target {
                        collect_all_servers_from_items(sub_items, servers);
                        return;
                    }
                    collect_servers_recursive(sub_items, target, servers);
                }
                LayoutItem::Server { .. } => {}
            }
        }
    }

    fn collect_all_servers_from_items(items: &[LayoutItem], servers: &mut Vec<String>) {
        for item in items {
            match item {
                LayoutItem::Server { name } => {
                    servers.push(name.clone());
                }
                LayoutItem::Folder {
                    items: sub_items, ..
                } => {
                    collect_all_servers_from_items(sub_items, servers);
                }
            }
        }
    }

    let mut servers = Vec::new();
    collect_servers_recursive(&layout.items, folder_name, &mut servers);
    servers
}

pub fn get_items_in_folder(layout: &Layout, folder_name: &str) -> Vec<LayoutItem> {
    fn find_folder_and_get_items(items: &[LayoutItem], target: &str) -> Option<Vec<LayoutItem>> {
        for item in items {
            match item {
                LayoutItem::Folder {
                    name,
                    items: sub_items,
                    ..
                } => {
                    if name == target {
                        return Some(sub_items.clone());
                    }
                    if let Some(result) = find_folder_and_get_items(sub_items, target) {
                        return Some(result);
                    }
                }
                LayoutItem::Server { .. } => {}
            }
        }
        None
    }

    find_folder_and_get_items(&layout.items, folder_name).unwrap_or_default()
}

pub fn rename_folder(
    layout: &mut Layout,
    key_id_or_name: &str,
    new_name: &str,
) -> Result<(), String> {
    fn rename_in(items: &mut [LayoutItem], key: &str, new_name: &str) -> bool {
        for item in items.iter_mut() {
            match item {
                LayoutItem::Folder { id, name, items } => {
                    if id == key || name == key {
                        *name = new_name.to_string();
                        return true;
                    }
                    if rename_in(items, key, new_name) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    if rename_in(&mut layout.items, key_id_or_name, new_name) {
        Ok(())
    } else {
        Err(format!("Folder '{}' not found", key_id_or_name))
    }
}

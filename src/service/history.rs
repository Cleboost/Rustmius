use chrono::{DateTime, Local};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    #[serde(default)]
    pub id: Option<String>,
    pub server_name: String,
    pub timestamp: String,
}

fn history_file_path() -> PathBuf {
    let mut base = config_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::Path::new(&home).join(".config")
    });
    base.push("rustmius");
    base.push("history.json");
    base
}

pub fn load_history() -> Vec<HistoryEntry> {
    let path = history_file_path();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut line_entries: Vec<HistoryEntry> = Vec::new();
    let mut non_json_line_found = false;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<HistoryEntry>(line) {
            Ok(entry) => line_entries.push(entry),
            Err(_) => {
                non_json_line_found = true;
                break;
            }
        }
    }
    if !line_entries.is_empty() && !non_json_line_found {
        return line_entries;
    }

    if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(&content) {
        let mut entries: Vec<HistoryEntry> = Vec::new();
        for v in arr {
            if let Ok(entry) = serde_json::from_value::<HistoryEntry>(v) {
                entries.push(entry);
            }
        }
        return entries;
    }

    let mut entries: Vec<HistoryEntry> = Vec::new();
    let mut stream = serde_json::Deserializer::from_str(&content).into_iter::<Value>();
    while let Some(item) = stream.next() {
        if let Ok(Value::Object(map)) = item {
            if let Ok(entry) = serde_json::from_value::<Value>(Value::Object(map))
                .and_then(|v| serde_json::from_value::<HistoryEntry>(v))
            {
                entries.push(entry);
            }
        }
    }

    entries
}

pub fn append_history(server_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let now: DateTime<Local> = Local::now();
    let entry = HistoryEntry {
        id: Some(Uuid::new_v4().to_string()),
        server_name: server_name.to_string(),
        timestamp: now.to_rfc3339(),
    };

    let path = history_file_path();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;

    let line = serde_json::to_string(&entry)?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

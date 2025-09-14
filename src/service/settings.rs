use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub remember_servers: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            remember_servers: false,
        }
    }
}

fn settings_file_path() -> PathBuf {
    let mut base = config_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::Path::new(&home).join(".config")
    });
    base.push("rustmius");
    base.push("settings.json");
    base
}

pub fn load_settings() -> AppSettings {
    let path = settings_file_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(parsed) = serde_json::from_str::<AppSettings>(&content) {
            return parsed;
        }
    }
    AppSettings::default()
}

pub fn save_settings(settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings_file_path();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_string_pretty(settings)?;
    let mut file = fs::File::create(&path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

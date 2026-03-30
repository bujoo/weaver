use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeaverSettings {
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_username: String,
    pub mqtt_password: String,
    pub instance_id: String,
    pub workspace: String,
    pub workspace_mount: String,
    pub capacity: u32,
    pub brain_api_url: String,
    pub auto_connect: bool,
}

impl Default for WeaverSettings {
    fn default() -> Self {
        let instance_id = format!("weaver-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let workspace_mount = dirs::home_dir()
            .unwrap_or_default()
            .join("Workspace")
            .to_string_lossy()
            .to_string();

        Self {
            mqtt_host: "localhost".into(),
            mqtt_port: 1883,
            mqtt_username: "weaver-dev".into(),
            mqtt_password: "weaver-dev-secret".into(),
            instance_id,
            workspace: "dev".into(),
            workspace_mount,
            capacity: 2,
            brain_api_url: "http://localhost:8000".into(),
            auto_connect: false,
        }
    }
}

fn settings_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".config"));
    config_dir.join("contexthub-weaver").join("settings.json")
}

pub fn load_settings() -> WeaverSettings {
    let path = settings_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => WeaverSettings::default(),
    }
}

pub fn save_settings(settings: &WeaverSettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create settings dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write settings: {}", e))?;
    Ok(())
}

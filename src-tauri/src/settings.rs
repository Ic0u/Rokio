use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Favorite game data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteGame {
    pub id: String,
    pub place_id: u64,
    pub name: String,
    pub thumbnail: Option<String>,
    pub added_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    // General
    pub auto_lock_timeout: String, // "never", "1min", "5min", "15min"
    pub launch_on_startup: bool,
    pub always_on_top: bool,

    // Appearance
    pub theme: String, // "dark", "light", "system"
    pub compact_mode: bool,
    #[serde(default = "default_accent")]
    pub accent_color: String, // "red", "orange", "yellow", "green", "teal", "blue", "indigo", "purple", "pink"

    // Launch Options
    #[serde(default)]
    pub multi_instance: bool, // Enable multi-instance launching with isolated environments
    #[serde(default)]
    pub launcher_preference: String, // "default", "bloxstrap", "fishstrap", "froststrap", "client"
    #[serde(default)]
    pub quarantine_installers: bool, // Move RobloxPlayerInstaller.exe to prevent update popups (Windows)
    #[serde(default)]
    pub save_logs: bool, // Save session logs
    #[serde(default)]
    pub force_handle_closure: bool, // Aggressive handle resolution for multi-instance
    #[serde(default)]
    pub low_cpu_mode: bool, // Lower CPU usage by reducing scan frequency

    // Favorite Games
    #[serde(default)]
    pub favorite_games: Vec<FavoriteGame>,
}

fn default_accent() -> String {
    "red".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_lock_timeout: "never".to_string(),
            launch_on_startup: false,
            always_on_top: false,
            theme: "dark".to_string(),
            compact_mode: false,
            accent_color: "red".to_string(),
            multi_instance: false, // Disabled by default
            launcher_preference: "default".to_string(),
            quarantine_installers: false,
            save_logs: false,
            force_handle_closure: false,
            low_cpu_mode: false,
            favorite_games: vec![],
        }
    }
}

fn get_settings_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("Failed to get app data dir")
        .join("settings.json")
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    let path = get_settings_path(&app);

    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {}", e))?;

    let settings: AppSettings =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))?;

    Ok(settings)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path(&app);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create settings directory: {}", e))?;
    }

    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn reset_settings(app: AppHandle) -> Result<AppSettings, String> {
    let settings = AppSettings::default();
    save_settings(app, settings.clone())?;
    Ok(settings)
}

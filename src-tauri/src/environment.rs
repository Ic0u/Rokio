use crate::binarycookies::{BinaryCookies, Cookie, Page};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Manager};

/// Get the environments directory path
fn get_environments_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("Failed to get app data dir")
        .join("environments")
}

/// Get the profile directory for a specific account
fn get_profile_dir(app: &AppHandle, account_id: &str) -> PathBuf {
    get_environments_dir(app).join(account_id)
}

/// Create isolated environment for an account
/// This creates the folder structure that Roblox expects with a custom HOME (macOS/Linux)
/// or custom AppData directories (Windows)
#[tauri::command]
pub fn create_environment(app: AppHandle, account_id: String) -> Result<(), String> {
    let profile_dir = get_profile_dir(&app, &account_id);

    // Create base directories
    fs::create_dir_all(&profile_dir).map_err(|e| e.to_string())?;

    // Platform-specific directory structures
    #[cfg(target_os = "windows")]
    {
        // Windows: Create LocalAppData and AppData structure
        let local_appdata = profile_dir.join("LocalAppData");
        let appdata = profile_dir.join("AppData");

        // Roblox LocalAppData structure
        let roblox_local = local_appdata.join("Roblox");
        fs::create_dir_all(&roblox_local.join("LocalStorage")).map_err(|e| e.to_string())?;
        fs::create_dir_all(&roblox_local.join("Versions")).map_err(|e| e.to_string())?;
        fs::create_dir_all(&roblox_local.join("Downloads")).map_err(|e| e.to_string())?;

        // Roblox AppData structure
        let roblox_appdata = appdata.join("Local").join("Roblox");
        fs::create_dir_all(&roblox_appdata).map_err(|e| e.to_string())?;

        // Create Documents for compatibility
        let documents_dir = profile_dir.join("Documents");
        fs::create_dir_all(&documents_dir).map_err(|e| e.to_string())?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        // macOS/Linux: Create HOME directory structure

        // Documents - required for various executors
        let documents_dir = profile_dir.join("Documents");
        fs::create_dir_all(&documents_dir).map_err(|e| e.to_string())?;

        // Downloads - Create symlink to real Downloads folder to bypass sandbox check
        let downloads_dir = profile_dir.join("Downloads");
        if !downloads_dir.exists() {
            // Get the real user Downloads folder
            if let Some(home) = dirs::home_dir() {
                let real_downloads = home.join("Downloads");
                if real_downloads.exists() {
                    // Create symlink to real Downloads
                    #[cfg(unix)]
                    {
                        std::os::unix::fs::symlink(&real_downloads, &downloads_dir)
                            .map_err(|e| format!("Failed to create Downloads symlink: {}", e))?;
                    }
                } else {
                    fs::create_dir_all(&downloads_dir).map_err(|e| e.to_string())?;
                }
            } else {
                fs::create_dir_all(&downloads_dir).map_err(|e| e.to_string())?;
            }
        }

        // Library for application support
        let library_dir = profile_dir.join("Library");
        fs::create_dir_all(&library_dir).map_err(|e| e.to_string())?;

        // Application Support
        let app_support_dir = library_dir.join("Application Support");
        fs::create_dir_all(&app_support_dir).map_err(|e| e.to_string())?;

        // Preferences
        let preferences_dir = library_dir.join("Preferences");
        fs::create_dir_all(&preferences_dir).map_err(|e| e.to_string())?;

        // Caches
        let caches_dir = library_dir.join("Caches");
        fs::create_dir_all(&caches_dir).map_err(|e| e.to_string())?;

        // Keychains directory
        let keychains_dir = library_dir.join("Keychains");
        fs::create_dir_all(&keychains_dir).map_err(|e| e.to_string())?;

        // Roblox custom assets path (macOS-specific)
        #[cfg(target_os = "macos")]
        {
            let content_dir = profile_dir
                .join("Applications")
                .join("Roblox.app")
                .join("Contents")
                .join("Resources")
                .join("content");
            fs::create_dir_all(&content_dir).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// Create a login keychain for the profile's custom HOME directory
/// This is required because Roblox tries to store credentials in the keychain
#[cfg(target_os = "macos")]
#[tauri::command]
pub fn create_keychain(app: AppHandle, account_id: String) -> Result<(), String> {
    use std::process::Command;

    let profile_dir = get_profile_dir(&app, &account_id);

    // Check if keychain already exists
    let keychain_path = profile_dir
        .join("Library")
        .join("Keychains")
        .join("login.keychain-db");
    if keychain_path.exists() {
        return Ok(()); // Already exists, no need to create
    }

    // Create keychain with empty password
    let status = Command::new("/usr/bin/security")
        .arg("create-keychain")
        .arg("-p")
        .arg("") // Empty password
        .arg("login.keychain")
        .env("HOME", &profile_dir)
        .status()
        .map_err(|e| format!("Failed to create keychain: {}", e))?;

    if !status.success() {
        // Keychain might already exist, which is fine
        return Ok(());
    }

    Ok(())
}

/// Unlock the login keychain for the profile
#[cfg(target_os = "macos")]
#[tauri::command]
pub fn unlock_keychain(app: AppHandle, account_id: String) -> Result<(), String> {
    use std::process::Command;

    let profile_dir = get_profile_dir(&app, &account_id);

    // Unlock keychain with empty password
    let _ = Command::new("/usr/bin/security")
        .arg("unlock-keychain")
        .arg("-p")
        .arg("") // Empty password
        .arg("login.keychain")
        .env("HOME", &profile_dir)
        .status();

    // Also set default keychain for this HOME
    let _ = Command::new("/usr/bin/security")
        .arg("default-keychain")
        .arg("-s")
        .arg("login.keychain")
        .env("HOME", &profile_dir)
        .status();

    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub fn create_keychain(_app: AppHandle, _account_id: String) -> Result<(), String> {
    Ok(()) // No-op on non-macOS
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub fn unlock_keychain(_app: AppHandle, _account_id: String) -> Result<(), String> {
    Ok(()) // No-op on non-macOS
}

/// Write cookies for an account to multiple locations in the profile's custom HOME directory
/// Roblox may look for cookies in different paths depending on version and configuration
#[tauri::command]
pub fn write_cookies(app: AppHandle, account_id: String, cookie: String) -> Result<(), String> {
    let profile_dir = get_profile_dir(&app, &account_id);

    // Create the cookie data once
    let cookie = Cookie {
        domain: ".roblox.com".into(),
        name: ".ROBLOSECURITY".into(),
        path: Some("/".into()),
        value: cookie,
        secure: Some(true),
        http_only: Some(true),
        expiration: Some(SystemTime::now() + Duration::from_secs(60 * 60 * 24 * 30)), // 30 days
        creation: Some(SystemTime::now()),
    };

    let page = Page::new(vec![cookie]);
    let binary_cookies = BinaryCookies::new(vec![page]);
    let bytes = binary_cookies.build();

    // Write to multiple locations to ensure Roblox finds the cookies

    // 1. HTTPStorages (where Roblox typically looks for cookies)
    let http_storages_dir = profile_dir.join("Library").join("HTTPStorages");
    fs::create_dir_all(&http_storages_dir).map_err(|e| e.to_string())?;
    let cookie_file = http_storages_dir.join("com.roblox.RobloxPlayer.binarycookies");
    let mut file = File::create(&cookie_file).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;

    // 2. Library/Cookies (standard macOS cookie location)
    let cookies_dir = profile_dir.join("Library").join("Cookies");
    fs::create_dir_all(&cookies_dir).map_err(|e| e.to_string())?;
    let cookie_file2 = cookies_dir.join("Cookies.binarycookies");
    let mut file2 = File::create(&cookie_file2).map_err(|e| e.to_string())?;
    file2.write_all(&bytes).map_err(|e| e.to_string())?;

    // 3. Also write with bundle ID suffix for compatibility
    let cookie_file3 = cookies_dir.join("com.roblox.RobloxPlayer.binarycookies");
    let mut file3 = File::create(&cookie_file3).map_err(|e| e.to_string())?;
    file3.write_all(&bytes).map_err(|e| e.to_string())?;

    Ok(())
}

/// Remove environment for an account (cleanup)
#[tauri::command]
pub fn remove_environment(app: AppHandle, account_id: String) -> Result<(), String> {
    let profile_dir = get_profile_dir(&app, &account_id);

    // Remove profile directory
    if profile_dir.exists() {
        fs::remove_dir_all(&profile_dir).map_err(|e| e.to_string())?;
    }

    // Also clean up the cookies file
    let data_dir = app.path().data_dir().map_err(|e| e.to_string())?;
    if let Some(library_dir) = data_dir.parent() {
        let http_storages_dir = library_dir.join("HTTPStorages");

        // Remove binary cookies file
        let cookie_file = http_storages_dir.join(format!(
            "com.roblox.RobloxPlayer.{}.binarycookies",
            account_id
        ));
        if cookie_file.exists() {
            fs::remove_file(&cookie_file).map_err(|e| e.to_string())?;
        }

        // Remove storage folder
        let storage_dir = http_storages_dir.join(format!("com.roblox.RobloxPlayer.{}", account_id));
        if storage_dir.exists() {
            fs::remove_dir_all(&storage_dir).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

/// Check if environment exists for an account
#[tauri::command]
pub fn environment_exists(app: AppHandle, account_id: String) -> bool {
    let profile_dir = get_profile_dir(&app, &account_id);
    profile_dir.exists()
}

/// Get the profile directory path for launching
pub fn get_launch_home_dir(app: &AppHandle, account_id: &str) -> PathBuf {
    get_profile_dir(app, account_id)
}

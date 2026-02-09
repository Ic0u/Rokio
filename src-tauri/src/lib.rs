//! ROKIO - Roblox Alt Manager
//! Main library entry point for Tauri.

// Module declarations
mod binarycookies;
mod browser_login;
mod client;
mod cookies;
mod crypto;
mod environment;
mod game_detection;
mod groups;
mod launcher;
mod process_utils;
mod profiles;
mod quick_login;
mod roblox;
mod settings;
mod utils;
mod vault;
mod watcher;

use crypto::CryptoState;
use launcher::LauncherState;
use serde::Serialize;

/// App initialization response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub platform: String,
    pub hwid: String,
}

/// Test command to verify Tauri bridge is working
#[tauri::command]
fn init_app_check() -> AppInfo {
    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: platform.to_string(),
        hwid: crypto::get_machine_id(),
    }
}

/// Tauri mobile entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Manage state
        .manage(CryptoState::default())
        .manage(LauncherState::default())
        // Register all commands
        .invoke_handler(tauri::generate_handler![
            // App commands
            init_app_check,
            // Crypto/Vault commands
            crypto::get_vault_status,
            crypto::get_hwid,
            crypto::create_vault,
            crypto::unlock_vault,
            crypto::lock_vault,
            // Account commands
            vault::get_accounts,
            vault::add_account,
            vault::update_account,
            vault::delete_account,
            vault::export_accounts,
            vault::import_accounts,
            vault::clear_accounts,
            // Roblox API commands
            roblox::validate_cookie,
            roblox::refresh_account_data,
            roblox::get_user_presence,
            roblox::get_user_details,
            roblox::get_user_by_username,
            roblox::get_user_game_info,
            // Launcher commands
            launcher::launch_game,
            launcher::launch_vip_server,
            launcher::kill_instance,
            launcher::get_active_instances,
            launcher::bypass_mutex,
            // Quick Login commands
            quick_login::quick_login_create,
            quick_login::quick_login_poll,
            quick_login::quick_login_complete,
            // Browser Login commands (auto cookie extraction)
            browser_login::browser_login_open,
            browser_login::browser_login_check,
            browser_login::browser_login_close,
            // Game Detection commands
            game_detection::get_universe_id,
            game_detection::get_game_info,
            game_detection::batch_get_game_icons,
            game_detection::get_game_servers,
            game_detection::get_popular_games,
            // Group commands
            groups::join_group,
            groups::leave_group,
            groups::get_group_info,
            // Utility commands
            utils::open_in_browser,
            utils::handle_deep_link,
            utils::set_always_on_top,
            // Settings commands
            settings::get_settings,
            settings::save_settings,
            settings::reset_settings,
            // Environment commands (for multi-instance launching)
            environment::create_environment,
            environment::write_cookies,
            environment::remove_environment,
            environment::environment_exists,
            environment::create_keychain,
            environment::unlock_keychain,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ROKIO");
}

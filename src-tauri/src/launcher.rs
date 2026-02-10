//! ROKIO Launcher Module
//! Handles launching Roblox with deep links (cross-platform).

use crate::crypto::CryptoState;
use crate::environment;
use crate::settings::get_settings;
use crate::vault::{load_accounts, save_accounts};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
#[cfg(not(target_os = "windows"))]
use std::process::Stdio;
use std::sync::Mutex;
use tauri::Manager;

/// Active Roblox instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveInstance {
    pub pid: u32,
    pub account_id: String,
    pub username: String,
    pub place_id: u64,
    pub started_at: u64,
}

/// Global state for tracking active instances
pub struct LauncherState {
    pub instances: Mutex<HashMap<u32, ActiveInstance>>,
}

impl Default for LauncherState {
    fn default() -> Self {
        Self {
            instances: Mutex::new(HashMap::new()),
        }
    }
}

// ============================================================================
// DEEP LINK LAUNCH (Cross-platform) - Simple mode
// ============================================================================

/// Build the Roblox deep link URL
fn build_deep_link(place_id: u64, job_id: Option<&str>) -> String {
    if let Some(job) = job_id {
        format!(
            "roblox://experiences/start?placeId={}&gameInstanceId={}",
            place_id, job
        )
    } else {
        format!("roblox://experiences/start?placeId={}", place_id)
    }
}

/// Build VIP/Private server deep link URL
fn build_vip_deep_link(place_id: u64, link_code: &str) -> String {
    format!("roblox://placeId={}&linkCode={}", place_id, link_code)
}

/// Find all running RobloxPlayer PIDs (macOS only)
#[cfg(target_os = "macos")]
fn find_roblox_pids() -> Vec<u32> {
    Command::new("pgrep")
        .arg("RobloxPlayer")
        .output()
        .ok()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| line.trim().parse::<u32>().ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Find the new RobloxPlayer PID (the one that wasn't in before_pids)
#[cfg(target_os = "macos")]
fn find_new_roblox_pid(before_pids: &[u32]) -> Result<u32, String> {
    // Try a few times in case Roblox is slow to start
    for _ in 0..5 {
        let after_pids = find_roblox_pids();
        for pid in &after_pids {
            if !before_pids.contains(pid) {
                return Ok(*pid);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    // If no new PID found, return the latest one (or error)
    find_roblox_pids()
        .into_iter()
        .max()
        .ok_or_else(|| "Failed to find Roblox process. Is Roblox installed?".to_string())
}

/// Launch Roblox to game menu (no specific game)
#[cfg(target_os = "macos")]
pub fn launch_roblox_menu() -> Result<u32, String> {
    // Count existing Roblox processes before launch
    let before_pids = find_roblox_pids();
    
    Command::new("open")
        .arg("-a")
        .arg("Roblox")
        .spawn()
        .map_err(|e| format!("Failed to open Roblox: {}", e))?;

    // Wait a moment for Roblox to start, then find the new PID
    std::thread::sleep(std::time::Duration::from_millis(1500));
    find_new_roblox_pid(&before_pids)
}

#[cfg(target_os = "windows")]
pub fn launch_roblox_menu() -> Result<u32, String> {
    let before_pids = find_roblox_pids();

    Command::new("cmd")
        .args(["/C", "start", "", "roblox://"])
        .spawn()
        .map_err(|e| format!("Failed to open Roblox: {}", e))?;

    std::thread::sleep(std::time::Duration::from_millis(2000));
    find_new_roblox_pid(&before_pids)
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn launch_roblox_menu() -> Result<u32, String> {
    Err("Roblox launch is not supported on this platform".to_string())
}

/// Launch Roblox via deep link on macOS
#[cfg(target_os = "macos")]
pub fn launch_roblox_deeplink(place_id: u64, job_id: Option<&str>) -> Result<u32, String> {
    // If place_id is 0, launch to menu instead
    if place_id == 0 {
        return launch_roblox_menu();
    }
    
    let deep_link = build_deep_link(place_id, job_id);

    // Count existing Roblox processes before launch
    let before_pids = find_roblox_pids();
    
    Command::new("open")
        .arg(&deep_link)
        .spawn()
        .map_err(|e| format!("Failed to open Roblox: {}", e))?;

    // Wait a moment for Roblox to start, then find the new PID
    std::thread::sleep(std::time::Duration::from_millis(1500));
    find_new_roblox_pid(&before_pids)
}

// ============================================================================
// WINDOWS SUPPORT - PID detection via wmic/tasklist
// ============================================================================

/// Find all running RobloxPlayerBeta PIDs (Windows)
#[cfg(target_os = "windows")]
fn find_roblox_pids() -> Vec<u32> {
    // Use wmic to get RobloxPlayerBeta process IDs
    Command::new("wmic")
        .args(["process", "where", "name='RobloxPlayerBeta.exe'", "get", "ProcessId"])
        .output()
        .ok()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| line.trim().parse::<u32>().ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Find the new RobloxPlayerBeta PID (Windows)
#[cfg(target_os = "windows")]
fn find_new_roblox_pid(before_pids: &[u32]) -> Result<u32, String> {
    for _ in 0..10 {
        let after_pids = find_roblox_pids();
        for pid in &after_pids {
            if !before_pids.contains(pid) {
                return Ok(*pid);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    find_roblox_pids()
        .into_iter()
        .max()
        .ok_or_else(|| "Failed to find Roblox process. Is Roblox installed?".to_string())
}

#[cfg(target_os = "windows")]
pub fn launch_roblox_deeplink(place_id: u64, job_id: Option<&str>) -> Result<u32, String> {
    if place_id == 0 {
        return launch_roblox_menu();
    }
    
    let deep_link = build_deep_link(place_id, job_id);
    let before_pids = find_roblox_pids();
    
    Command::new("cmd")
        .args(["/C", "start", "", &deep_link])
        .spawn()
        .map_err(|e| format!("Failed to open Roblox: {}", e))?;

    std::thread::sleep(std::time::Duration::from_millis(2000));
    find_new_roblox_pid(&before_pids)
}

// ============================================================================
// LINUX SUPPORT - Sober (Roblox via Flatpak)
// ============================================================================

/// Find all running Sober PIDs (Linux)
#[cfg(target_os = "linux")]
fn find_sober_pids() -> Vec<u32> {
    Command::new("pgrep")
        .args(["-f", "org.vinegarhq.Sober"])
        .output()
        .ok()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| line.trim().parse::<u32>().ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Find the new Sober PID (Linux)
#[cfg(target_os = "linux")]
fn find_new_sober_pid(before_pids: &[u32]) -> Result<u32, String> {
    for _ in 0..10 {
        let after_pids = find_sober_pids();
        for pid in &after_pids {
            if !before_pids.contains(pid) {
                return Ok(*pid);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    find_sober_pids()
        .into_iter()
        .max()
        .ok_or_else(|| "Failed to find Sober process. Is Sober installed via Flatpak?".to_string())
}

/// Launch Sober to menu (Linux)
#[cfg(target_os = "linux")]
pub fn launch_roblox_menu() -> Result<u32, String> {
    let before_pids = find_sober_pids();
    
    Command::new("flatpak")
        .args(["run", "org.vinegarhq.Sober"])
        .spawn()
        .map_err(|e| format!("Failed to launch Sober: {}", e))?;

    std::thread::sleep(std::time::Duration::from_millis(2000));
    find_new_sober_pid(&before_pids)
}

/// Launch Sober with deeplink (Linux)
#[cfg(target_os = "linux")]
pub fn launch_roblox_deeplink(place_id: u64, job_id: Option<&str>) -> Result<u32, String> {
    if place_id == 0 {
        return launch_roblox_menu();
    }
    
    let deep_link = build_deep_link(place_id, job_id);
    let before_pids = find_sober_pids();
    
    Command::new("flatpak")
        .args(["run", "org.vinegarhq.Sober", &deep_link])
        .spawn()
        .map_err(|e| format!("Failed to launch Sober: {}", e))?;

    std::thread::sleep(std::time::Duration::from_millis(2000));
    find_new_sober_pid(&before_pids)
}

/// Launch Sober with custom HOME for multi-instance (Linux)
#[cfg(target_os = "linux")]
pub fn launch_with_custom_home(
    home_dir: &std::path::Path,
    place_id: u64,
    job_id: Option<&str>,
) -> Result<u32, String> {
    let before_pids = find_sober_pids();
    
    let mut cmd = Command::new("flatpak");
    cmd.arg("run")
        .arg("--env=HOME=".to_owned() + home_dir.to_str().unwrap_or("/tmp"))
        .arg("org.vinegarhq.Sober");
    
    if place_id > 0 {
        cmd.arg(build_deep_link(place_id, job_id));
    }
    
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch Sober: {}", e))?;

    std::thread::sleep(std::time::Duration::from_millis(2000));
    find_new_sober_pid(&before_pids)
}

/// Fallback for unsupported platforms
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn launch_roblox_deeplink(_place_id: u64, _job_id: Option<&str>) -> Result<u32, String> {
    Err("Roblox launch is not supported on this platform".to_string())
}

// ============================================================================
// MULTI-INSTANCE LAUNCH (macOS only) - Launches RobloxPlayer with custom HOME
// ============================================================================

/// Find the Roblox.app path on macOS
#[cfg(target_os = "macos")]
fn find_roblox_app() -> Result<std::path::PathBuf, String> {
    use std::path::PathBuf;
    
    // Check common installation locations
    let mut locations = vec![PathBuf::from("/Applications/Roblox.app")];
    
    if let Some(home) = dirs::home_dir() {
        locations.push(home.join("Applications/Roblox.app"));
    }

    for loc in locations {
        if loc.exists() {
            return Ok(loc);
        }
    }

    Err("Roblox.app not found. Please install Roblox first.".to_string())
}

/// Launch Roblox with custom HOME directory for multi-instance support
/// Note: This launches to main menu. Game join is handled separately.
#[cfg(target_os = "macos")]
pub fn launch_with_custom_home(
    home_dir: &std::path::Path,
    _place_id: u64,
    _job_id: Option<&str>,
) -> Result<u32, String> {
    let roblox_app = find_roblox_app()?;
    let player_path = roblox_app
        .join("Contents")
        .join("MacOS")
        .join("RobloxPlayer");

    if !player_path.exists() {
        return Err(format!(
            "RobloxPlayer not found at: {}",
            player_path.display()
        ));
    }

    // Launch RobloxPlayer with custom HOME environment (like raptormanager)
    // Roblox will read cookies from {HOME}/Library/HTTPStorages/
    // Use Stdio::null() to let the process run independently
    let child = Command::new(&player_path)
        .env("HOME", home_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch Roblox: {}", e))?;

    Ok(child.id())
}

/// Launch with custom AppData for multi-instance (Windows)
#[cfg(target_os = "windows")]
pub fn launch_with_custom_home(
    home_dir: &std::path::Path,
    place_id: u64,
    job_id: Option<&str>,
) -> Result<u32, String> {
    let local_appdata = home_dir.join("LocalAppData");
    let appdata = home_dir.join("AppData");

    // Build deep link
    let deep_link = if place_id > 0 {
        build_deep_link(place_id, job_id)
    } else {
        "roblox://".to_string()
    };

    let before_pids = find_roblox_pids();

    // Launch Roblox with custom environment variables
    // This makes Roblox store cookies and data in isolated directories
    Command::new("cmd")
        .args(["/C", "start", "", &deep_link])
        .env("LOCALAPPDATA", &local_appdata)
        .env("APPDATA", &appdata)
        .spawn()
        .map_err(|e| format!("Failed to launch Roblox: {}", e))?;

    std::thread::sleep(std::time::Duration::from_millis(2000));
    find_new_roblox_pid(&before_pids)
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn launch_with_custom_home(
    _home_dir: &std::path::Path,
    _place_id: u64,
    _job_id: Option<&str>,
) -> Result<u32, String> {
    Err("Multi-instance launch is not supported on this platform".to_string())
}

// ============================================================================
// PROCESS UTILITIES
// ============================================================================

// Re-export specific functions if needed, or just use crate::process_utils directly
use crate::process_utils::{kill_process, is_process_running};

// ============================================================================
// TAURI COMMANDS
// ============================================================================

use crate::profiles::Profile;
use crate::settings::AppSettings;
use std::path::PathBuf;

/// Shared launch preparation: unlock vault, load accounts, get settings & timestamps
struct LaunchContext {
    account: Profile,
    accounts: Vec<Profile>,
    key: [u8; 32],
    app_data_dir: PathBuf,
    settings: AppSettings,
    now_ms: u64,
    now_secs: u64,
}

fn prepare_launch(
    app_handle: &tauri::AppHandle,
    account_id: &str,
    crypto_state: &tauri::State<'_, CryptoState>,
) -> Result<LaunchContext, String> {
    let key = crypto_state
        .key
        .lock()
        .unwrap()
        .ok_or("Vault is locked")?;

    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;

    let accounts = load_accounts(&app_data_dir, &key)?;
    let account = accounts
        .iter()
        .find(|a| a.id == account_id)
        .ok_or("Account not found")?
        .clone();

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let now_secs = now_ms / 1000;

    let settings = get_settings(app_handle.clone()).unwrap_or_default();

    Ok(LaunchContext {
        account,
        accounts,
        key,
        app_data_dir,
        settings,
        now_ms,
        now_secs,
    })
}

/// Shared launch finalization: update timestamp, save accounts, track instance
fn finalize_launch(
    ctx: LaunchContext,
    pid: u32,
    place_id: u64,
    account_id: &str,
    launcher_state: &tauri::State<'_, LauncherState>,
) -> Result<ActiveInstance, String> {
    let mut accounts = ctx.accounts;

    // Update last_played_at timestamp
    if let Some(acc) = accounts.iter_mut().find(|a| a.id == account_id) {
        acc.last_played_at = ctx.now_ms;
    }
    let _ = save_accounts(&ctx.app_data_dir, &ctx.key, &accounts);

    // Track the instance
    let instance = ActiveInstance {
        pid,
        account_id: ctx.account.id.clone(),
        username: ctx.account.username.clone(),
        place_id,
        started_at: ctx.now_secs,
    };

    launcher_state
        .instances
        .lock()
        .unwrap()
        .insert(pid, instance.clone());

    Ok(instance)
}

/// Set up multi-instance environment (keychain, cookies, custom HOME)
fn setup_multi_instance_env(
    app_handle: &tauri::AppHandle,
    account_id: &str,
    cookie: &str,
) -> Result<std::path::PathBuf, String> {
    environment::create_environment(app_handle.clone(), account_id.to_string())?;
    environment::create_keychain(app_handle.clone(), account_id.to_string())?;
    environment::unlock_keychain(app_handle.clone(), account_id.to_string())?;
    environment::write_cookies(
        app_handle.clone(),
        account_id.to_string(),
        cookie.to_string(),
    )?;
    Ok(environment::get_launch_home_dir(app_handle, account_id))
}

/// Launch a game with a specific account
#[tauri::command]
pub async fn launch_game(
    app_handle: tauri::AppHandle,
    account_id: String,
    place_id: u64,
    job_id: Option<String>,
    crypto_state: tauri::State<'_, CryptoState>,
    launcher_state: tauri::State<'_, LauncherState>,
) -> Result<ActiveInstance, String> {
    let ctx = prepare_launch(&app_handle, &account_id, &crypto_state)?;

    let pid = if ctx.settings.multi_instance {
        let home_dir =
            setup_multi_instance_env(&app_handle, &account_id, &ctx.account.cookie)?;
        launch_with_custom_home(&home_dir, place_id, job_id.as_deref())?
    } else {
        launch_roblox_deeplink(place_id, job_id.as_deref())?
    };

    finalize_launch(ctx, pid, place_id, &account_id, &launcher_state)
}

/// Kill a running Roblox instance
#[tauri::command]
pub fn kill_instance(
    pid: u32,
    launcher_state: tauri::State<'_, LauncherState>,
) -> Result<(), String> {
    kill_process(pid)?;
    launcher_state.instances.lock().unwrap().remove(&pid);
    Ok(())
}

/// Get all active instances
#[tauri::command]
pub fn get_active_instances(
    launcher_state: tauri::State<'_, LauncherState>,
) -> Vec<ActiveInstance> {
    let mut instances = launcher_state.instances.lock().unwrap();

    // Clean up dead instances
    let dead_pids: Vec<u32> = instances
        .iter()
        .filter(|(pid, _)| !is_process_running(**pid))
        .map(|(pid, _)| *pid)
        .collect();

    for pid in dead_pids {
        instances.remove(&pid);
    }

    instances.values().cloned().collect()
}

/// Bypass the singleton mutex (placeholder for multi-instance)
#[tauri::command]
pub fn bypass_mutex() -> Result<u32, String> {
    Ok(0)
}

/// Launch a VIP/Private server with a specific account
#[tauri::command]
pub async fn launch_vip_server(
    app_handle: tauri::AppHandle,
    account_id: String,
    place_id: u64,
    link_code: String,
    crypto_state: tauri::State<'_, CryptoState>,
    launcher_state: tauri::State<'_, LauncherState>,
) -> Result<ActiveInstance, String> {
    let ctx = prepare_launch(&app_handle, &account_id, &crypto_state)?;

    let vip_link = build_vip_deep_link(place_id, &link_code);

    // Set up multi-instance environment if enabled (but VIP always launches via deeplink)
    if ctx.settings.multi_instance {
        let _ = setup_multi_instance_env(&app_handle, &account_id, &ctx.account.cookie)?;
    }

    // Launch via platform-specific deeplink
    #[cfg(target_os = "macos")]
    let pid = {
        let before_pids = find_roblox_pids();
        Command::new("open")
            .arg(&vip_link)
            .spawn()
            .map_err(|e| format!("Failed to open Roblox: {}", e))?;
        std::thread::sleep(std::time::Duration::from_millis(1500));
        find_new_roblox_pid(&before_pids)?
    };

    #[cfg(target_os = "windows")]
    let pid = {
        let before_pids = find_roblox_pids();
        Command::new("cmd")
            .args(["/C", "start", "", &vip_link])
            .spawn()
            .map_err(|e| format!("Failed to open Roblox: {}", e))?;
        std::thread::sleep(std::time::Duration::from_millis(2000));
        find_new_roblox_pid(&before_pids)?
    };

    #[cfg(target_os = "linux")]
    let pid = {
        let before_pids = find_sober_pids();
        Command::new("flatpak")
            .args(["run", "org.vinegarhq.Sober", &vip_link])
            .spawn()
            .map_err(|e| format!("Failed to launch Sober: {}", e))?;
        std::thread::sleep(std::time::Duration::from_millis(2000));
        find_new_sober_pid(&before_pids)?
    };

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    let pid: u32 = return Err("VIP server launch not supported on this platform".to_string());

    finalize_launch(ctx, pid, place_id, &account_id, &launcher_state)
}


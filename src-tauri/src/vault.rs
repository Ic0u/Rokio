//! ROKIO Vault Module
//! Handles encrypted account storage (load/save operations).

use crate::crypto::{decrypt_string, encrypt_string, vault_path, CryptoState};
use crate::profiles::Profile;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Manager;

/// Vault data structure (stored encrypted on disk)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultData {
    pub version: u32,
    pub verification: String,
    pub accounts: Vec<EncryptedAccount>,
}

/// Encrypted account (stored in vault)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedAccount {
    pub id: String,
    pub encrypted_cookie: String,
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub alias: String,
    #[serde(default)]
    pub description: String,
    pub is_favorite: bool,
    pub last_played_at: u64,
    pub created_at: u64,
}

// ============================================================================
// VAULT OPERATIONS
// ============================================================================

/// Load all accounts from the encrypted vault
pub fn load_accounts(
    app_data_dir: &std::path::Path,
    key: &[u8; 32],
) -> Result<Vec<Profile>, String> {
    let vault_file = vault_path(app_data_dir);

    if !vault_file.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&vault_file).map_err(|e| e.to_string())?;

    let vault: VaultData = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    // Decrypt each account's cookie
    let accounts: Result<Vec<Profile>, String> = vault
        .accounts
        .into_iter()
        .map(|enc_acc| {
            let cookie = decrypt_string(&enc_acc.encrypted_cookie, key)?;
            Ok(Profile {
                id: enc_acc.id,
                cookie,
                user_id: enc_acc.user_id,
                username: enc_acc.username,
                display_name: enc_acc.display_name,
                thumbnail: enc_acc.thumbnail,
                alias: enc_acc.alias,
                description: enc_acc.description,
                is_favorite: enc_acc.is_favorite,
                last_played_at: enc_acc.last_played_at,
            })
        })
        .collect();

    accounts
}

/// Save all accounts to the encrypted vault
pub fn save_accounts(
    app_data_dir: &std::path::Path,
    key: &[u8; 32],
    accounts: &[Profile],
) -> Result<(), String> {
    let vault_file = vault_path(app_data_dir);

    // Read existing vault to preserve verification string
    let content = fs::read_to_string(&vault_file).map_err(|e| e.to_string())?;
    let mut vault: VaultData = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    // Encrypt each account's cookie
    vault.accounts = accounts
        .iter()
        .map(|acc| {
            let encrypted_cookie = encrypt_string(&acc.cookie, key)?;
            Ok(EncryptedAccount {
                id: acc.id.clone(),
                encrypted_cookie,
                user_id: acc.user_id,
                username: acc.username.clone(),
                display_name: acc.display_name.clone(),
                thumbnail: acc.thumbnail.clone(),
                alias: acc.alias.clone(),
                description: acc.description.clone(),
                is_favorite: acc.is_favorite,
                last_played_at: acc.last_played_at,
                created_at: chrono::Utc::now().timestamp() as u64,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    // Write back
    let json = serde_json::to_string_pretty(&vault).map_err(|e| e.to_string())?;
    fs::write(&vault_file, json).map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

/// Get all accounts (decrypted)
#[tauri::command]
pub fn get_accounts(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, CryptoState>,
) -> Result<Vec<Profile>, String> {
    let key = state
        .key
        .lock()
        .unwrap()
        .ok_or("Vault is locked")?;

    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;

    load_accounts(&app_data_dir, &key)
}

/// Add a new account
#[tauri::command]
pub async fn add_account(
    app_handle: tauri::AppHandle,
    cookie: String,
    state: tauri::State<'_, CryptoState>,
) -> Result<Profile, String> {
    let key = state
        .key
        .lock()
        .unwrap()
        .ok_or("Vault is locked")?;

    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;

    // Validate cookie and get user data from Roblox API
    let user_data = crate::roblox::validate_and_get_user(&cookie).await?;

    // Create new profile
    let profile = Profile {
        id: uuid::Uuid::new_v4().to_string(),
        cookie: cookie.clone(),
        user_id: user_data.id,
        username: user_data.name,
        display_name: user_data.display_name,
        thumbnail: user_data.thumbnail,
        alias: String::new(),
        description: String::new(),
        is_favorite: false,
        last_played_at: 0,
    };

    // Load existing accounts
    let mut accounts = load_accounts(&app_data_dir, &key)?;

    // Check for duplicate
    if accounts.iter().any(|a| a.user_id == profile.user_id) {
        return Err(format!(
            "Account {} is already added",
            profile.display_name
        ));
    }

    // Add and save
    accounts.push(profile.clone());
    save_accounts(&app_data_dir, &key, &accounts)?;

    Ok(profile)
}

/// Update an existing account
#[tauri::command]
pub fn update_account(
    app_handle: tauri::AppHandle,
    profile: Profile,
    state: tauri::State<'_, CryptoState>,
) -> Result<(), String> {
    let key = state
        .key
        .lock()
        .unwrap()
        .ok_or("Vault is locked")?;

    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;

    let mut accounts = load_accounts(&app_data_dir, &key)?;

    // Find and update
    if let Some(acc) = accounts.iter_mut().find(|a| a.id == profile.id) {
        acc.alias = profile.alias;
        acc.description = profile.description;
        acc.is_favorite = profile.is_favorite;
        acc.last_played_at = profile.last_played_at;
    } else {
        return Err("Account not found".to_string());
    }

    save_accounts(&app_data_dir, &key, &accounts)?;
    Ok(())
}

/// Delete an account
#[tauri::command]
pub fn delete_account(
    app_handle: tauri::AppHandle,
    id: String,
    state: tauri::State<'_, CryptoState>,
) -> Result<(), String> {
    let key = state
        .key
        .lock()
        .unwrap()
        .ok_or("Vault is locked")?;

    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;

    let mut accounts = load_accounts(&app_data_dir, &key)?;
    let original_len = accounts.len();

    accounts.retain(|a| a.id != id);

    if accounts.len() == original_len {
        return Err("Account not found".to_string());
    }

    save_accounts(&app_data_dir, &key, &accounts)?;
    Ok(())
}

/// Export accounts to JSON string (for backup)
#[tauri::command]
pub fn export_accounts(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, CryptoState>,
) -> Result<String, String> {
    let _key = state
        .key
        .lock()
        .unwrap()
        .ok_or("Vault is locked")?;

    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let vault_file = vault_path(&app_data_dir);

    if !vault_file.exists() {
        return Ok("{}".to_string());
    }

    // Return the raw encrypted vault file content
    fs::read_to_string(&vault_file).map_err(|e| e.to_string())
}

/// Import accounts from JSON string (from backup)
#[tauri::command]
pub fn import_accounts(
    app_handle: tauri::AppHandle,
    data: String,
    merge: bool,
    state: tauri::State<'_, CryptoState>,
) -> Result<usize, String> {
    let key = state
        .key
        .lock()
        .unwrap()
        .ok_or("Vault is locked")?;

    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;

    // Parse imported vault data
    let imported_vault: VaultData = serde_json::from_str(&data)
        .map_err(|e| format!("Invalid backup file: {}", e))?;

    // Decrypt imported accounts
    let imported_accounts: Vec<Profile> = imported_vault
        .accounts
        .into_iter()
        .filter_map(|enc_acc| {
            let cookie = decrypt_string(&enc_acc.encrypted_cookie, &key).ok()?;
            Some(Profile {
                id: enc_acc.id,
                cookie,
                user_id: enc_acc.user_id,
                username: enc_acc.username,
                display_name: enc_acc.display_name,
                thumbnail: enc_acc.thumbnail,
                alias: enc_acc.alias,
                description: enc_acc.description,
                is_favorite: enc_acc.is_favorite,
                last_played_at: enc_acc.last_played_at,
            })
        })
        .collect();

    let count = imported_accounts.len();

    if merge {
        // Merge with existing accounts
        let mut existing = load_accounts(&app_data_dir, &key).unwrap_or_default();
        for acc in imported_accounts {
            if !existing.iter().any(|e| e.user_id == acc.user_id) {
                existing.push(acc);
            }
        }
        save_accounts(&app_data_dir, &key, &existing)?;
    } else {
        // Replace all accounts
        save_accounts(&app_data_dir, &key, &imported_accounts)?;
    }

    Ok(count)
}

/// Clear all accounts
#[tauri::command]
pub fn clear_accounts(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, CryptoState>,
) -> Result<(), String> {
    let key = state
        .key
        .lock()
        .unwrap()
        .ok_or("Vault is locked")?;

    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    save_accounts(&app_data_dir, &key, &[])?;
    Ok(())
}


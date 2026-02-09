//! ROKIO Crypto Module
//! AES-256-GCM encryption with PBKDF2 key derivation from machine ID + password.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Mutex;
use tauri::Manager;

// ============================================================================
// CRYPTO STATE
// ============================================================================

/// Application state for managing encryption keys
pub struct CryptoState {
    /// Derived encryption key (32 bytes for AES-256)
    pub key: Mutex<Option<[u8; 32]>>,
    /// Whether the vault is currently unlocked
    pub unlocked: Mutex<bool>,
}

impl Default for CryptoState {
    fn default() -> Self {
        Self {
            key: Mutex::new(None),
            unlocked: Mutex::new(false),
        }
    }
}

/// Vault status information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    pub exists: bool,
    pub unlocked: bool,
}

// ============================================================================
// HARDWARE ID (Platform-specific)
// ============================================================================

/// Get a unique machine identifier for key derivation
#[cfg(target_os = "macos")]
pub fn get_machine_id() -> String {
    use mac_address2::get_mac_address;

    match get_mac_address() {
        Ok(Some(ma)) => {
            format!("ROKIO-MAC-{}", ma.to_string().replace(":", ""))
        }
        _ => {
            let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
            let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
            format!("ROKIO-FALLBACK-{}-{}", hostname, user)
        }
    }
}

#[cfg(target_os = "windows")]
pub fn get_machine_id() -> String {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    if let Ok(key) = hklm.open_subkey("SOFTWARE\\Microsoft\\Cryptography") {
        if let Ok(guid) = key.get_value::<String, _>("MachineGuid") {
            return format!("ROKIO-WIN-{}", guid.replace("-", ""));
        }
    }

    let computer = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string());
    let user = std::env::var("USERNAME").unwrap_or_else(|_| "user".to_string());
    format!("ROKIO-FALLBACK-{}-{}", computer, user)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn get_machine_id() -> String {
    let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
    let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    format!("ROKIO-LINUX-{}-{}", hostname, user)
}

// ============================================================================
// KEY DERIVATION
// ============================================================================

const PBKDF2_ITERATIONS: u32 = 100_000;
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

/// Derive a 32-byte encryption key from password + machine ID using PBKDF2
pub fn derive_key(password: &str) -> [u8; 32] {
    let machine_id = get_machine_id();
    let salt = format!("ROKIO-VAULT-{}", machine_id);

    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(
        password.as_bytes(),
        salt.as_bytes(),
        PBKDF2_ITERATIONS,
        &mut key,
    );
    key
}

// ============================================================================
// ENCRYPTION / DECRYPTION
// ============================================================================

/// Encrypt a string using AES-256-GCM
/// Returns: base64(nonce || ciphertext)
pub fn encrypt_string(plaintext: &str, key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Cipher init error: {}", e))?;

    // Generate random 12-byte nonce
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption error: {}", e))?;

    // Combine nonce + ciphertext and encode as base64
    let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

/// Decrypt a string using AES-256-GCM
/// Input: base64(nonce || ciphertext)
pub fn decrypt_string(encrypted: &str, key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Cipher init error: {}", e))?;

    // Decode base64
    let combined = BASE64
        .decode(encrypted)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    if combined.len() < NONCE_LEN {
        return Err("Invalid ciphertext: too short".to_string());
    }

    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed: invalid key or corrupted data".to_string())?;

    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 decode error: {}", e))
}

// ============================================================================
// VAULT FILE OPERATIONS
// ============================================================================

/// Check if the vault file exists
pub fn vault_exists(app_data_dir: &std::path::Path) -> bool {
    app_data_dir.join("vault.dat").exists()
}

/// Get the vault file path
pub fn vault_path(app_data_dir: &std::path::Path) -> std::path::PathBuf {
    app_data_dir.join("vault.dat")
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

/// Check vault status (exists + unlocked)
#[tauri::command]
pub fn get_vault_status(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, CryptoState>,
) -> Result<VaultStatus, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let exists = vault_exists(&app_data_dir);
    let unlocked = *state.unlocked.lock().unwrap();

    Ok(VaultStatus { exists, unlocked })
}

/// Get machine ID (for display)
#[tauri::command]
pub fn get_hwid() -> String {
    get_machine_id()
}

/// Create a new vault with a master password
#[tauri::command]
pub fn create_vault(
    app_handle: tauri::AppHandle,
    password: String,
    state: tauri::State<'_, CryptoState>,
) -> Result<(), String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    // Create app data directory if it doesn't exist
    std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;

    // Derive key from password
    let key = derive_key(&password);

    // Create empty vault with a verification string
    let verification = encrypt_string("ROKIO_VAULT_V1", &key)?;

    // Write vault file
    let vault_data = serde_json::json!({
        "version": 1,
        "verification": verification,
        "accounts": []
    });

    std::fs::write(vault_path(&app_data_dir), vault_data.to_string())
        .map_err(|e| format!("Failed to write vault: {}", e))?;

    // Store key in state
    *state.key.lock().unwrap() = Some(key);
    *state.unlocked.lock().unwrap() = true;

    Ok(())
}

/// Unlock an existing vault with password
#[tauri::command]
pub fn unlock_vault(
    app_handle: tauri::AppHandle,
    password: String,
    state: tauri::State<'_, CryptoState>,
) -> Result<bool, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    // Read vault file
    let vault_content =
        std::fs::read_to_string(vault_path(&app_data_dir)).map_err(|e| e.to_string())?;

    let vault: serde_json::Value =
        serde_json::from_str(&vault_content).map_err(|e| e.to_string())?;

    // Derive key and verify
    let key = derive_key(&password);
    let verification = vault["verification"]
        .as_str()
        .ok_or("Invalid vault format")?;

    match decrypt_string(verification, &key) {
        Ok(decrypted) if decrypted == "ROKIO_VAULT_V1" => {
            // Password correct - store key
            *state.key.lock().unwrap() = Some(key);
            *state.unlocked.lock().unwrap() = true;
            Ok(true)
        }
        _ => Ok(false), // Wrong password
    }
}

/// Lock the vault (clear key from memory)
#[tauri::command]
pub fn lock_vault(state: tauri::State<'_, CryptoState>) -> Result<(), String> {
    *state.key.lock().unwrap() = None;
    *state.unlocked.lock().unwrap() = false;
    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = derive_key("test_password");
        let plaintext = "Hello, ROKIO!";

        let encrypted = encrypt_string(plaintext, &key).unwrap();
        let decrypted = decrypt_string(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = derive_key("password1");
        let key2 = derive_key("password2");

        let encrypted = encrypt_string("secret", &key1).unwrap();
        let result = decrypt_string(&encrypted, &key2);

        assert!(result.is_err());
    }

    #[test]
    fn test_machine_id_not_empty() {
        let hwid = get_machine_id();
        assert!(!hwid.is_empty());
        assert!(hwid.starts_with("ROKIO-"));
    }
}

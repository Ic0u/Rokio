//! Groups API - Join Group with CSRF Token Handling
//!
//! The CSRF Challenge Pattern:
//! 1. Send POST request
//! 2. If 403 with x-csrf-token header, extract token
//! 3. Retry with X-CSRF-TOKEN header

use serde::{Deserialize, Serialize};
use crate::crypto::CryptoState;
use tauri::Manager;

/// CSRF Retry Wrapper
/// Handles the Roblox CSRF token challenge automatically
pub async fn csrf_post(
    client: &reqwest::Client,
    url: &str,
    cookie: &str,
    body: &str,
) -> Result<reqwest::Response, String> {
    // First attempt without CSRF token
    let response = client
        .post(url)
        .header("Cookie", format!(".ROBLOSECURITY={}", cookie))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    // If 403, extract CSRF token and retry
    if response.status() == 403 {
        if let Some(csrf_token) = response.headers().get("x-csrf-token") {
            let token_str = csrf_token
                .to_str()
                .map_err(|_| "Invalid CSRF token header")?;

            // Retry with CSRF token
            let retry_response = client
                .post(url)
                .header("Cookie", format!(".ROBLOSECURITY={}", cookie))
                .header("Content-Type", "application/json")
                .header("X-CSRF-TOKEN", token_str)
                .body(body.to_string())
                .send()
                .await
                .map_err(|e| format!("Retry request failed: {}", e))?;

            return Ok(retry_response);
        }
    }

    Ok(response)
}

/// Join a Roblox group
#[tauri::command]
pub async fn join_group(
    app_handle: tauri::AppHandle,
    account_id: String,
    group_id: u64,
    state: tauri::State<'_, CryptoState>,
) -> Result<String, String> {
    // Get cookie from vault
    let cookie = get_account_cookie(&app_handle, &account_id, &state)?;
    let client = reqwest::Client::new();
    let url = format!("https://groups.roblox.com/v1/groups/{}/users", group_id);

    let response = csrf_post(&client, &url, &cookie, "{}").await?;
    let status = response.status();

    if status.is_success() {
        Ok("Successfully joined group".to_string())
    } else if status == 400 {
        Err("Already in group or group is locked".to_string())
    } else if status == 403 {
        Err("Cannot join group (private or banned)".to_string())
    } else {
        let body = response.text().await.unwrap_or_default();
        Err(format!("Failed to join group: {} - {}", status, body))
    }
}

/// Leave a Roblox group
#[tauri::command]
pub async fn leave_group(
    app_handle: tauri::AppHandle,
    account_id: String,
    group_id: u64,
    state: tauri::State<'_, CryptoState>,
) -> Result<String, String> {
    let cookie = get_account_cookie(&app_handle, &account_id, &state)?;
    let client = reqwest::Client::new();
    
    // Need to get user ID for the leave endpoint
    let user_id = get_account_user_id(&app_handle, &account_id, &state)?;
    let url = format!(
        "https://groups.roblox.com/v1/groups/{}/users/{}",
        group_id, user_id
    );

    // First attempt
    let response = client
        .delete(&url)
        .header("Cookie", format!(".ROBLOSECURITY={}", cookie))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    // Handle CSRF if needed
    let final_response = if response.status() == 403 {
        if let Some(csrf_token) = response.headers().get("x-csrf-token") {
            client
                .delete(&url)
                .header("Cookie", format!(".ROBLOSECURITY={}", cookie))
                .header("X-CSRF-TOKEN", csrf_token.to_str().unwrap_or(""))
                .send()
                .await
                .map_err(|e| format!("Retry failed: {}", e))?
        } else {
            response
        }
    } else {
        response
    };

    if final_response.status().is_success() {
        Ok("Successfully left group".to_string())
    } else {
        let body = final_response.text().await.unwrap_or_default();
        Err(format!("Failed to leave group: {}", body))
    }
}

/// Get group info
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupInfo {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub member_count: u64,
    pub is_locked: bool,
}

#[tauri::command]
pub async fn get_group_info(group_id: u64) -> Result<GroupInfo, String> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("https://groups.roblox.com/v1/groups/{}", group_id))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch group: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Group not found: {}", group_id));
    }

    let info: GroupInfo = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse group: {}", e))?;

    Ok(info)
}

// Helper: Get account cookie from vault
fn get_account_cookie(
    app_handle: &tauri::AppHandle,
    account_id: &str,
    state: &tauri::State<'_, CryptoState>,
) -> Result<String, String> {
    let key = state
        .key
        .lock()
        .unwrap()
        .ok_or("Vault is locked")?;

    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let accounts = crate::vault::load_accounts(&app_data_dir, &key)?;

    accounts
        .iter()
        .find(|a| a.id == account_id)
        .map(|a| a.cookie.clone())
        .ok_or_else(|| "Account not found".to_string())
}

// Helper: Get account user_id from vault
fn get_account_user_id(
    app_handle: &tauri::AppHandle,
    account_id: &str,
    state: &tauri::State<'_, CryptoState>,
) -> Result<i64, String> {
    let key = state
        .key
        .lock()
        .unwrap()
        .ok_or("Vault is locked")?;

    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let accounts = crate::vault::load_accounts(&app_data_dir, &key)?;

    accounts
        .iter()
        .find(|a| a.id == account_id)
        .map(|a| a.user_id)
        .ok_or_else(|| "Account not found".to_string())
}

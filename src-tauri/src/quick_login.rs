//! Quick Login - Cross-Device QR Code Authentication
//!
//! Flow:
//! 1. Generate: POST /login/create → Returns code + QR URL
//! 2. Poll: POST /login/status every 2s until CONFIRMED
//! 3. Extract: Get .ROBLOSECURITY cookie

use serde::{Deserialize, Serialize};

/// Response from /login/create
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateLoginResponse {
    code: String,
    #[serde(rename = "qrCodeUrl")]
    qr_code_url: Option<String>,
}

/// Response from /login/status
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginStatusResponse {
    status: String,
}

/// Frontend-facing Quick Login session data
#[derive(Debug, Clone, Serialize)]
pub struct QuickLoginSession {
    pub code: String,
    pub qr_code_url: String,
}

/// Create a new Quick Login session
#[tauri::command]
pub async fn quick_login_create() -> Result<QuickLoginSession, String> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://apis.roblox.com/auth-token-service/v1/login/create")
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
        .map_err(|e| format!("Failed to create login session: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Create login failed: {} - {}", status, body));
    }

    let data: CreateLoginResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Generate QR code URL if not provided
    let qr_url = data.qr_code_url.unwrap_or_else(|| {
        format!(
            "https://api.qrserver.com/v1/create-qr-code/?size=200x200&data={}",
            urlencoding::encode(&format!("ROBLOX_QR:{}", data.code))
        )
    });

    Ok(QuickLoginSession {
        code: data.code,
        qr_code_url: qr_url,
    })
}

/// Poll the login status
#[tauri::command]
pub async fn quick_login_poll(code: String) -> Result<String, String> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://apis.roblox.com/auth-token-service/v1/login/status")
        .header("Content-Type", "application/json")
        .body(format!(r#"{{"code":"{}"}}"#, code))
        .send()
        .await
        .map_err(|e| format!("Failed to poll status: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Poll failed: {}", response.status()));
    }

    let data: LoginStatusResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse status: {}", e))?;

    Ok(data.status)
}

/// Complete the Quick Login - extract cookie
#[tauri::command]
pub async fn quick_login_complete(code: String) -> Result<String, String> {
    let client = reqwest::Client::new();

    // Redeem the code for authentication
    let response = client
        .post("https://apis.roblox.com/auth-token-service/v1/login/redeem")
        .header("Content-Type", "application/json")
        .body(format!(r#"{{"code":"{}"}}"#, code))
        .send()
        .await
        .map_err(|e| format!("Failed to redeem code: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Redeem failed: {}", response.status()));
    }

    // Extract .ROBLOSECURITY from Set-Cookie header
    let cookie = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|v| {
            let s = v.to_str().ok()?;
            if s.contains(".ROBLOSECURITY=") {
                let start = s.find(".ROBLOSECURITY=")? + 15;
                let end = s[start..].find(';').map(|i| start + i).unwrap_or(s.len());
                Some(s[start..end].to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| "No .ROBLOSECURITY cookie in response".to_string())?;

    // Validate the cookie by fetching user info
    let user_response = client
        .get("https://users.roblox.com/v1/users/authenticated")
        .header("Cookie", format!(".ROBLOSECURITY={}", cookie))
        .send()
        .await
        .map_err(|e| format!("Failed to validate cookie: {}", e))?;

    if !user_response.status().is_success() {
        return Err("Cookie validation failed".to_string());
    }

    #[derive(Deserialize)]
    struct AuthUser {
        id: u64,
        name: String,
        #[serde(rename = "displayName")]
        display_name: String,
    }

    let user: AuthUser = user_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse user: {}", e))?;

    // Return data for frontend to use with add_account
    Ok(format!(
        "{}|{}|{}|{}",
        user.id, user.name, user.display_name, cookie
    ))
}

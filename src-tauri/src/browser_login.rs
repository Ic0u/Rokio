//! Browser Login Module
//! Opens a webview to Roblox login and auto-extracts cookie after successful login

use tauri::{AppHandle, Manager, WebviewWindowBuilder, WebviewUrl};
use std::sync::Arc;
use tokio::sync::Mutex;

/// State to track browser login session
pub struct BrowserLoginState {
    pub cookie: Arc<Mutex<Option<String>>>,
    pub completed: Arc<Mutex<bool>>,
}

impl Default for BrowserLoginState {
    fn default() -> Self {
        Self {
            cookie: Arc::new(Mutex::new(None)),
            completed: Arc::new(Mutex::new(false)),
        }
    }
}

/// Open a new webview window for Roblox login
#[tauri::command]
pub async fn browser_login_open(app: AppHandle) -> Result<(), String> {
    // Create a new webview window for login
    let _login_window = WebviewWindowBuilder::new(
        &app,
        "roblox-login",
        WebviewUrl::External("https://www.roblox.com/login".parse().unwrap()),
    )
    .title("Login to Roblox")
    .inner_size(450.0, 650.0)
    .resizable(true)
    .center()
    .build()
    .map_err(|e| format!("Failed to open login window: {}", e))?;

    Ok(())
}

/// Check if the login window has the .ROBLOSECURITY cookie
#[tauri::command]
pub async fn browser_login_check(app: AppHandle) -> Result<Option<String>, String> {
    // Try to get the login window
    let login_window = match app.get_webview_window("roblox-login") {
        Some(w) => w,
        None => return Ok(None), // Window closed, no cookie
    };

    // Get cookies from the webview
    let cookies = login_window
        .cookies()
        .map_err(|e| format!("Failed to get cookies: {}", e))?;

    // Look for .ROBLOSECURITY cookie
    for cookie in cookies {
        if cookie.name() == ".ROBLOSECURITY" {
            let value = cookie.value().to_string();
            // Close the login window
            let _ = login_window.close();
            return Ok(Some(value));
        }
    }

    Ok(None)
}

/// Close the browser login window
#[tauri::command]
pub async fn browser_login_close(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("roblox-login") {
        window.close().map_err(|e| format!("Failed to close: {}", e))?;
    }
    Ok(())
}

//! Utilities Module
//! Helper functions for Roblox integration

use rand::Rng;

/// List of common user agents for rotation

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15",
];

/// Get a random user agent string

pub fn get_random_user_agent() -> &'static str {
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..USER_AGENTS.len());
    USER_AGENTS[idx]
}

/// Extract Place ID from various Roblox URL formats
/// Supports:
/// - https://www.roblox.com/games/123456789/Game-Name
/// - https://roblox.com/games/123456789
/// - roblox://placeId=123456789
/// - 123456789 (raw ID)

pub fn extract_place_id_from_url(url: &str) -> Option<u64> {
    let url = url.trim();

    // Try parsing as raw number first
    if let Ok(id) = url.parse::<u64>() {
        return Some(id);
    }

    // Try roblox:// protocol
    if url.starts_with("roblox://") {
        // roblox://placeId=123456789
        if let Some(start) = url.find("placeId=") {
            let after = &url[start + 8..];
            let end = after.find('&').unwrap_or(after.len());
            if let Ok(id) = after[..end].parse::<u64>() {
                return Some(id);
            }
        }
    }

    // Try web URL: /games/123456789/...
    if url.contains("/games/") {
        let parts: Vec<&str> = url.split("/games/").collect();
        if parts.len() > 1 {
            let after_games = parts[1];
            // Take until next slash or end
            let end = after_games.find('/').unwrap_or(after_games.len());
            if let Ok(id) = after_games[..end].parse::<u64>() {
                return Some(id);
            }
        }
    }

    None
}

/// Extract Job ID from a Roblox deep link
/// roblox://placeId=123&gameInstanceId=abc-def-ghi

pub fn extract_job_id_from_url(url: &str) -> Option<String> {
    if let Some(start) = url.find("gameInstanceId=") {
        let after = &url[start + 15..];
        let end = after.find('&').unwrap_or(after.len());
        let job_id = &after[..end];
        if !job_id.is_empty() {
            return Some(job_id.to_string());
        }
    }
    None
}

/// Open URL in system default browser
#[tauri::command]
pub async fn open_in_browser(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }

    Ok(())
}

/// Handle deep link (roblox:// protocol)
#[tauri::command]
pub async fn handle_deep_link(url: String) -> Result<(), String> {
    // For now, just open in system browser
    // The system will handle roblox:// protocol
    open_in_browser(url).await
}

/// Set window always on top
#[tauri::command]
pub fn set_always_on_top(window: tauri::Window, enabled: bool) -> Result<(), String> {
    window
        .set_always_on_top(enabled)
        .map_err(|e| format!("Failed to set always on top: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_place_id() {
        assert_eq!(extract_place_id_from_url("123456789"), Some(123456789));
        assert_eq!(
            extract_place_id_from_url("https://www.roblox.com/games/123456789/Game-Name"),
            Some(123456789)
        );
        assert_eq!(
            extract_place_id_from_url("roblox://placeId=123456789"),
            Some(123456789)
        );
        assert_eq!(
            extract_place_id_from_url("roblox://placeId=123456789&gameInstanceId=abc"),
            Some(123456789)
        );
        assert_eq!(extract_place_id_from_url("invalid"), None);
    }

    #[test]
    fn test_extract_job_id() {
        assert_eq!(
            extract_job_id_from_url("roblox://placeId=123&gameInstanceId=abc-def-ghi"),
            Some("abc-def-ghi".to_string())
        );
        assert_eq!(extract_job_id_from_url("roblox://placeId=123"), None);
    }
}

//! ROKIO Roblox API Module
//! Validates cookies and fetches user data from Roblox APIs.

use reqwest::header::{HeaderMap, HeaderValue, COOKIE, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

const ROBLOX_AUTH_API: &str = "https://users.roblox.com/v1/users/authenticated";
const ROBLOX_THUMBNAIL_API: &str = "https://thumbnails.roblox.com/v1/users/avatar-headshot";
const ROBLOX_PRESENCE_API: &str = "https://presence.roblox.com/v1/presence/users";

/// Roblox user data from API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobloxUser {
    pub id: i64,
    pub name: String,
    pub display_name: String,
}

/// Extended user data with thumbnail
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobloxUserData {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub thumbnail: Option<String>,
}

/// Thumbnail response from Roblox API
#[derive(Debug, Clone, Deserialize)]
pub struct ThumbnailResponse {
    pub data: Vec<ThumbnailData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailData {
    #[allow(dead_code)]
    pub target_id: i64,
    #[allow(dead_code)]
    pub state: String,
    pub image_url: Option<String>,
}

/// User presence response from Roblox API
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceResponse {
    pub user_presences: Vec<UserPresenceData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPresenceData {
    pub user_presence_type: i32,
    pub last_location: Option<String>,
    pub place_id: Option<i64>,
    pub root_place_id: Option<i64>,
    pub game_id: Option<String>,
    pub universe_id: Option<i64>,
    pub user_id: i64,
    pub last_online: Option<String>,
}

/// Simplified presence status for frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPresence {
    pub status: String,        // "offline", "online", "ingame", "studio"
    pub last_location: Option<String>,
    pub place_id: Option<i64>,
}

// ============================================================================
// API FUNCTIONS
// ============================================================================

/// Validate a .ROBLOSECURITY cookie and get user data
pub async fn validate_and_get_user(cookie: &str) -> Result<RobloxUserData, String> {
    let client = reqwest::Client::new();

    // Prepare cookie header
    let cookie_value = if cookie.starts_with(".ROBLOSECURITY=") {
        cookie.to_string()
    } else {
        format!(".ROBLOSECURITY={}", cookie)
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        HeaderValue::from_str(&cookie_value).map_err(|e| e.to_string())?,
    );

    // Get authenticated user
    let response = client
        .get(ROBLOX_AUTH_API)
        .headers(headers.clone())
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status() == 401 {
        return Err("Invalid or expired cookie".to_string());
    }

    if response.status() == 403 {
        return Err("Cookie rejected by Roblox (possibly banned)".to_string());
    }

    if !response.status().is_success() {
        return Err(format!("Roblox API error: {}", response.status()));
    }

    let user: RobloxUser = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Get thumbnail
    let thumbnail = get_user_thumbnail(&client, user.id).await.ok();

    Ok(RobloxUserData {
        id: user.id,
        name: user.name,
        display_name: user.display_name,
        thumbnail,
    })
}

/// Get user avatar thumbnail
async fn get_user_thumbnail(client: &reqwest::Client, user_id: i64) -> Result<String, String> {
    let url = format!(
        "{}?userIds={}&size=150x150&format=Png&isCircular=false",
        ROBLOX_THUMBNAIL_API, user_id
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Thumbnail fetch error: {}", e))?;

    let thumbnail_response: ThumbnailResponse = response
        .json()
        .await
        .map_err(|e| format!("Thumbnail parse error: {}", e))?;

    thumbnail_response
        .data
        .first()
        .and_then(|d| d.image_url.clone())
        .ok_or_else(|| "No thumbnail available".to_string())
}

/// Fetch user presence (Online/Offline/InGame/Studio)
async fn fetch_user_presence(user_id: i64) -> Result<UserPresence, String> {
    let client = reqwest::Client::new();
    
    let body = serde_json::json!({
        "userIds": [user_id]
    });

    let response = client
        .post(ROBLOX_PRESENCE_API)
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Presence fetch error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Presence API error: {}", response.status()));
    }

    let presence_response: PresenceResponse = response
        .json()
        .await
        .map_err(|e| format!("Presence parse error: {}", e))?;

    let presence_data = presence_response
        .user_presences
        .first()
        .ok_or_else(|| "No presence data".to_string())?;

    // Map presence type to status string
    // 0 = Offline, 1 = Online, 2 = In Game, 3 = In Studio
    let status = match presence_data.user_presence_type {
        0 => "offline",
        1 => "online",
        2 => "ingame",
        3 => "studio",
        _ => "unknown",
    };

    Ok(UserPresence {
        status: status.to_string(),
        last_location: presence_data.last_location.clone(),
        place_id: presence_data.place_id,
    })
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

/// Validate a cookie (returns user data or error)
#[tauri::command]
pub async fn validate_cookie(cookie: String) -> Result<RobloxUserData, String> {
    validate_and_get_user(&cookie).await
}

/// Refresh account data (re-fetch from Roblox API)
#[tauri::command]
pub async fn refresh_account_data(cookie: String) -> Result<RobloxUserData, String> {
    validate_and_get_user(&cookie).await
}

/// Get user presence status (Online/Offline/InGame/Studio)
#[tauri::command]
pub async fn get_user_presence(user_id: i64) -> Result<UserPresence, String> {
    fetch_user_presence(user_id).await
}

// ============================================================================
// JOIN USER API - For joining friends in-game
// ============================================================================

const ROBLOX_USERNAMES_API: &str = "https://users.roblox.com/v1/usernames/users";

/// Response from username lookup API
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UsernamesResponse {
    data: Vec<UsernamesData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UsernamesData {
    #[serde(rename = "requestedUsername")]
    #[allow(dead_code)]
    requested_username: Option<String>,
    #[serde(rename = "hasVerifiedBadge")]
    #[allow(dead_code)]
    has_verified_badge: bool,
    id: i64,
    name: String,
    #[serde(rename = "displayName")]
    display_name: String,
}

/// User game info for joining
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGameInfo {
    pub user_id: i64,
    pub username: String,
    pub is_in_game: bool,
    pub place_id: Option<i64>,
    pub game_id: Option<String>,
    pub game_name: Option<String>,
}

/// Lookup user by username
#[tauri::command]
pub async fn get_user_by_username(username: String) -> Result<RobloxUserData, String> {
    let client = reqwest::Client::new();
    
    let body = serde_json::json!({
        "usernames": [username],
        "excludeBannedUsers": true
    });
    
    let response = client
        .post(ROBLOX_USERNAMES_API)
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }
    
    let data: UsernamesResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;
    
    let user = data.data.first()
        .ok_or_else(|| format!("User '{}' not found", username))?;
    
    // Get thumbnail
    let thumbnail_client = reqwest::Client::new();
    let thumbnail = get_user_thumbnail(&thumbnail_client, user.id).await.ok();
    
    Ok(RobloxUserData {
        id: user.id,
        name: user.name.clone(),
        display_name: user.display_name.clone(),
        thumbnail,
    })
}

/// Get user's current game info (for joining)
#[tauri::command]
pub async fn get_user_game_info(cookie: String, target_user_id: i64, target_username: String) -> Result<UserGameInfo, String> {
    let client = reqwest::Client::new();
    
    // Prepare cookie header for authenticated presence request
    let cookie_value = if cookie.starts_with(".ROBLOSECURITY=") {
        cookie.clone()
    } else {
        format!(".ROBLOSECURITY={}", cookie)
    };
    
    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        HeaderValue::from_str(&cookie_value).map_err(|e| e.to_string())?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    
    let body = serde_json::json!({
        "userIds": [target_user_id]
    });
    
    let response = client
        .post(ROBLOX_PRESENCE_API)
        .headers(headers)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Presence error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Presence API error: {}", response.status()));
    }
    
    let presence_response: PresenceResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;
    
    let presence = presence_response
        .user_presences
        .first()
        .ok_or_else(|| "No presence data".to_string())?;
    
    // Check if user is in game (presence type 2)
    let is_in_game = presence.user_presence_type == 2;
    
    Ok(UserGameInfo {
        user_id: target_user_id,
        username: target_username,
        is_in_game,
        place_id: presence.place_id,
        game_id: presence.game_id.clone(),
        game_name: presence.last_location.clone(),
    })
}

// ============================================================================
// EXTENDED USER DETAILS API
// ============================================================================

const ROBLOX_ECONOMY_API: &str = "https://economy.roblox.com/v1/users";
const ROBLOX_GROUPS_API: &str = "https://groups.roblox.com/v1/users";
const ROBLOX_USERS_API: &str = "https://users.roblox.com/v1/users";

/// Group info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupInfo {
    #[allow(dead_code)]
    pub id: i64,
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub member_count: Option<i64>,
}

/// Group membership response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMembership {
    pub group: GroupInfo,
    #[allow(dead_code)]
    pub role: Option<GroupRole>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroupRole {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub rank: i32,
}

/// Groups response wrapper
#[derive(Debug, Clone, Deserialize)]
pub struct GroupsResponse {
    pub data: Vec<GroupMembership>,
}

/// Robux response
#[derive(Debug, Clone, Deserialize)]
pub struct RobuxResponse {
    pub robux: i64,
}

/// User profile response (includes created date)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileResponse {
    #[allow(dead_code)]
    pub id: i64,
    #[allow(dead_code)]
    pub name: String,
    pub display_name: String,
    pub created: String,
    pub is_banned: bool,
}

/// Extended user details for frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtendedUserDetails {
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub robux: Option<i64>,
    pub created_at: Option<String>,
    pub is_banned: bool,
    pub groups_count: i32,
    pub groups: Vec<GroupInfo>,
    pub profile_url: String,
    pub login_link: String,
}

/// Fetch Robux balance (requires authenticated cookie)
async fn fetch_robux(cookie: &str, user_id: i64) -> Result<i64, String> {
    let client = reqwest::Client::new();
    let cookie_value = if cookie.starts_with(".ROBLOSECURITY=") {
        cookie.to_string()
    } else {
        format!(".ROBLOSECURITY={}", cookie)
    };

    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_str(&cookie_value).map_err(|e| e.to_string())?);

    let url = format!("{}/{}/currency", ROBLOX_ECONOMY_API, user_id);
    let response = client.get(&url).headers(headers).send().await.map_err(|e| e.to_string())?;
    
    if !response.status().is_success() {
        return Err("Could not fetch Robux".to_string());
    }

    let data: RobuxResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(data.robux)
}

/// Fetch user groups
async fn fetch_groups(user_id: i64) -> Result<Vec<GroupInfo>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/{}/groups/roles", ROBLOX_GROUPS_API, user_id);
    
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    
    if !response.status().is_success() {
        return Ok(vec![]); // Return empty on error
    }

    let data: GroupsResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(data.data.into_iter().map(|m| m.group).collect())
}

/// Fetch user profile (includes created date)
async fn fetch_user_profile(user_id: i64) -> Result<UserProfileResponse, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/{}", ROBLOX_USERS_API, user_id);
    
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    
    if !response.status().is_success() {
        return Err("User not found".to_string());
    }

    response.json().await.map_err(|e| e.to_string())
}

/// Get extended user details (Robux, Groups, Created Date, Links)
#[tauri::command]
pub async fn get_user_details(cookie: String, user_id: i64, username: String) -> Result<ExtendedUserDetails, String> {
    // Fetch all data concurrently
    let (robux_result, groups_result, profile_result) = tokio::join!(
        fetch_robux(&cookie, user_id),
        fetch_groups(user_id),
        fetch_user_profile(user_id)
    );

    let robux = robux_result.ok();
    let groups = groups_result.unwrap_or_default();
    let profile = profile_result.ok();

    let created_at = profile.as_ref().map(|p| p.created.clone());
    let is_banned = profile.as_ref().map(|p| p.is_banned).unwrap_or(false);
    let display_name = profile.as_ref().map(|p| p.display_name.clone()).unwrap_or_default();

    Ok(ExtendedUserDetails {
        user_id,
        username: username.clone(),
        display_name,
        robux,
        created_at,
        is_banned,
        groups_count: groups.len() as i32,
        groups,
        profile_url: format!("https://www.roblox.com/users/{}/profile", user_id),
        login_link: format!("https://www.roblox.com/home?userId={}", user_id),
    })
}

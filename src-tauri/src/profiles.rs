//! ROKIO Profiles Module
//! Profile data structure for Roblox accounts.

use serde::{Deserialize, Serialize};

/// A Roblox account profile
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// Decrypted .ROBLOSECURITY cookie
    pub cookie: String,
    /// Roblox user ID
    pub user_id: i64,
    /// Roblox username
    pub username: String,
    /// Roblox display name
    pub display_name: String,
    /// Avatar thumbnail URL
    pub thumbnail: Option<String>,
    /// User-defined alias (custom name)
    #[serde(default)]
    pub alias: String,
    /// User-defined description/notes
    #[serde(default)]
    pub description: String,
    /// Is favorite account
    pub is_favorite: bool,
    /// Last played timestamp (Unix epoch)
    pub last_played_at: u64,
    /// Optional password for the account
    #[serde(default)]
    pub password: Option<String>,
    /// Account creation timestamp (Unix epoch)
    #[serde(default)]
    pub created_at: Option<u64>,
    /// Premium membership status
    #[serde(default)]
    pub is_premium: Option<bool>,
}

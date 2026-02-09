//! ROKIO Client Module
//! Handles launching Roblox with specific account profiles.

use serde::Serialize;

/// State of a launched client
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchState {
    pub profile_id: String,
    pub pid: u32,
    pub running: bool,
}

// TODO: Phase 3 implementation
// - launch_profile
// - stop_client
// - get_running_clients

//! Game Detection - Get game info from Roblox API

use serde::{Deserialize, Serialize};

/// Game info returned to frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub universe_id: u64,
    pub place_id: u64,
    pub name: String,
    pub description: String,
    pub creator_name: String,
    pub playing: u64,
    pub visits: u64,
    pub thumbnail: Option<String>,
}

/// Roblox API response for game details
#[derive(Debug, Deserialize)]
struct GameDetailsResponse {
    data: Vec<GameDetail>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameDetail {
    #[allow(dead_code)]
    id: u64,
    root_place_id: u64,
    name: String,
    description: Option<String>,
    creator: GameCreator,
    playing: u64,
    visits: u64,
}

#[derive(Debug, Deserialize)]
struct GameCreator {
    name: String,
}

/// Get universe ID from place ID
#[tauri::command]
pub async fn get_universe_id(place_id: u64) -> Result<u64, String> {
    let url = format!(
        "https://apis.roblox.com/universes/v1/places/{}/universe",
        place_id
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch universe: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct UniverseResponse {
        universe_id: u64,
    }

    let data: UniverseResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(data.universe_id)
}

/// Get game info from universe ID
#[tauri::command]
pub async fn get_game_info(universe_id: u64) -> Result<GameInfo, String> {
    let url = format!(
        "https://games.roblox.com/v1/games?universeIds={}",
        universe_id
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch game: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let data: GameDetailsResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    let game = data
        .data
        .into_iter()
        .next()
        .ok_or("Game not found")?;

    // Get thumbnail
    let thumbnail = get_game_thumbnail(universe_id).await.ok();

    Ok(GameInfo {
        universe_id,
        place_id: game.root_place_id,
        name: game.name,
        description: game.description.unwrap_or_default(),
        creator_name: game.creator.name,
        playing: game.playing,
        visits: game.visits,
        thumbnail,
    })
}

/// Get game thumbnail URL
async fn get_game_thumbnail(universe_id: u64) -> Result<String, String> {
    let url = format!(
        "https://thumbnails.roblox.com/v1/games/icons?universeIds={}&size=150x150&format=Png",
        universe_id
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?;

    #[derive(Deserialize)]
    struct ThumbnailResponse {
        data: Vec<ThumbnailData>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ThumbnailData {
        image_url: Option<String>,
    }

    let data: ThumbnailResponse = response.json().await.map_err(|e| e.to_string())?;

    data.data
        .into_iter()
        .next()
        .and_then(|d| d.image_url)
        .ok_or("No thumbnail".to_string())
}

/// Get game icons for multiple universe IDs in batch
#[tauri::command]
pub async fn batch_get_game_icons(universe_ids: Vec<u64>) -> Result<std::collections::HashMap<u64, String>, String> {
    println!("Fetching icons for {} games...", universe_ids.len());
    let ids_str = universe_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
    let url = format!(
        "https://thumbnails.roblox.com/v1/games/icons?universeIds={}&size=420x420&format=Png",
        ids_str
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    #[derive(serde::Deserialize)]
    struct ThumbnailResponse {
        data: Vec<ThumbnailData>,
    }

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ThumbnailData {
        target_id: u64,
        image_url: Option<String>,
    }

    let data: ThumbnailResponse = response.json().await.map_err(|e| e.to_string())?;
    
    let mut result = std::collections::HashMap::new();
    for item in data.data {
        if let Some(url) = item.image_url {
            result.insert(item.target_id, url);
        }
    }
    
    Ok(result)
}

/// Server info returned from Roblox API
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub id: String,
    pub playing: u32,
    pub max_players: u32,
    pub ping: Option<u32>,
    pub fps: Option<f32>,
}

/// Get public servers for a game
#[tauri::command]
pub async fn get_game_servers(place_id: u64, cursor: Option<String>) -> Result<(Vec<ServerInfo>, Option<String>), String> {
    let url = if let Some(c) = cursor {
        format!(
            "https://games.roblox.com/v1/games/{}/servers/Public?sortOrder=Asc&excludeFullGames=false&limit=100&cursor={}",
            place_id, c
        )
    } else {
        format!(
            "https://games.roblox.com/v1/games/{}/servers/Public?sortOrder=Asc&excludeFullGames=false&limit=100",
            place_id
        )
    };

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch servers: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ServersResponse {
        data: Vec<ServerData>,
        next_page_cursor: Option<String>,
    }

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ServerData {
        id: String,
        playing: u32,
        max_players: u32,
        ping: Option<u32>,
        fps: Option<f32>,
    }

    let data: ServersResponse = response.json().await.map_err(|e| format!("Parse error: {}", e))?;

    let servers = data.data.into_iter().map(|s| ServerInfo {
        id: s.id,
        playing: s.playing,
        max_players: s.max_players,
        ping: s.ping,
        fps: s.fps,
    }).collect();

    Ok((servers, data.next_page_cursor))
}

/// Popular game info
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PopularGame {
    pub universe_id: u64,
    pub place_id: u64,
    pub name: String,
    pub player_count: u64,
    pub thumbnail: Option<String>,
}

/// Get popular games with thumbnails
#[tauri::command]
pub async fn get_popular_games() -> Result<Vec<PopularGame>, String> {
    println!("get_popular_games called");
    // Use curated popular games list (Roblox's games list API is complex and undocumented)
    let games = vec![
        // Roleplay & Life Sim
        (2788229376, 8761176980, "Brookhaven 🏡RP", 500000),
        (65241, 286090429, "Adopt Me!", 400000),
        (13058, 150387031, "Bloxburg", 75000),
        (2897587123, 9295537553, "Livetopia", 35000),
        (4577884168, 13772394625, "Berry Avenue", 80000),
        
        // Action & Adventure
        (3318029993, 2753915549, "Blox Fruits", 550000),
        (1224212277, 3260590327, "Tower of Hell", 60000),
        (220552094, 606849621, "Jailbreak", 40000),
        (189707, 292439477, "Murder Mystery 2", 180000),
        (2534724415, 6516141723, "DOORS 👁️", 50000),
        (4717143960, 15561560493, "Blade Ball", 120000),
        (1973634035, 6872265039, "BedWars", 85000),
        (2377868063, 8560631822, "Anime Adventures", 40000),
        (1537156947, 537413528, "Build A Boat", 35000),

        // Simulators
        (746820961, 8737602449, "Pet Simulator 99", 150000),
        (1606394348, 5952327072, "Strongman Sim", 25000),
        (63830488, 286090429, "Arsenal", 15000),
        (4252370517, 14704022432, "Dress To Impress", 250000),
        (5167733276, 17265328292, "The Strongest Battlegrounds", 90000),
        
        // Other Popular
        (1165192137, 2753915549, "Royale High", 45000),
        (245662005, 6516141723, "MeepCity", 30000),
        (1720207389, 4872321990, "World // Zero", 10000),
        (3322223393, 9872472334, "Fruit Battlegrounds", 20000),
        (286419782, 1071062235, "Lumber Tycoon 2", 15000),
    ];

    // Get thumbnails for all games
    // BLOCKED: User requested to remove image fetch causing hangs
    // let universe_ids: Vec<u64> = games.iter().map(|(u, _, _, _)| *u).collect();
    // let icons = batch_get_game_icons(universe_ids).await.unwrap_or_default();

    let result = games.into_iter().map(|(uid, pid, name, count)| PopularGame {
        thumbnail: None, // icons.get(&uid).cloned(),
        universe_id: uid,
        place_id: pid,
        name: name.to_string(),
        player_count: count,
    }).collect();

    Ok(result)
}

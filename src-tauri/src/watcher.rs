//! ROKIO Watcher Module
//! Background process monitoring for active Roblox instances.

use crate::launcher::{ActiveInstance, LauncherState};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Event payload for instance closed
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct InstanceClosedEvent {
    pub pid: u32,
    pub account_id: String,
    pub username: String,
}

/// Start the background process watcher
#[allow(dead_code)]
pub fn start_watcher(app_handle: AppHandle, launcher_state: Arc<LauncherState>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();

        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(2)).await;

                let dead_instances: Vec<ActiveInstance> = {
                    let instances = launcher_state.instances.lock().unwrap();
                    instances
                        .iter()
                        .filter(|(pid, _)| !crate::process_utils::is_process_running(**pid))
                        .map(|(_, inst)| inst.clone())
                        .collect()
                };

                // Remove dead instances and emit events
                for instance in dead_instances {
                    launcher_state
                        .instances
                        .lock()
                        .unwrap()
                        .remove(&instance.pid);

                    // Emit event to frontend
                    let event = InstanceClosedEvent {
                        pid: instance.pid,
                        account_id: instance.account_id.clone(),
                        username: instance.username.clone(),
                    };

                    if let Err(e) = app_handle.emit("instance-closed", event) {
                        eprintln!("Failed to emit instance-closed event: {}", e);
                    }
                }
            }
        });
    });
}

/// Initialize the watcher on app startup
#[allow(dead_code)]
pub fn init_watcher(app: &tauri::App) {
    let _app_handle = app.handle().clone();
    let _launcher_state: tauri::State<'_, LauncherState> = app.state();
    
    // Note: In a real implementation, we'd share the actual state
    // For now, we rely on the Tauri State management
    
    println!("[ROKIO] Process watcher initialized");
}

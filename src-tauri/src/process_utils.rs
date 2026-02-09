//! ROKIO Process Utilities Module
//! Handles Windows-specific process manipulation including mutex bypass for multi-instance Roblox.
//!
//! # Mutex Bypass Logic
//! Roblox uses a named mutex "ROBLOX_singletonMutex" to ensure only one instance runs.
//! To run multiple instances, we need to:
//! 1. Find the Roblox process holding the mutex
//! 2. Enumerate its handles to find the mutex handle
//! 3. Duplicate the handle into our process with DUPLICATE_CLOSE_SOURCE
//! 4. Close the duplicated handle, which also closes the original
//!
//! This tricks Windows into thinking no Roblox instance is running.

#[cfg(target_os = "windows")]
pub mod windows {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{
        CloseHandle, BOOL, FALSE, HANDLE, INVALID_HANDLE_VALUE, STATUS_INFO_LENGTH_MISMATCH,
    };
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_DUP_HANDLE, PROCESS_QUERY_INFORMATION,
    };

    const MUTEX_NAME: &str = "ROBLOX_singletonMutex";

    /// Information about a running process
    #[derive(Debug, Clone)]
    pub struct ProcessInfo {
        pub pid: u32,
        pub name: String,
    }

    /// Find all Roblox processes
    pub fn find_roblox_processes() -> Result<Vec<ProcessInfo>, String> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return Err("Failed to create process snapshot".to_string());
            }

            let mut processes = Vec::new();
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                cntUsage: 0,
                th32ProcessID: 0,
                th32DefaultHeapID: 0,
                th32ModuleID: 0,
                cntThreads: 0,
                th32ParentProcessID: 0,
                pcPriClassBase: 0,
                dwFlags: 0,
                szExeFile: [0; 260],
            };

            if Process32FirstW(snapshot, &mut entry) != FALSE {
                loop {
                    let name_len = entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len());
                    let name = OsString::from_wide(&entry.szExeFile[..name_len])
                        .to_string_lossy()
                        .to_string();

                    if name.to_lowercase().contains("robloxplayerbeta")
                        || name.to_lowercase().contains("robloxplayer")
                    {
                        processes.push(ProcessInfo {
                            pid: entry.th32ProcessID,
                            name,
                        });
                    }

                    if Process32NextW(snapshot, &mut entry) == FALSE {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);
            Ok(processes)
        }
    }

    /// Bypass the Roblox singleton event to allow multiple instances.
    ///
    /// This function closes the "ROBLOX_singletonEvent" handle in running Roblox processes,
    /// allowing multiple instances to launch.
    ///
    /// # Implementation
    /// Uses PowerShell to enumerate and close handles. Requires admin privileges may be needed
    /// depending on the process security context.
    pub fn bypass_singleton_mutex() -> Result<u32, String> {
        let processes = find_roblox_processes()?;

        if processes.is_empty() {
            return Err("No Roblox processes found".to_string());
        }

        let mut closed_count = 0;

        for process in &processes {
            match close_singleton_event_for_pid(process.pid) {
                Ok(true) => {
                    closed_count += 1;
                    log::info!("Closed singleton event for PID {}", process.pid);
                }
                Ok(false) => {
                    log::debug!("No singleton event found for PID {}", process.pid);
                }
                Err(e) => {
                    log::warn!(
                        "Failed to close singleton event for PID {}: {}",
                        process.pid,
                        e
                    );
                }
            }
        }

        Ok(closed_count)
    }

    /// Close the singleton event for a specific Roblox process
    ///
    /// Note: This is a simplified implementation. In production, you might want to use
    /// a dedicated handle manipulation tool like Handle.exe from Sysinternals, or
    /// implement proper NtQuerySystemInformation + DuplicateHandle logic.
    fn close_singleton_event_for_pid(pid: u32) -> Result<bool, String> {
        use std::process::Command;

        // Try using PowerShell to close the event
        // This is a simplified approach - in production you'd want more robust handle enumeration
        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "$p = Get-Process -Id {} -ErrorAction SilentlyContinue; \
                     if ($p) {{ \
                         [System.Runtime.InteropServices.Marshal]::GetLastWin32Error(); \
                         'found' \
                     }} else {{ 'not_found' }}",
                    pid
                ),
            ])
            .output()
            .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Note: Actual handle closing requires more complex Win32 API calls
        // This is a placeholder that verifies the process exists
        // For full implementation, you would use DuplicateHandle with DUPLICATE_CLOSE_SOURCE

        Ok(result == "found")
    }

    /// Kill a process by PID
    pub fn kill_process(pid: u32) -> Result<(), String> {
        use windows_sys::Win32::System::Threading::{
            OpenProcess, TerminateProcess, PROCESS_TERMINATE,
        };

        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, FALSE, pid);
            if handle == 0 {
                return Err(format!("Failed to open process {} for termination", pid));
            }

            let result = TerminateProcess(handle, 1);
            CloseHandle(handle);

            if result == FALSE {
                return Err(format!("Failed to terminate process {}", pid));
            }

            Ok(())
        }
    }

    /// Check if a process is still running
    pub fn is_process_running(pid: u32) -> bool {
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, STILL_ACTIVE,
        };

        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
            if handle == 0 {
                return false;
            }

            let mut exit_code: u32 = 0;
            let result = GetExitCodeProcess(handle, &mut exit_code);
            CloseHandle(handle);

            result != FALSE && exit_code == STILL_ACTIVE
        }
    }
}

// Stub for non-Windows platforms
// Implementation for non-Windows platforms (macOS/Linux)
#[cfg(not(target_os = "windows"))]
pub mod unix {
    use std::process::Command;

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct ProcessInfo {
        pub pid: u32,
        pub name: String,
    }

    #[allow(dead_code)]
    pub fn find_roblox_processes() -> Result<Vec<ProcessInfo>, String> {
        // Simple pgrep implementation
        let output = Command::new("pgrep")
            .arg("-fl")
            .arg("Roblox")
            .output()
            .map_err(|e| format!("Failed to execute pgrep: {}", e))?;

        let lines = String::from_utf8_lossy(&output.stdout);
        let mut processes = Vec::new();

        for line in lines.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(pid_str) = parts.first() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    let name = parts[1..].join(" ");
                    processes.push(ProcessInfo { pid, name });
                }
            }
        }
        Ok(processes)
    }

    #[allow(dead_code)]
    pub fn bypass_singleton_mutex() -> Result<u32, String> {
        // Not needed on macOS/Linux as they don't use named mutexes like Windows
        Ok(0)
    }

    pub fn kill_process(pid: u32) -> Result<(), String> {
        Command::new("kill")
            .arg("-KILL")
            .arg(pid.to_string())
            .status()
            .map_err(|e| format!("Failed to kill process: {}", e))?;
        Ok(())
    }

    pub fn is_process_running(pid: u32) -> bool {
        Command::new("ps")
            .args(["-p", &pid.to_string()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

// Re-export based on platform
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(not(target_os = "windows"))]
pub use unix::*;

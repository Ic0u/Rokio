use std::time::SystemTime;

/// Convert Unix timestamp to Cocoa timestamp (macOS uses different epoch)
pub fn to_cocoa_timestamp(time: SystemTime) -> f64 {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => (duration.as_secs() as i64 - 978307200) as f64,
        Err(_) => 0.0,
    }
}

#[allow(dead_code)]
pub fn from_cocoa_timestamp(timestamp: f64) -> SystemTime {
    use std::time::Duration;
    SystemTime::UNIX_EPOCH + Duration::from_secs((timestamp as u64) + 978307200)
}

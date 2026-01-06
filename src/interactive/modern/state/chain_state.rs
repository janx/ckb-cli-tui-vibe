use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainState {
    pub height: u64,
    pub epoch: u64,
    pub peers: usize,
    pub sync_progress: f64,
    pub last_block_time: u64,
    pub is_syncing: bool,
    pub last_updated: u64,
}

impl Default for ChainState {
    fn default() -> Self {
        Self {
            height: 0,
            epoch: 0,
            peers: 0,
            sync_progress: 0.0,
            last_block_time: 0,
            is_syncing: false,
            last_updated: current_timestamp(),
        }
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

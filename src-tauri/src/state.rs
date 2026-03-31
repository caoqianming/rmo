use crate::config::Config;
use crate::metrics::{MetricsCollector, MetricsSnapshot};
use std::sync::Mutex;

pub struct AppState {
    pub config: Mutex<Config>,
    pub collector: Mutex<MetricsCollector>,
    pub latest_snapshot: Mutex<MetricsSnapshot>,
    pub auto_positioning: Mutex<bool>,
}

impl AppState {
    pub fn new(config: Config, collector: MetricsCollector, latest_snapshot: MetricsSnapshot) -> Self {
        Self {
            config: Mutex::new(config),
            collector: Mutex::new(collector),
            latest_snapshot: Mutex::new(latest_snapshot),
            auto_positioning: Mutex::new(true),
        }
    }
}

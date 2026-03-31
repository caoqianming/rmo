use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_refresh")]
    pub refresh_interval_secs: f64,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default = "default_true")]
    pub show_cpu: bool,
    #[serde(default = "default_true")]
    pub show_memory: bool,
    #[serde(default)]
    pub show_network: bool,
    #[serde(default)]
    pub show_disk_io: bool,
}

fn default_version() -> u32 {
    1
}

fn default_refresh() -> f64 {
    2.0
}

fn default_opacity() -> f64 {
    0.85
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: default_version(),
            refresh_interval_secs: default_refresh(),
            opacity: default_opacity(),
            show_cpu: true,
            show_memory: true,
            show_network: false,
            show_disk_io: false,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let mut path = dirs_next::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("rmo");
        path.push("config.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match std::fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<Config>(&text) {
                Ok(mut config) => {
                    config.clamp();
                    if !config.any_metric_enabled() {
                        config = Config::default();
                        config.save();
                    }
                    config
                }
                Err(error) => {
                    eprintln!("rmo: failed to parse config: {error}");
                    let config = Config::default();
                    config.save();
                    config
                }
            },
            Err(_) => {
                let config = Config::default();
                config.save();
                config
            }
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        match serde_json::to_string_pretty(self) {
            Ok(text) => {
                if let Err(error) = std::fs::write(path, text) {
                    eprintln!("rmo: failed to write config: {error}");
                }
            }
            Err(error) => eprintln!("rmo: failed to serialize config: {error}"),
        }
    }

    pub fn clamp(&mut self) {
        self.opacity = self.opacity.clamp(0.3, 1.0);
        self.refresh_interval_secs = self.refresh_interval_secs.clamp(1.0, 10.0);
    }

    pub fn any_metric_enabled(&self) -> bool {
        self.show_cpu || self.show_memory || self.show_network || self.show_disk_io
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn default_config_matches_expected_defaults() {
        let config = Config::default();

        assert_eq!(config.version, 1);
        assert_eq!(config.refresh_interval_secs, 2.0);
        assert_eq!(config.opacity, 0.85);
        assert!(config.show_cpu);
        assert!(config.show_memory);
        assert!(!config.show_network);
        assert!(!config.show_disk_io);
    }

    #[test]
    fn clamp_limits_refresh_and_opacity_ranges() {
        let mut config = Config {
            refresh_interval_secs: 999.0,
            opacity: 0.1,
            ..Config::default()
        };

        config.clamp();

        assert_eq!(config.refresh_interval_secs, 10.0);
        assert_eq!(config.opacity, 0.3);
    }

    #[test]
    fn active_metric_count_counts_enabled_flags() {
        let config = Config {
            show_network: true,
            ..Config::default()
        };

        let active_metric_count = [
            config.show_cpu,
            config.show_memory,
            config.show_network,
            config.show_disk_io,
        ]
        .into_iter()
        .filter(|enabled| *enabled)
        .count();

        assert_eq!(active_metric_count, 3);
        assert!(config.any_metric_enabled());
    }

    #[test]
    fn deserialize_ignores_unknown_fields() {
        let text = r#"{
            "version": 1,
            "refresh_interval_secs": 2.5,
            "opacity": 0.9,
            "show_cpu": true,
            "show_memory": true,
            "show_network": false,
            "show_disk_io": false,
            "future_field": "ignored"
        }"#;

        let config: Config = serde_json::from_str(text).expect("config should deserialize");

        assert_eq!(config.refresh_interval_secs, 2.5);
        assert_eq!(config.opacity, 0.9);
    }

    #[test]
    fn round_trip_preserves_fields() {
        let original = Config {
            refresh_interval_secs: 4.0,
            opacity: 0.6,
            show_network: true,
            ..Config::default()
        };

        let serialized = serde_json::to_string(&original).expect("config should serialize");
        let restored: Config =
            serde_json::from_str(&serialized).expect("config should deserialize");

        assert_eq!(restored, original);
    }
}

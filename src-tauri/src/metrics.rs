use serde::Serialize;
use std::time::Instant;
use sysinfo::{DiskRefreshKind, Disks, Networks, System};

#[derive(Debug, Clone, Serialize, Default)]
pub struct MetricsSnapshot {
    pub cpu_pct: f32,
    pub mem_pct: f32,
    pub net_up_bps: f64,
    pub net_down_bps: f64,
    pub disk_read_bps: f64,
    pub disk_write_bps: f64,
}

pub struct MetricsCollector {
    system: System,
    networks: Networks,
    disks: Disks,
    last_refresh: Instant,
    first_sample: bool,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_usage();
        system.refresh_memory();

        Self {
            system,
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list_specifics(
                DiskRefreshKind::nothing().with_io_usage(),
            ),
            last_refresh: Instant::now(),
            first_sample: true,
        }
    }

    pub fn refresh(&mut self, collect_network: bool, collect_disk: bool) -> MetricsSnapshot {
        let elapsed = self.last_refresh.elapsed().as_secs_f64().max(0.001);
        self.last_refresh = Instant::now();

        self.system.refresh_cpu_usage();
        self.system.refresh_memory();

        let cpu_pct = self.system.global_cpu_usage();
        let mem_pct = if self.system.total_memory() > 0 {
            (self.system.used_memory() as f32 / self.system.total_memory() as f32) * 100.0
        } else {
            0.0
        };

        let (net_up_bps, net_down_bps) = if collect_network {
            self.networks.refresh(false);
            let rx_bytes: u64 = self
                .networks
                .iter()
                .map(|(_, network)| network.received())
                .sum();
            let tx_bytes: u64 = self
                .networks
                .iter()
                .map(|(_, network)| network.transmitted())
                .sum();

            if self.first_sample {
                (0.0, 0.0)
            } else {
                (tx_bytes as f64 / elapsed, rx_bytes as f64 / elapsed)
            }
        } else {
            (0.0, 0.0)
        };

        let (disk_read_bps, disk_write_bps) = if collect_disk {
            self.disks
                .refresh_specifics(false, DiskRefreshKind::nothing().with_io_usage());
            let read_bytes: u64 = self.disks.iter().map(|disk| disk.usage().read_bytes).sum();
            let write_bytes: u64 = self
                .disks
                .iter()
                .map(|disk| disk.usage().written_bytes)
                .sum();

            if self.first_sample {
                (0.0, 0.0)
            } else {
                (read_bytes as f64 / elapsed, write_bytes as f64 / elapsed)
            }
        } else {
            (0.0, 0.0)
        };

        self.first_sample = false;

        MetricsSnapshot {
            cpu_pct,
            mem_pct,
            net_up_bps,
            net_down_bps,
            disk_read_bps,
            disk_write_bps,
        }
    }
}

#[cfg(test)]
mod tests {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum MetricColor {
        Green,
        Yellow,
        Red,
    }

    fn threshold_color(value: f32) -> MetricColor {
        if value < 60.0 {
            MetricColor::Green
        } else if value < 85.0 {
            MetricColor::Yellow
        } else {
            MetricColor::Red
        }
    }

    fn format_speed(bytes_per_sec: f64) -> String {
        let kilobytes_per_sec = bytes_per_sec / 1024.0;
        if kilobytes_per_sec < 1024.0 {
            format!("{kilobytes_per_sec:.1} KB/s")
        } else {
            format!("{:.1} MB/s", kilobytes_per_sec / 1024.0)
        }
    }

    #[test]
    fn threshold_color_uses_expected_bands() {
        assert_eq!(threshold_color(12.0), MetricColor::Green);
        assert_eq!(threshold_color(60.0), MetricColor::Yellow);
        assert_eq!(threshold_color(85.0), MetricColor::Red);
    }

    #[test]
    fn format_speed_uses_kilobytes_below_one_megabyte() {
        assert_eq!(format_speed(512.0 * 1024.0), "512.0 KB/s");
    }

    #[test]
    fn format_speed_uses_megabytes_at_or_above_one_megabyte() {
        assert_eq!(format_speed(2.0 * 1024.0 * 1024.0), "2.0 MB/s");
    }

    #[test]
    fn format_speed_handles_zero() {
        assert_eq!(format_speed(0.0), "0.0 KB/s");
    }
}

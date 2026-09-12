use super::{GpuDeviceStats, HardwareCollector, HostStats};
use crate::telemetry::host::HostCollector;

pub struct MockCollector {
    host: HostCollector,
    tick: u64,
}

impl MockCollector {
    pub fn new() -> Self {
        Self {
            host: HostCollector::new(),
            tick: 0,
        }
    }
}

impl HardwareCollector for MockCollector {
    fn poll_host(&mut self) -> HostStats {
        self.host.poll()
    }

    fn poll_gpus(&mut self) -> Vec<GpuDeviceStats> {
        self.tick += 1;
        vec![
            GpuDeviceStats {
                index: 0,
                name: "NVIDIA GeForce RTX 3080".to_string(),
                vram_used: 8_589_934_592, // ~8.0 GB
                vram_total: 10_737_418_240, // 10 GB
                gpu_util: ((self.tick * 6) % 100) as u32,
                mem_util: 78,
                temp_c: 65,
                power_watts: 280.0,
                power_limit_watts: 320.0,
            },
            GpuDeviceStats {
                index: 1,
                name: "NVIDIA GeForce RTX 4060".to_string(),
                vram_used: 4_294_967_296, // ~4.0 GB
                vram_total: 8_589_934_592, // 8 GB
                gpu_util: ((self.tick * 9) % 100) as u32,
                mem_util: 42,
                temp_c: 54,
                power_watts: 90.0,
                power_limit_watts: 115.0,
            },
        ]
    }
}
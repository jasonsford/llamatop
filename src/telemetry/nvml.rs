use super::{GpuDeviceStats, HardwareCollector, HostStats};
use crate::telemetry::host::HostCollector;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::Nvml;

pub struct NvmlCollector {
    host: HostCollector,
    nvml: Nvml,
}

impl NvmlCollector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let nvml = Nvml::init()?;
        Ok(Self {
            host: HostCollector::new(),
            nvml,
        })
    }
}

impl HardwareCollector for NvmlCollector {
    fn poll_host(&mut self) -> HostStats {
        self.host.poll()
    }

    fn poll_gpus(&mut self) -> Vec<GpuDeviceStats> {
        let count = match self.nvml.device_count() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut stats = Vec::with_capacity(count as usize);

        for i in 0..count {
            if let Ok(dev) = self.nvml.device_by_index(i) {
                let name = dev.name().unwrap_or_else(|_| format!("GPU {}", i));
                let mem = dev.memory_info().ok();
                let util = dev.utilization_rates().ok();
                let temp = dev.temperature(TemperatureSensor::Gpu).unwrap_or(0);
                let power = dev.power_usage().unwrap_or(0) as f32 / 1000.0;
                let power_limit = dev.enforced_power_limit().unwrap_or(0) as f32 / 1000.0;

                stats.push(GpuDeviceStats {
                    index: i,
                    name,
                    vram_used: mem.as_ref().map(|m| m.used).unwrap_or(0),
                    vram_total: mem.as_ref().map(|m| m.total).unwrap_or(0),
                    gpu_util: util.as_ref().map(|u| u.gpu).unwrap_or(0),
                    mem_util: util.as_ref().map(|u| u.memory).unwrap_or(0),
                    temp_c: temp,
                    power_watts: power,
                    power_limit_watts: power_limit,
                });
            }
        }

        stats
    }
}
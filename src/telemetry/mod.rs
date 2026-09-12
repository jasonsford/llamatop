pub mod host;

#[cfg(target_os = "linux")]
pub mod nvml;

#[cfg(not(target_os = "linux"))]
pub mod mock;

pub use host::HostStats;

#[derive(Debug, Clone, Default)]
pub struct GpuDeviceStats {
    pub index: u32,
    pub name: String,
    pub vram_used: u64,
    pub vram_total: u64,
    pub gpu_util: u32,
    pub mem_util: u32,
    pub temp_c: u32,
    pub power_watts: f32,
    pub power_limit_watts: f32,
}

pub trait HardwareCollector: Send + Sync {
    fn poll_host(&mut self) -> HostStats;
    fn poll_gpus(&mut self) -> Vec<GpuDeviceStats>;
}

pub fn create_collector() -> Box<dyn HardwareCollector> {
    #[cfg(target_os = "linux")]
    {
        Box::new(nvml::NvmlCollector::new().expect("Failed to initialize NVML"))
    }
    #[cfg(not(target_os = "linux"))]
    {
        Box::new(mock::MockCollector::new())
    }
}
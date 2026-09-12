use std::collections::VecDeque;

pub mod host;
#[cfg(target_os = "linux")]
pub mod nvml;

pub use host::HostStats;

#[derive(Debug, Clone)]
pub struct GpuProcessInfo {
    pub pid: u32,
    pub used_vram: u64,
}

#[derive(Debug, Clone)]
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
    pub pcie_tx_mb_s: f32,
    pub pcie_rx_mb_s: f32,
    pub compute_history: VecDeque<u64>,
    pub memory_history: VecDeque<u64>,
    pub running_processes: Vec<GpuProcessInfo>,
}

pub trait HardwareCollector {
    fn poll_host(&mut self) -> HostStats;
    fn poll_gpus(&mut self) -> Vec<GpuDeviceStats>;
}

#[cfg(target_os = "linux")]
pub fn create_collector() -> Box<dyn HardwareCollector> {
    match nvml::NvmlCollector::new() {
        Ok(c) => Box::new(c),
        Err(e) => {
            eprintln!("NVML init failed ({}), falling back to mock.", e);
            Box::new(MockCollector::new())
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn create_collector() -> Box<dyn HardwareCollector> {
    Box::new(MockCollector::new())
}

pub struct MockCollector {
    host: host::HostCollector,
    mock_compute_hist: [VecDeque<u64>; 2],
    mock_mem_hist: [VecDeque<u64>; 2],
    step: u64,
}

impl MockCollector {
    pub fn new() -> Self {
        Self {
            host: host::HostCollector::new(),
            mock_compute_hist: [VecDeque::with_capacity(30), VecDeque::with_capacity(30)],
            mock_mem_hist: [VecDeque::with_capacity(30), VecDeque::with_capacity(30)],
            step: 0,
        }
    }
}

impl HardwareCollector for MockCollector {
    fn poll_host(&mut self) -> HostStats {
        self.host.poll()
    }

    fn poll_gpus(&mut self) -> Vec<GpuDeviceStats> {
        self.step = self.step.wrapping_add(1);
        let compute_val = ((self.step as f64 * 0.3).sin() * 40.0 + 50.0).clamp(0.0, 100.0) as u64;
        let mem_val = ((self.step as f64 * 0.2).cos() * 30.0 + 40.0).clamp(0.0, 100.0) as u64;

        for h in &mut self.mock_compute_hist {
            if h.len() >= 30 {
                h.pop_front();
            }
            h.push_back(compute_val);
        }

        for h in &mut self.mock_mem_hist {
            if h.len() >= 30 {
                h.pop_front();
            }
            h.push_back(mem_val);
        }

        vec![
            GpuDeviceStats {
                index: 0,
                name: "RTX 3080".into(),
                vram_used: 8 * 1024 * 1024 * 1024,
                vram_total: 10 * 1024 * 1024 * 1024,
                gpu_util: compute_val as u32,
                mem_util: mem_val as u32,
                temp_c: 62,
                power_watts: 245.0,
                power_limit_watts: 320.0,
                pcie_tx_mb_s: 124.5,
                pcie_rx_mb_s: 8.2,
                compute_history: self.mock_compute_hist[0].clone(),
                memory_history: self.mock_mem_hist[0].clone(),
                running_processes: vec![GpuProcessInfo {
                    pid: 12345,
                    used_vram: 7 * 1024 * 1024 * 1024,
                }],
            },
            GpuDeviceStats {
                index: 1,
                name: "RTX 4060".into(),
                vram_used: 6 * 1024 * 1024 * 1024,
                vram_total: 8 * 1024 * 1024 * 1024,
                gpu_util: (compute_val / 2) as u32,
                mem_util: (mem_val / 2) as u32,
                temp_c: 54,
                power_watts: 85.0,
                power_limit_watts: 115.0,
                pcie_tx_mb_s: 45.1,
                pcie_rx_mb_s: 3.0,
                compute_history: self.mock_compute_hist[1].clone(),
                memory_history: self.mock_mem_hist[1].clone(),
                running_processes: vec![],
            },
        ]
    }
}

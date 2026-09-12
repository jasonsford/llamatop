use super::{GpuDeviceStats, GpuProcessInfo, HardwareCollector, HostStats};
use crate::telemetry::host::HostCollector;
use nvml_wrapper::enum_wrappers::device::{PcieUtilCounter, TemperatureSensor};
use nvml_wrapper::Nvml;
use std::collections::{HashMap, VecDeque};

pub struct NvmlCollector {
    host: HostCollector,
    nvml: Nvml,
    compute_histories: HashMap<u32, VecDeque<u64>>,
    memory_histories: HashMap<u32, VecDeque<u64>>,
}

impl NvmlCollector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let nvml = Nvml::init()?;
        Ok(Self {
            host: HostCollector::new(),
            nvml,
            compute_histories: HashMap::new(),
            memory_histories: HashMap::new(),
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

                // PCIe Throughput in MB/s (API returns KB/s)
                let tx_kb = dev.pcie_throughput(PcieUtilCounter::Send).unwrap_or(0);
                let rx_kb = dev.pcie_throughput(PcieUtilCounter::Receive).unwrap_or(0);
                let pcie_tx_mb_s = tx_kb as f32 / 1024.0;
                let pcie_rx_mb_s = rx_kb as f32 / 1024.0;

                let gpu_u = util.as_ref().map(|u| u.gpu).unwrap_or(0) as u64;
                let mem_u = util.as_ref().map(|u| u.memory).unwrap_or(0) as u64;

                // Rolling Sparkline History (last 30 samples)
                let c_hist = self
                    .compute_histories
                    .entry(i)
                    .or_insert_with(|| VecDeque::with_capacity(30));
                if c_hist.len() >= 30 {
                    c_hist.pop_front();
                }
                c_hist.push_back(gpu_u);

                let m_hist = self
                    .memory_histories
                    .entry(i)
                    .or_insert_with(|| VecDeque::with_capacity(30));
                if m_hist.len() >= 30 {
                    m_hist.pop_front();
                }
                m_hist.push_back(mem_u);

                // Compute Processes on this GPU
                let running_processes = match dev.running_compute_processes() {
                    Ok(procs) => procs
                        .into_iter()
                        .map(|p| GpuProcessInfo {
                            pid: p.pid,
                            used_vram: match p.used_gpu_memory {
                                nvml_wrapper::enums::device::UsedGpuMemory::Used(bytes) => bytes,
                                nvml_wrapper::enums::device::UsedGpuMemory::Unavailable => 0,
                            },
                        })
                        .collect(),
                    Err(_) => Vec::new(),
                };

                stats.push(GpuDeviceStats {
                    index: i,
                    name,
                    vram_used: mem.as_ref().map(|m| m.used).unwrap_or(0),
                    vram_total: mem.as_ref().map(|m| m.total).unwrap_or(0),
                    gpu_util: gpu_u as u32,
                    mem_util: mem_u as u32,
                    temp_c: temp,
                    power_watts: power,
                    power_limit_watts: power_limit,
                    pcie_tx_mb_s,
                    pcie_rx_mb_s,
                    compute_history: c_hist.clone(),
                    memory_history: m_hist.clone(),
                    running_processes,
                });
            }
        }

        stats
    }
}

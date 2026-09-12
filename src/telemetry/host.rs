use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

#[derive(Debug, Clone, Default)]
pub struct HostStats {
    pub total_memory: u64,
    pub used_memory: u64,
    pub free_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub cpu_load_percent: f32,
}

pub struct HostCollector {
    sys: System,
}

impl HostCollector {
    pub fn new() -> Self {
        let refresh = RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_cpu(CpuRefreshKind::everything());
        let sys = System::new_with_specifics(refresh);
        Self { sys }
    }

    pub fn poll(&mut self) -> HostStats {
        self.sys.refresh_memory();
        self.sys.refresh_cpu_all();

        HostStats {
            total_memory: self.sys.total_memory(),
            used_memory: self.sys.used_memory(),
            free_memory: self.sys.free_memory(),
            total_swap: self.sys.total_swap(),
            used_swap: self.sys.used_swap(),
            cpu_load_percent: self.sys.global_cpu_usage(),
        }
    }
}
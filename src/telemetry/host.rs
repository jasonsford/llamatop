use std::time::Instant;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

#[derive(Debug, Clone, Default)]
pub struct HostStats {
    pub total_memory: u64,
    pub used_memory: u64,
    pub free_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub cpu_load_percent: f32,
    pub page_in_mb_s: f64,
    pub page_out_mb_s: f64,
}

pub struct HostCollector {
    sys: System,
    last_poll: Instant,
    last_pgin: Option<u64>,
    last_pgout: Option<u64>,
}

impl HostCollector {
    pub fn new() -> Self {
        let refresh = RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_cpu(CpuRefreshKind::everything());
        let sys = System::new_with_specifics(refresh);
        Self {
            sys,
            last_poll: Instant::now(),
            last_pgin: None,
            last_pgout: None,
        }
    }

    #[cfg(target_os = "linux")]
    fn read_vmstat_paging(&mut self) -> (f64, f64) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_poll).as_secs_f64();
        self.last_poll = now;

        let content = std::fs::read_to_string("/proc/vmstat").unwrap_or_default();
        let mut cur_pgin = None;
        let mut cur_pgout = None;

        for line in content.lines() {
            if line.starts_with("pgpgin ") {
                cur_pgin = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse::<u64>().ok());
            } else if line.starts_with("pgpgout ") {
                cur_pgout = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse::<u64>().ok());
            }
        }

        let mut in_mb = 0.0;
        let mut out_mb = 0.0;

        if let (Some(c_in), Some(c_out)) = (cur_pgin, cur_pgout) {
            if let (Some(p_in), Some(p_out)) = (self.last_pgin, self.last_pgout) {
                if dt > 0.0 {
                    // /proc/vmstat pgpgin/out are in kilobytes
                    let delta_in_kb = c_in.saturating_sub(p_in) as f64;
                    let delta_out_kb = c_out.saturating_sub(p_out) as f64;
                    in_mb = (delta_in_kb / 1024.0) / dt;
                    out_mb = (delta_out_kb / 1024.0) / dt;
                }
            }
            self.last_pgin = Some(c_in);
            self.last_pgout = Some(c_out);
        }

        (in_mb, out_mb)
    }

    #[cfg(not(target_os = "linux"))]
    fn read_vmstat_paging(&mut self) -> (f64, f64) {
        (0.0, 0.0)
    }

    pub fn poll(&mut self) -> HostStats {
        self.sys.refresh_memory();
        self.sys.refresh_cpu_all();
        let (page_in_mb_s, page_out_mb_s) = self.read_vmstat_paging();

        HostStats {
            total_memory: self.sys.total_memory(),
            used_memory: self.sys.used_memory(),
            free_memory: self.sys.free_memory(),
            total_swap: self.sys.total_swap(),
            used_swap: self.sys.used_swap(),
            cpu_load_percent: self.sys.global_cpu_usage(),
            page_in_mb_s,
            page_out_mb_s,
        }
    }
}

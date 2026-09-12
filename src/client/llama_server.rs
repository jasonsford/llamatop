use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SlotInfo {
    #[serde(default)]
    pub id: u32,

    #[serde(default = "default_id_task")]
    pub id_task: i64,

    #[serde(default)]
    pub is_processing: bool,

    #[serde(default)]
    pub n_ctx: usize,

    #[serde(default)]
    pub n_prompt_tokens: usize,

    #[serde(default)]
    pub n_prompt_tokens_processed: usize,

    #[serde(default)]
    pub n_decoded: usize,

    #[serde(default)]
    pub t_token: Option<f64>,

    // Calculated fields across polls
    #[serde(skip)]
    pub calculated_tps: Option<f64>,
}

fn default_id_task() -> i64 {
    -1
}

impl SlotInfo {
    pub fn state_str(&self) -> &'static str {
        if self.is_processing {
            if self.n_decoded > 0 {
                "GENERATING"
            } else if self.n_prompt_tokens_processed < self.n_prompt_tokens {
                "PREFILLING"
            } else {
                "GENERATING"
            }
        } else {
            "IDLE"
        }
    }

    pub fn context_used(&self) -> usize {
        self.n_prompt_tokens_processed.max(self.n_prompt_tokens) + self.n_decoded
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ModelsResponse {
    #[serde(default)]
    data: Vec<ModelEntry>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ModelEntry {
    #[serde(default)]
    id: String,
}

#[derive(Debug, Clone)]
pub struct LlamaInstance {
    pub pid: u32,
    pub port: u16,
    pub model_name: String,
    pub active_slots: usize,
    pub total_slots: usize,
    pub tokens_per_sec: f64,
    pub slots: Vec<SlotInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct MultiLlamaStats {
    pub instances: Vec<LlamaInstance>,
    pub total_tokens_per_sec: f64,
}

#[derive(Clone)]
struct PrevSlotState {
    decoded: usize,
    timestamp: Instant,
}

pub struct MultiLlamaManager {
    client: reqwest::Client,
    sys: System,
    default_ports: Vec<u16>,
    prev_slots: HashMap<(u16, u32), PrevSlotState>,
}

impl MultiLlamaManager {
    pub fn new() -> Self {
        let refresh = RefreshKind::nothing().with_processes(ProcessRefreshKind::everything());
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_millis(400))
                .build()
                .unwrap(),
            sys: System::new_with_specifics(refresh),
            default_ports: vec![8080, 8081, 8082, 8083, 8084, 8085, 8000],
            prev_slots: HashMap::new(),
        }
    }

    fn scan_processes(&mut self) -> HashMap<u16, (u32, String)> {
        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let mut map = HashMap::new();

        for (pid, process) in self.sys.processes() {
            let name = process.name().to_string_lossy();
            let cmd: Vec<String> = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect();

            if name.contains("llama-server") || cmd.iter().any(|arg| arg.contains("llama-server")) {
                let mut port = 8080;
                let mut model = String::new();

                let mut iter = cmd.iter().peekable();
                while let Some(arg) = iter.next() {
                    if arg == "--port" || arg == "-p" {
                        if let Some(p) = iter.peek() {
                            if let Ok(val) = p.parse::<u16>() {
                                port = val;
                            }
                        }
                    } else if (arg == "-m" || arg == "--model" || arg == "-a" || arg == "--alias")
                        && model.is_empty()
                    {
                        if let Some(m) = iter.peek() {
                            let file = std::path::Path::new(m)
                                .file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_else(|| m.to_string());
                            model = file;
                        }
                    }
                }
                map.insert(port, (pid.as_u32(), model));
            }
        }
        map
    }

    pub async fn poll(&mut self) -> MultiLlamaStats {
        let process_map = self.scan_processes();
        let mut candidate_ports: HashSet<u16> = self.default_ports.iter().copied().collect();
        for &port in process_map.keys() {
            candidate_ports.insert(port);
        }

        let mut tasks = Vec::new();

        for port in candidate_ports {
            let client = self.client.clone();
            let proc_info = process_map.get(&port).cloned();

            tasks.push(tokio::spawn(async move {
                let slots_url = format!("http://127.0.0.1:{}/slots", port);
                let resp = client.get(&slots_url).send().await.ok()?;

                if !resp.status().is_success() {
                    return None;
                }

                let slots: Vec<SlotInfo> = resp.json().await.ok()?;
                if slots.is_empty() {
                    return None;
                }

                let mut model_name = proc_info
                    .as_ref()
                    .map(|(_, m)| m.clone())
                    .unwrap_or_default();

                if model_name.is_empty() {
                    let models_url = format!("http://127.0.0.1:{}/v1/models", port);
                    if let Ok(models_resp) = client.get(&models_url).send().await {
                        if let Ok(models_data) = models_resp.json::<ModelsResponse>().await {
                            if let Some(first) = models_data.data.first() {
                                model_name = std::path::Path::new(&first.id)
                                    .file_name()
                                    .map(|f| f.to_string_lossy().to_string())
                                    .unwrap_or_else(|| first.id.clone());
                            }
                        }
                    }
                }

                if model_name.is_empty() {
                    model_name = format!("llama-server:{}", port);
                }

                Some((port, proc_info.map(|(p, _)| p).unwrap_or(0), model_name, slots))
            }));
        }

        let mut instances = Vec::new();
        let mut total_tps = 0.0;
        let now = Instant::now();

        for task in tasks {
            if let Ok(Some((port, pid, model_name, mut slots))) = task.await {
                let mut inst_tps = 0.0;

                for slot in &mut slots {
                    let key = (port, slot.id);

                    if slot.is_processing {
                        if let Some(prev) = self.prev_slots.get(&key) {
                            let dt = now.duration_since(prev.timestamp).as_secs_f64();
                            if dt > 0.1 && slot.n_decoded >= prev.decoded {
                                let delta_tokens = (slot.n_decoded - prev.decoded) as f64;
                                let tps = delta_tokens / dt;
                                if tps > 0.0 {
                                    slot.calculated_tps = Some(tps);
                                    inst_tps += tps;
                                }
                            }
                        }

                        // Fallback to internal t_token if delta hasn't populated yet
                        if slot.calculated_tps.is_none() {
                            if let Some(ms) = slot.t_token {
                                if ms > 0.0 {
                                    let tps = 1000.0 / ms;
                                    slot.calculated_tps = Some(tps);
                                    inst_tps += tps;
                                }
                            }
                        }

                        self.prev_slots.insert(
                            key,
                            PrevSlotState {
                                decoded: slot.n_decoded,
                                timestamp: now,
                            },
                        );
                    } else {
                        // Slot is idle; reset state tracking
                        self.prev_slots.remove(&key);
                        slot.calculated_tps = None;
                    }
                }

                total_tps += inst_tps;
                let active = slots.iter().filter(|s| s.is_processing).count();

                instances.push(LlamaInstance {
                    pid,
                    port,
                    model_name,
                    active_slots: active,
                    total_slots: slots.len(),
                    tokens_per_sec: inst_tps,
                    slots,
                });
            }
        }

        instances.sort_by_key(|inst| inst.port);

        MultiLlamaStats {
            instances,
            total_tokens_per_sec: total_tps,
        }
    }
}
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

#[derive(Debug, Clone, Deserialize)]
pub struct SlotInfo {
    pub id: u32,
    pub state: u32, // 0 = IDLE, 1 = PROCESSING_PROMPT, 2 = GENERATING
    pub n_ctx: usize,
    pub n_past: usize,
    pub n_decoded: Option<usize>,
    pub t_token: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PropsResponse {
    pub default_generation_settings: Option<serde_json::Value>,
    pub total_slots: Option<usize>,
    pub model_path: Option<String>,
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

pub struct MultiLlamaManager {
    client: reqwest::Client,
    sys: System,
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
        }
    }

    /// Discover running `llama-server` processes and their listening ports
    pub fn discover_ports(&mut self) -> HashMap<u16, (u32, String)> {
        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let mut discovered = HashMap::new();

        for (pid, process) in self.sys.processes() {
            let name = process.name().to_string_lossy();
            let cmd: Vec<String> = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect();

            if name.contains("llama-server") || cmd.iter().any(|arg| arg.contains("llama-server")) {
                let mut port = 8080; // default
                let mut model = String::from("unknown");

                // Parse command-line args for --port and -m
                let mut iter = cmd.iter().peekable();
                while let Some(arg) = iter.next() {
                    if arg == "--port" || arg == "-p" {
                        if let Some(p) = iter.peek() {
                            if let Ok(parsed_port) = p.parse::<u16>() {
                                port = parsed_port;
                            }
                        }
                    } else if arg == "-m" || arg == "--model" || arg == "-a" || arg == "--alias" {
                        if let Some(m) = iter.peek() {
                            // Extract just the filename if it is a path
                            let file = std::path::Path::new(m)
                                .file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_else(|| m.to_string());
                            model = file;
                        }
                    }
                }
                discovered.insert(port, (pid.as_u32(), model));
            }
        }

        // Fallback default probes if process scanning found none (e.g. running inside container)
        if discovered.is_empty() {
            discovered.insert(8080, (0, "llama-server".into()));
            discovered.insert(8081, (0, "llama-server".into()));
            discovered.insert(8082, (0, "llama-server".into()));
        }

        discovered
    }

    pub async fn poll(&mut self) -> MultiLlamaStats {
        let ports = self.discover_ports();
        let mut tasks = Vec::new();

        for (port, (pid, model_hint)) in ports {
            let client = self.client.clone();
            tasks.push(tokio::spawn(async move {
                let url = format!("http://127.0.0.1:{}/slots", port);
                if let Ok(resp) = client.get(&url).send().await {
                    if resp.status().is_success() {
                        let slots: Vec<SlotInfo> = resp.json().await.unwrap_or_default();
                        let total = slots.len();
                        let active = slots.iter().filter(|s| s.state != 0).count();
                        let avg_tps = slots
                            .iter()
                            .filter_map(|s| s.t_token)
                            .filter(|t| *t > 0.0)
                            .map(|t| 1000.0 / t)
                            .sum::<f64>();

                        return Some(LlamaInstance {
                            pid,
                            port,
                            model_name: model_hint,
                            active_slots: active,
                            total_slots: total,
                            tokens_per_sec: avg_tps,
                            slots,
                        });
                    }
                }
                None
            }));
        }

        let mut instances = Vec::new();
        let mut total_tps = 0.0;

        for task in tasks {
            if let Ok(Some(instance)) = task.await {
                total_tps += instance.tokens_per_sec;
                instances.push(instance);
            }
        }

        instances.sort_by_key(|inst| inst.port);

        MultiLlamaStats {
            instances,
            total_tokens_per_sec: total_tps,
        }
    }
}
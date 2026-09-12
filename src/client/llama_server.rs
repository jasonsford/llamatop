use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct SlotInfo {
    pub id: u32,
    pub state: u32, // 0 = IDLE, 1 = PROCESSING_PROMPT, 2 = GENERATING
    pub n_ctx: usize,
    pub n_past: usize,
    pub n_decoded: Option<usize>,
    pub t_token: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct LlamaServerStats {
    pub is_online: bool,
    pub active_slots: usize,
    pub total_slots: usize,
    pub tokens_per_sec: f64,
    pub slots: Vec<SlotInfo>,
}

pub struct LlamaClient {
    base_url: String,
    client: reqwest::Client,
}

impl LlamaClient {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            base_url: format!("http://{}:{}", host, port),
            client: reqwest::Client::builder()
                .timeout(Duration::from_millis(500))
                .build()
                .unwrap(),
        }
    }

    pub async fn poll(&self) -> LlamaServerStats {
        let url = format!("{}/slots", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let slots: Vec<SlotInfo> = resp.json().await.unwrap_or_default();
                let total = slots.len();
                let active = slots.iter().filter(|s| s.state != 0).count();
                let avg_tps = slots
                    .iter()
                    .filter_map(|s| s.t_token)
                    .filter(|t| *t > 0.0)
                    .map(|t| 1000.0 / t)
                    .sum::<f64>();

                LlamaServerStats {
                    is_online: true,
                    active_slots: active,
                    total_slots: total,
                    tokens_per_sec: avg_tps,
                    slots,
                }
            }
            _ => LlamaServerStats::default(),
        }
    }
}
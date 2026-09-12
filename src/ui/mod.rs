use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table},
    Frame,
};

use crate::client::llama_server::MultiLlamaStats;
use crate::telemetry::{GpuDeviceStats, HostStats};

const CYAN: Color = Color::Rgb(90, 202, 225);
const GREEN: Color = Color::Rgb(88, 211, 147);
const YELLOW: Color = Color::Rgb(246, 193, 79);
const RED: Color = Color::Rgb(248, 113, 113);
const DIM: Color = Color::Rgb(78, 90, 108);
const PANEL: Color = Color::Rgb(18, 24, 34);

pub fn draw_dashboard(
    frame: &mut Frame,
    host: &HostStats,
    gpus: &[GpuDeviceStats],
    llama: &MultiLlamaStats,
) {
    let area = frame.area();

    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(10, 14, 21))),
        area,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header bar
            Constraint::Length(10), // Host RAM/CPU & Multi-Instance Slot Table
            Constraint::Min(12),    // 5-GPU Cards Strip
        ])
        .split(area);

    draw_header(frame, chunks[0], llama);
    draw_system_overview(frame, chunks[1], host, llama);
    draw_gpus(frame, chunks[2], gpus);
}

fn draw_header(frame: &mut Frame, area: Rect, llama: &MultiLlamaStats) {
    let count = llama.instances.len();
    let status_span = if count > 0 {
        Span::styled(
            format!(" ● {} INSTANCE{} ONLINE ", count, if count > 1 { "S" } else { "" }),
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" ○ NO SERVERS DETECTED ", Style::default().fg(RED).add_modifier(Modifier::BOLD))
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(" llamatop ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("v0.1.0 ", Style::default().fg(CYAN)),
        Span::raw("│ "),
        status_span,
        Span::styled(
            format!("(Aggregate: {:.1} tok/s)", llama.total_tokens_per_sec),
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(DIM)),
    );

    frame.render_widget(title, area);
}

fn draw_system_overview(frame: &mut Frame, area: Rect, host: &HostStats, llama: &MultiLlamaStats) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Host Memory & CPU
    let ram_used_gb = host.used_memory as f64 / 1024.0 / 1024.0 / 1024.0;
    let ram_total_gb = host.total_memory as f64 / 1024.0 / 1024.0 / 1024.0;
    let ram_pct = if host.total_memory > 0 {
        ((host.used_memory as f64 / host.total_memory as f64) * 100.0) as u16
    } else {
        0
    };

    let host_block = Block::default()
        .title(" Host System ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .style(Style::default().bg(PANEL));

    let inner = host_block.inner(cols[0]);
    frame.render_widget(host_block, cols[0]);

    let host_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(2), Constraint::Min(1)])
        .split(inner);

    let ram_gauge = Gauge::default()
        .block(Block::default().title(format!("RAM: {:.1} / {:.1} GiB", ram_used_gb, ram_total_gb)))
        .gauge_style(Style::default().fg(CYAN).bg(DIM))
        .percent(ram_pct);
    frame.render_widget(ram_gauge, host_rows[0]);

    let cpu_gauge = Gauge::default()
        .block(Block::default().title(format!("CPU: {:.1}%", host.cpu_load_percent)))
        .gauge_style(Style::default().fg(YELLOW).bg(DIM))
        .percent(host.cpu_load_percent.clamp(0.0, 100.0) as u16);
    frame.render_widget(cpu_gauge, host_rows[1]);

    // Multi-Server Active Slots Table
    let slots_block = Block::default()
        .title(" Active Models & Server Slots ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .style(Style::default().bg(PANEL));

    let inner_slots = slots_block.inner(cols[1]);
    frame.render_widget(slots_block, cols[1]);

    let mut rows = Vec::new();
    for inst in &llama.instances {
        for s in &inst.slots {
            let state = s.state_str();
            let color = match state {
                "GENERATING" => GREEN,
                "PREFILLING" | "BUSY" => YELLOW,
                _ => DIM,
            };

            let ctx_label = if s.n_ctx > 0 {
                format!("{}/{} ({:.0}%)", s.context_used(), s.n_ctx, (s.context_used() as f64 / s.n_ctx as f64) * 100.0)
            } else {
                "—".into()
            };

            let speed_label = if let Some(tps) = s.calculated_tps {
                format!("{:.1} t/s", tps)
            } else if let Some(t) = s.t_token {
                format!("{:.1}ms", t)
            } else {
                "—".into()
            };

            rows.push(
                Row::new(vec![
                    format!(":{}", inst.port),
                    truncate_str(&inst.model_name, 26),
                    format!("Slot {}", s.id),
                    state.to_string(),
                    ctx_label,
                    speed_label,
                ])
                .style(Style::default().fg(color)),
            );
        }
    }

    if rows.is_empty() {
        rows.push(
            Row::new(vec![
                "—",
                "No active server slots found",
                "—",
                "—",
                "—",
                "—",
            ])
            .style(Style::default().fg(DIM)),
        );
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),  // Port
            Constraint::Length(26), // Model
            Constraint::Length(8),  // Slot
            Constraint::Length(12), // State
            Constraint::Length(18), // Context
            Constraint::Min(9),     // Speed
        ],
    )
    .header(
        Row::new(vec!["Port", "Model", "Slot", "State", "Context", "Speed"])
            .style(Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
    );

    frame.render_widget(table, inner_slots);
}

fn draw_gpus(frame: &mut Frame, area: Rect, gpus: &[GpuDeviceStats]) {
    let block = Block::default()
        .title(format!(" NVIDIA Accelerators ({}) ", gpus.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .style(Style::default().bg(PANEL));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if gpus.is_empty() {
        let empty_msg = Paragraph::new("No GPUs detected or NVML not initialized.")
            .style(Style::default().fg(YELLOW));
        frame.render_widget(empty_msg, inner);
        return;
    }

    let constraints = vec![Constraint::Ratio(1, gpus.len() as u32); gpus.len()];
    let gpu_columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(inner);

    for (i, gpu) in gpus.iter().enumerate() {
        draw_gpu_card(frame, gpu_columns[i], gpu);
    }
}

fn draw_gpu_card(frame: &mut Frame, area: Rect, gpu: &GpuDeviceStats) {
    let clean_name = clean_gpu_name(&gpu.name);
    let card = Block::default()
        .title(format!(" [{}] {} ", gpu.index, truncate_str(&clean_name, 18)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN))
        .style(Style::default().bg(PANEL));

    let inner = card.inner(area);
    frame.render_widget(card, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // VRAM Gauge
            Constraint::Length(2), // Core Util Gauge
            Constraint::Length(2), // Memory Util Gauge
            Constraint::Min(2),    // Temp & Power Stats
        ])
        .split(inner);

    let vram_used_gb = gpu.vram_used as f64 / 1024.0 / 1024.0 / 1024.0;
    let vram_total_gb = gpu.vram_total as f64 / 1024.0 / 1024.0 / 1024.0;
    let vram_pct = if gpu.vram_total > 0 {
        ((gpu.vram_used as f64 / gpu.vram_total as f64) * 100.0) as u16
    } else {
        0
    };

    let vram_gauge = Gauge::default()
        .block(Block::default().title(format!("VRAM: {:.1}/{:.1}G", vram_used_gb, vram_total_gb)))
        .gauge_style(Style::default().fg(GREEN).bg(DIM))
        .percent(vram_pct);
    frame.render_widget(vram_gauge, rows[0]);

    let core_gauge = Gauge::default()
        .block(Block::default().title(format!("Core: {}%", gpu.gpu_util)))
        .gauge_style(Style::default().fg(CYAN).bg(DIM))
        .percent(gpu.gpu_util.min(100) as u16);
    frame.render_widget(core_gauge, rows[1]);

    let mem_gauge = Gauge::default()
        .block(Block::default().title(format!("Bus: {}%", gpu.mem_util)))
        .gauge_style(Style::default().fg(YELLOW).bg(DIM))
        .percent(gpu.mem_util.min(100) as u16);
    frame.render_widget(mem_gauge, rows[2]);

    let details = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(format!("{}°C ", gpu.temp_c), Style::default().fg(if gpu.temp_c > 80 { RED } else { GREEN })),
            Span::styled(format!("{:.0}W/{:.0}W", gpu.power_watts, gpu.power_limit_watts), Style::default().fg(Color::White)),
        ]),
    ]);
    frame.render_widget(details, rows[3]);
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        format!("{}…", &s.chars().take(max_len.saturating_sub(1)).collect::<String>())
    } else {
        s.to_string()
    }
}

fn clean_gpu_name(name: &str) -> String {
    name.replace("NVIDIA GeForce ", "")
        .replace("NVIDIA ", "")
        .trim()
        .to_string()
}
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table},
    Frame,
};

use crate::client::llama_server::LlamaServerStats;
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
    llama: &LlamaServerStats,
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
            Constraint::Length(10), // Host RAM/CPU & llama-server status
            Constraint::Min(12),    // Multi-GPU cards
        ])
        .split(area);

    draw_header(frame, chunks[0], llama);
    draw_system_overview(frame, chunks[1], host, llama);
    draw_gpus(frame, chunks[2], gpus);
}

fn draw_header(frame: &mut Frame, area: Rect, llama: &LlamaServerStats) {
    let status_span = if llama.is_online {
        Span::styled(" ● ONLINE ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" ○ OFFLINE ", Style::default().fg(RED).add_modifier(Modifier::BOLD))
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(" llamatop ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("v0.1.0 ", Style::default().fg(CYAN)),
        Span::raw("│ llama.cpp server: "),
        status_span,
        Span::styled(
            format!("({:.1} tok/s avg)", llama.tokens_per_sec),
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

fn draw_system_overview(frame: &mut Frame, area: Rect, host: &HostStats, llama: &LlamaServerStats) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
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
        .title(" Host System (CPU / RAM) ")
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
        .block(Block::default().title(format!("System RAM: {:.1} / {:.1} GiB", ram_used_gb, ram_total_gb)))
        .gauge_style(Style::default().fg(CYAN).bg(DIM))
        .percent(ram_pct);
    frame.render_widget(ram_gauge, host_rows[0]);

    let cpu_gauge = Gauge::default()
        .block(Block::default().title(format!("CPU Usage: {:.1}%", host.cpu_load_percent)))
        .gauge_style(Style::default().fg(YELLOW).bg(DIM))
        .percent(host.cpu_load_percent.clamp(0.0, 100.0) as u16);
    frame.render_widget(cpu_gauge, host_rows[1]);

    // llama-server Slots overview
    let slots_block = Block::default()
        .title(" llama-server Slots ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .style(Style::default().bg(PANEL));

    let inner_slots = slots_block.inner(cols[1]);
    frame.render_widget(slots_block, cols[1]);

    let slot_rows: Vec<Row> = llama
        .slots
        .iter()
        .map(|s| {
            let state_str = match s.state {
                1 => "PROCESSING",
                2 => "GENERATING",
                _ => "IDLE",
            };
            let color = match s.state {
                1 => YELLOW,
                2 => GREEN,
                _ => DIM,
            };
            Row::new(vec![
                format!("Slot {}", s.id),
                state_str.to_string(),
                format!("{}/{} ctx", s.n_past, s.n_ctx),
                s.t_token.map(|t| format!("{:.1} ms", t)).unwrap_or_else(|| "—".into()),
            ])
            .style(Style::default().fg(color))
        })
        .collect();

    let table = Table::new(
        slot_rows,
        [
            Constraint::Length(10),
            Constraint::Length(14),
            Constraint::Length(16),
            Constraint::Min(10),
        ],
    )
    .header(
        Row::new(vec!["Slot", "State", "Context", "Speed"])
            .style(Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
    );

    frame.render_widget(table, inner_slots);
}

fn draw_gpus(frame: &mut Frame, area: Rect, gpus: &[GpuDeviceStats]) {
    let block = Block::default()
        .title(" NVIDIA Accelerators ")
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
    let card = Block::default()
        .title(format!(" [{}] {} ", gpu.index, gpu.name))
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
        .block(Block::default().title(format!("VRAM: {:.2} / {:.2} GiB", vram_used_gb, vram_total_gb)))
        .gauge_style(Style::default().fg(GREEN).bg(DIM))
        .percent(vram_pct);
    frame.render_widget(vram_gauge, rows[0]);

    let core_gauge = Gauge::default()
        .block(Block::default().title(format!("Core Utilization: {}%", gpu.gpu_util)))
        .gauge_style(Style::default().fg(CYAN).bg(DIM))
        .percent(gpu.gpu_util.min(100) as u16);
    frame.render_widget(core_gauge, rows[1]);

    let mem_gauge = Gauge::default()
        .block(Block::default().title(format!("Memory Bus Activity: {}%", gpu.mem_util)))
        .gauge_style(Style::default().fg(YELLOW).bg(DIM))
        .percent(gpu.mem_util.min(100) as u16);
    frame.render_widget(mem_gauge, rows[2]);

    let details = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Temp: ", Style::default().fg(DIM)),
            Span::styled(
                format!("{}°C", gpu.temp_c),
                Style::default().fg(if gpu.temp_c > 80 { RED } else { GREEN }),
            ),
            Span::styled("  Power: ", Style::default().fg(DIM)),
            Span::styled(
                format!("{:.1}W / {:.1}W", gpu.power_watts, gpu.power_limit_watts),
                Style::default().fg(Color::White),
            ),
        ]),
    ]);
    frame.render_widget(details, rows[3]);
}
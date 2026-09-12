use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Sparkline, Table},
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
            Constraint::Length(10), // Host RAM/Swap & Instances Table
            Constraint::Min(14),    // 5-GPU Cards with Sparklines & Attribution
        ])
        .split(area);

    draw_header(frame, chunks[0], llama);
    draw_system_overview(frame, chunks[1], host, llama);
    draw_gpus(frame, chunks[2], gpus, llama);
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
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    // Host Memory, Swap, Paging & CPU
    let ram_used_gb = host.used_memory as f64 / 1024.0 / 1024.0 / 1024.0;
    let ram_total_gb = host.total_memory as f64 / 1024.0 / 1024.0 / 1024.0;
    let ram_pct = if host.total_memory > 0 {
        ((host.used_memory as f64 / host.total_memory as f64) * 100.0) as u16
    } else {
        0
    };

    let swap_used_gb = host.used_swap as f64 / 1024.0 / 1024.0 / 1024.0;
    let swap_total_gb = host.total_swap as f64 / 1024.0 / 1024.0 / 1024.0;
    let swap_pct = if host.total_swap > 0 {
        ((host.used_swap as f64 / host.total_swap as f64) * 100.0) as u16
    } else {
        0
    };

    let host_block = Block::default()
        .title(" Host System (Memory & Paging) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .style(Style::default().bg(PANEL));

    let inner = host_block.inner(cols[0]);
    frame.render_widget(host_block, cols[0]);

    let host_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // RAM
            Constraint::Length(2), // Swap
            Constraint::Length(2), // CPU
            Constraint::Length(1), // Paging I/O
        ])
        .split(inner);

    let ram_gauge = Gauge::default()
        .block(Block::default().title(format!("RAM: {:.1}/{:.1} GiB", ram_used_gb, ram_total_gb)))
        .gauge_style(Style::default().fg(CYAN).bg(DIM))
        .percent(ram_pct);
    frame.render_widget(ram_gauge, host_rows[0]);

    let swap_gauge = Gauge::default()
        .block(Block::default().title(format!("Swap: {:.1}/{:.1} GiB", swap_used_gb, swap_total_gb)))
        .gauge_style(Style::default().fg(if swap_pct > 50 { RED } else { YELLOW }).bg(DIM))
        .percent(swap_pct);
    frame.render_widget(swap_gauge, host_rows[1]);

    let cpu_gauge = Gauge::default()
        .block(Block::default().title(format!("CPU: {:.1}%", host.cpu_load_percent)))
        .gauge_style(Style::default().fg(GREEN).bg(DIM))
        .percent(host.cpu_load_percent.clamp(0.0, 100.0) as u16);
    frame.render_widget(cpu_gauge, host_rows[2]);

    let paging_line = Paragraph::new(Line::from(vec![
        Span::styled("Paging I/O: ", Style::default().fg(DIM)),
        Span::styled(format!("▼ {:.1} MB/s in  ", host.page_in_mb_s), Style::default().fg(if host.page_in_mb_s > 5.0 { RED } else { Color::White })),
        Span::styled(format!("▲ {:.1} MB/s out", host.page_out_mb_s), Style::default().fg(if host.page_out_mb_s > 5.0 { RED } else { Color::White })),
    ]));
    frame.render_widget(paging_line, host_rows[3]);

    // Active Instances Table (No Slot Column)
    let slots_block = Block::default()
        .title(" Active Models & Instances ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .style(Style::default().bg(PANEL));

    let inner_slots = slots_block.inner(cols[1]);
    frame.render_widget(slots_block, cols[1]);

    let mut rows = Vec::new();
    for inst in &llama.instances {
        for s in &inst.slots {
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
                    truncate_str(&inst.model_name, 34),
                    ctx_label,
                    speed_label,
                ])
                .style(Style::default().fg(if s.is_processing { Color::White } else { DIM })),
            );
        }
    }

    if rows.is_empty() {
        rows.push(
            Row::new(vec![
                "—",
                "No active server instances found",
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
            Constraint::Length(34), // Model
            Constraint::Length(22), // Context
            Constraint::Min(10),    // Speed
        ],
    )
    .header(
        Row::new(vec!["Port", "Model", "Context", "Speed"])
            .style(Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
    );

    frame.render_widget(table, inner_slots);
}

fn draw_gpus(frame: &mut Frame, area: Rect, gpus: &[GpuDeviceStats], llama: &MultiLlamaStats) {
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
        draw_gpu_card(frame, gpu_columns[i], gpu, llama);
    }
}

fn clean_gpu_name(name: &str) -> String {
    name.replace("NVIDIA GeForce ", "")
        .replace("NVIDIA ", "")
        .trim()
        .to_string()
}

fn draw_gpu_card(frame: &mut Frame, area: Rect, gpu: &GpuDeviceStats, llama: &MultiLlamaStats) {
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
            Constraint::Length(1), // Compute Utilization Sparkline
            Constraint::Length(1), // Memory Controller Sparkline
            Constraint::Length(2), // Temp & Power
            Constraint::Length(1), // PCIe TX/RX Throughput
            Constraint::Min(1),    // Active Process / Model Attribution
        ])
        .split(inner);

    // 1. VRAM Gauge
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

    // 2. Compute Utilization Sparkline
    let compute_data: Vec<u64> = gpu.compute_history.iter().copied().collect();
    let compute_spark = Sparkline::default()
        .block(Block::default().title(format!("SM: {}%", gpu.gpu_util)))
        .data(&compute_data)
        .max(100)
        .style(Style::default().fg(CYAN));
    frame.render_widget(compute_spark, rows[1]);

    // 3. Memory Controller Sparkline
    let mem_data: Vec<u64> = gpu.memory_history.iter().copied().collect();
    let mem_spark = Sparkline::default()
        .block(Block::default().title(format!("Bus: {}%", gpu.mem_util)))
        .data(&mem_data)
        .max(100)
        .style(Style::default().fg(YELLOW));
    frame.render_widget(mem_spark, rows[2]);

    // 4. Temp & Power Details
    let details = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(format!("{}°C ", gpu.temp_c), Style::default().fg(if gpu.temp_c > 80 { RED } else { GREEN })),
            Span::styled(format!("{:.0}W/{:.0}W", gpu.power_watts, gpu.power_limit_watts), Style::default().fg(Color::White)),
        ]),
    ]);
    frame.render_widget(details, rows[3]);

    // 5. PCIe Throughput
    let pcie_line = Paragraph::new(Line::from(vec![
        Span::styled("PCIe: ", Style::default().fg(DIM)),
        Span::styled(format!("TX {:.0}M ", gpu.pcie_tx_mb_s), Style::default().fg(Color::White)),
        Span::styled(format!("RX {:.0}M", gpu.pcie_rx_mb_s), Style::default().fg(Color::White)),
    ]));
    frame.render_widget(pcie_line, rows[4]);

    // 6. Process / Model Attribution
    let mut model_labels = Vec::new();
    for proc in &gpu.running_processes {
        if let Some(inst) = llama.instances.iter().find(|i| i.pid == proc.pid) {
            let proc_gb = proc.used_vram as f64 / 1024.0 / 1024.0 / 1024.0;
            model_labels.push(Span::styled(
                format!("▶ :{} ({:.1}G)", inst.port, proc_gb),
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
            ));
        }
    }

    if model_labels.is_empty() {
        if gpu.vram_used > 500 * 1024 * 1024 {
            model_labels.push(Span::styled("● Allocated", Style::default().fg(DIM)));
        } else {
            model_labels.push(Span::styled("○ Free", Style::default().fg(DIM)));
        }
    }

    frame.render_widget(Paragraph::new(Line::from(model_labels)), rows[5]);
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        format!("{}…", &s.chars().take(max_len.saturating_sub(1)).collect::<String>())
    } else {
        s.to_string()
    }
}
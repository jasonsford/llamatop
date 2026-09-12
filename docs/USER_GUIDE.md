# llamatop User Guide

[Back to the README](../README.md) · [Contributing Guide](../CONTRIBUTING.md) · [Security Policy](../SECURITY.md)

Comprehensive controls, runtime configuration, telemetry metrics, and architectural explanations for `llamatop`.

---

## Overview & Dashboard Layout

`llamatop` provides a unified, real-time TUI dashboard split into functional telemetry panes designed specifically for monitoring local LLM inference across multi-GPU NVIDIA rigs on Linux (with cross-platform local development support on Windows and macOS).

---

## Telemetry Panes & Metrics Explained

### 1. Host System & Kernel Paging
- **CPU & System RAM:** Gathered via `sysinfo`. High host CPU usage often indicates token prefill preprocessing, prompt template formatting, tokenization, or CPU-offloaded layers.
- **Swap Allocation & Kernel Paging:** Queried directly from `/proc/vmstat` (`pswpin` and `pswpout`) on Linux. When host RAM exhausts and context data or model weights spill into swap memory, paging activity spikes, resulting in severe inference throughput drops.

### 2. NVIDIA GPU Telemetry (Direct NVML)
On Linux systems, `llamatop` queries the official NVIDIA Management Library (`NVML`) directly:
- **VRAM Allocation:** Real-time allocated video memory versus physical hardware capacity.
- **Compute (SM) Utilization:** Percentage of Streaming Multiprocessors actively executing CUDA kernels, tracked across a 30-sample rolling sparkline.
- **Memory Bus Utilization:** Percentage of time the physical memory controller is busy. Memory bandwidth is typically the primary bottleneck during LLM autoregressive token generation (decoding phase).
- **PCIe Bus Bandwidth:** Host-to-device (TX) and device-to-host (RX) throughput in MB/s. Useful for detecting bottlenecks during initial model layer weight transfers, dynamic context ingestion, or multi-GPU pipeline tensor transfers across PCIe risers.
- **Thermal & Power Draw:** Core temperatures (°C) and dynamic power consumption (Watts) relative to configured TDP caps.

### 3. Engine & Server Telemetry (`llama-server`)
`llamatop` inspects active inference engine metrics:
- **Active / Total Slots:** Concurrent generation requests compared to total initialized slots (`--parallel`).
- **Context Capacity:** KV-cache token consumption across active slots relative to configured `--ctx-size`.
- **Generation Speed:** Real-time token generation throughput (`tokens/sec`) and Time-To-First-Token (`TTFT`) latency scraped from the Prometheus `/metrics` endpoint.
- **Process & Model Attribution:** Maps active Linux process IDs (`PIDs`) running `llama-server` to their listening ports, model filenames, and associated physical GPUs.

---

## Interactive Controls

| Key | Action |
| --- | --- |
| `q` / `Esc` / `Ctrl-C` | Exit cleanly and restore terminal raw mode |
| `+` / `=` | Increase polling frequency (shorter interval) |
| `-` | Decrease polling frequency (longer interval) |
| `Space` / `p` | Pause or resume real-time sampling |
| `r` | Reset rolling sparkline history |

---

## Command-Line Usage

```bash
llamatop [OPTIONS]
```

### Options

| Flag | Option | Description | Default |
| --- | --- | --- | --- |
| `-p` | `--port <PORT>` | Target port of `llama-server` instance | `8080` |
| `-H` | `--host <HOST>` | Target hostname or IP address | `127.0.0.1` |
| `-i` | `--interval <SECS>` | Telemetry refresh interval in seconds | `1` |
| `-h` | `--help` | Print command-line help information | — |
| `-V` | `--version` | Print installed version | — |

---

## Configuring `llama-server` for Full Telemetry

To enable engine-level token throughput, TTFT latency, and slot context counters, start `llama-server` with the `--metrics` flag:

```bash
llama-server \
  -m /models/Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf \
  --ctx-size 16384 \
  --n-gpu-layers 99 \
  --parallel 4 \
  --metrics \
  --port 8080
```

> **Note:** If `llama-server` is started without `--metrics`, `llamatop` degrades gracefully: it will continue monitoring VRAM, SM compute, memory bus, and slot status via `/slots`, displaying `—` for metric counters that are not exposed.

---

## Multi-GPU Topologies

`llamatop` enumerates all available NVIDIA GPUs via NVML:
- Supports asymmetric multi-GPU setups (e.g., pairing RTX 3080 and RTX 4060 Ti series).
- Displays individual per-card sparklines for both compute utilization and memory bus activity.
- Dynamically scales GPU cards across terminal columns to match viewport dimensions.

---

## Privacy & Security

`llamatop` is strictly read-only and local-first:
- Communicates exclusively over local loopback sockets (`127.0.0.1`) and native kernel/driver interfaces.
- Does not log, intercept, or inspect prompt text, completion tokens, or model weights.
- Contains no analytics, background phone-home routines, or third-party telemetry collection.
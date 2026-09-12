# llamatop

**A real-time terminal monitor for local LLM inference across multi-GPU NVIDIA rigs.**

Track engine-level generation speed, active inference slots, context saturation, and hardware bottlenecks in real time. `llamatop` couples direct NVIDIA hardware telemetry (NVML) with native `llama-server` Prometheus and slot metrics in a dense, low-overhead TUI.

[Quick Start](#quick-start) · [Telemetry & Features](#telemetry--features) · [User Guide](docs/USER_GUIDE.md) · [Contributing](CONTRIBUTING.md)

---

## Quick Start

### Clone and build locally:

```bash
git clone [https://github.com/jasonsford/llamatop.git](https://github.com/jasonsford/llamatop.git)
cd llamatop
cargo build --release
./target/release/llamatop
```

*(Note: Cross-platform development is fully supported on Windows and macOS via fallback mocks; live NVML and Linux paging telemetry activate automatically when running on Linux with proprietary NVIDIA drivers installed).*

### Launching `llama-server` with Metrics

To enable full throughput tracking, Time-To-First-Token (TTFT), and context load counters, launch `llama-server` with the `--metrics` flag:

```bash
llama-server \
  -m /models/Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf \
  --ctx-size 16384 \
  --n-gpu-layers 99 \
  --parallel 4 \
  --metrics \
  --port 8080
```

### Running `llamatop`

```bash
# Monitor local server on default port 8080
llamatop

# Monitor a specific port or custom interval
llamatop --port 8080 --interval 1
```

---

## Telemetry & Features

| Telemetry Pane | Sourced Via | Key Metrics Tracked |
| --- | --- | --- |
| **Host System** | `sysinfo` & `/proc/vmstat` | Host CPU %, RAM allocation, swap memory, and kernel swap page-in/page-out rates (`MB/s`). |
| **NVIDIA Multi-GPU** | Direct NVML (`nvml-wrapper`) | Per-GPU VRAM usage, Streaming Multiprocessor (SM) % sparklines, Memory Bus % sparklines, core temp, dynamic power draw vs. TDP, and PCIe TX/RX throughput. |
| **Inference Server** | `llama-server` HTTP / Prometheus | Active vs. total parallel slots (`/slots`), KV context load high-water mark, live token generation throughput (`tok/s`), and TTFT latency. |
| **Process Mapping** | System process tables | Automatic correlation of listening server ports to OS PIDs and associated physical GPUs. |

### Diagnostic Signals

- **Memory Bus vs. Compute (SM):** Autoregressive token decoding is typically memory-bandwidth bound. Independent sparklines make it obvious whether your GPUs are stalled on memory bus transfers or saturated on compute cores.
- **Kernel Paging Delays:** Spike in swap page-in/out rates on Linux indicates that model weights, KV caches, or system buffers have spilled into disk swap, causing catastrophic latency spikes.
- **PCIe Saturation:** High PCIe RX/TX throughput highlights pipeline stalls during prompt prefill or tensor exchange across multi-GPU risers.

---

## Controls

| Key | Action |
| --- | --- |
| `q` / `Esc` / `Ctrl-C` | Exit cleanly and restore terminal raw mode |
| `+` / `=` | Increase refresh rate (shorter interval) |
| `-` | Decrease refresh rate (longer interval) |
| `Space` / `p` | Pause or resume real-time sampling |
| `r` | Reset rolling sparkline history |

---

## Privacy & Threat Model

- **Local-First & Read-Only:** `llamatop` queries local loopback sockets (`127.0.0.1`) and native kernel/driver interfaces. It never issues synthetic inference requests, modifies process priorities, or changes driver settings.
- **Zero Prompt/Output Inspection:** `llamatop` monitors quantitative operational telemetry only. It never captures, logs, or inspects model prompts, completion text, or weights.
- **No Remote Telemetry:** Contains no third-party trackers, analytics, or background phone-home routines.

See [SECURITY.md](SECURITY.md) for vulnerability reporting and detailed security boundaries.

---

## Acknowledgments & Lineage

`llamatop` is a Linux-native, multi-GPU evolution inspired by [`mlxtop`](https://github.com/maximpri/mlxtop), originally created by **Maxim Priezjev** for Apple Silicon. 

While the architecture has been rewritten for Linux kernels, NVIDIA NVML bindings, and `llama.cpp` server environments, `llamatop` retains `mlxtop`'s core philosophy: clean TUI presentation, zero-overhead observability, causal temporal tracking, and truthful telemetry without fabricated averages.

---

## License

Distributed under the [MIT License](LICENSE). Third-party dependencies retain their upstream licenses as documented in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
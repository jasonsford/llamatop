# Security policy

## Supported versions

Security fixes are made on the current default branch and included in the next
release. Older releases may not receive backports while the project is maintained
by a small team.

## Reporting a vulnerability

Please do not disclose a suspected vulnerability in a public issue, pull request, or
discussion. Use GitHub's private vulnerability reporting feature from the
repository's **Security** tab. If private reporting is unavailable, open a public
issue containing no sensitive details and request a private contact channel.

When reporting, include:
- The affected `llamatop` version or commit hash.
- Host OS and kernel version (e.g., Ubuntu 24.04, Linux 6.8).
- NVIDIA driver version and CUDA toolkit release.
- Hardware configuration (GPU models and topology).
- Detailed reproduction steps, expected impact, and any proposed mitigation.

Remove API keys, prompts, model output, internal IP addresses, and private hostnames
before submitting any logs or screenshots.

## Security boundaries and threat model

- **Read-Only Telemetry:** `llamatop` is designed as a read-only monitoring tool.
  It queries NVIDIA hardware counters via NVML and polls local HTTP/Prometheus
  endpoints exposed by `llama-server`. It does not execute model inference,
  modify driver parameters, or alter process execution priorities.
- **Local Loopback Communication:** By default, `llamatop` communicates with
  `llama-server` instances across local loopback interfaces (`127.0.0.1` / `localhost`).
  Unencrypted HTTP is used exclusively for local IPC. If monitoring across remote
  networks, always run traffic through an encrypted tunnel (e.g., WireGuard,
  Tailscale, or an SSH tunnel).
- **Process and Memory Inspection:** Hardware metrics are gathered through the
  official NVIDIA Management Library (`NVML`), and host metrics are derived from
  standard Linux virtual filesystems (`/proc/stat`, `/proc/vmstat`). `llamatop`
  inspects process command-line arguments solely to map PIDs to server ports and
  model names; it does not read process memory or inference context buffers.
- **Data Privacy:** `llamatop` never logs or captures prompt text, completion tokens,
  or model generation outputs. Only quantitative performance telemetry (tokens per
  second, latency, context usage counters, VRAM allocation, and utilization rates)
  is surfaced.

Reports will be acknowledged and evaluated on a best-effort basis. Please allow
sufficient time for validation and patch preparation prior to public disclosure.
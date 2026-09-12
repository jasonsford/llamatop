---
name: Bug report
about: Report incorrect telemetry, NVML errors, crashes, or TUI display problems
title: ""
labels: "bug"
assignees: ""
---

## What happened?

Describe the observed problem and what you expected to see instead.

## Reproduction steps

1. Start `llama-server` with command: `...`
2. Run `llamatop` with arguments: `...`
3. Observe: `...`

## Environment & Hardware

- **llamatop version:** (e.g., `llamatop --version` or git commit hash)
- **Host OS & Kernel:** (e.g., Ubuntu 24.04 LTS / Linux 6.8, or Windows 11)
- **GPU Model(s):** (e.g., 1x RTX 3080, 2x RTX 4060 Ti 16GB)
- **NVIDIA Driver & CUDA version:** (e.g., Driver 550.54.14, CUDA 12.4)
- **Terminal Emulator & Geometry:** (e.g., Alacritty, WezTerm, Windows Terminal; rows x cols)
- **llama-server version & build:** (e.g., build b3500)
- **llama-server flags:** (e.g., `--metrics`, `--slots`, `--port 8080`)

## Relevant output or terminal logs

Include terminal error output, panic tracebacks, or sanitized screenshots illustrating the display issue.

*Note: Redact internal IP addresses, network paths, API tokens, and prompt contents before submitting. Report suspected security vulnerabilities using [SECURITY.md](SECURITY.md) instead of this form.*
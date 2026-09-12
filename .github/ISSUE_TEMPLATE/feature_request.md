---
name: Feature request
about: Suggest an improvement, new telemetry metric, or provider adapter
title: ""
labels: "enhancement"
assignees: ""
---

## User problem

What monitoring, telemetry, or hardware observation problem are you trying to solve?

## Proposed behavior

Describe the proposed change, how it surfaces in the TUI, and any alternative designs you considered.

## Telemetry source & interfaces

If proposing new hardware metrics or an inference provider adapter (e.g., vLLM, Ollama, TGI):
- Link to the official API, Prometheus metrics endpoint, or NVML/kernel counter documentation.
- How should unavailable, unprivileged, or stale data be presented (`—`, dimmed, or omitted)?
- Does collecting this metric incur noticeable CPU/GPU overhead during active inference?

*Note: Do not include API keys, private hostnames, or confidential prompts/logs in your request.*
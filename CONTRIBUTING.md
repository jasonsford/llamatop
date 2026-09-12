# Contributing

Thank you for helping improve llamatop. Bug reports, documentation, telemetry
fixtures, and focused implementation changes are all useful contributions.

## Before opening a change

Run the local checks:

~~~sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release
~~~

If developing on a non-Linux workstation (such as Windows), verify that
Linux-specific code paths compile cleanly:

~~~sh
cargo check --target x86_64-unknown-linux-gnu
~~~

For UI changes, test running the interactive dashboard:

~~~sh
cargo run
~~~

Verify that keyboard inputs (`q`, `Esc`) terminate cleanly, raw terminal mode is
disabled, and the cursor is properly restored.

## Design boundaries

- Keep telemetry collection lightweight so monitoring does not compete with
  active model inference for CPU or memory bandwidth.
- Never display stale provider values as live telemetry.
- Show telemetry provenance and status whenever an instance API or NVML is
  unavailable.
- Use fixed-width stepped time-series traces (sparklines) for indicator
  history. One displayed step maps to one captured sample; new samples enter on
  the right and old samples leave on the left once the viewport is full. Keep
  smoothing causal; never recalculate history using future samples.
- Treat color as a secondary cue. Every visible severity or operating condition
  must also have a semantic label that does not depend on color perception.
- Keep unavailable counters as unavailable; do not substitute a healthy zero.
- Preserve visual hierarchy: serving throughput and active slots are primary;
  GPU compute, VRAM, PCIe bus, and host paging are supporting evidence.
- Use responsive disclosure instead of squeezing cards or columns. Ensure cards
  remain readable across multi-GPU setups.
- Keep provider network scraping out of native telemetry collectors. Process
  signatures and command lines may be used only for lightweight discovery and
  PID-to-model attribution.

## Pull requests

Describe the user-visible behavior, the platform assumptions, and the checks
you ran. Include a terminal screenshot for substantial layout changes when
possible.

Do not include API keys, prompts, private logs, or other workload data in an
issue, fixture, or screenshot. Report suspected vulnerabilities according to
[SECURITY.md](SECURITY.md).

## Contribution license

By submitting a contribution, you confirm that you have the right to provide
it and agree that it is licensed under the project's MIT License.
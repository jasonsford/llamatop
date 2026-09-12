## Change

Describe the problem, the architectural motivation, and the user-visible behavior changes.

## Environment & Hardware Tested

- **Host OS:** (e.g., Ubuntu 24.04 LTS / Windows 11 Pro)
- **GPUs:** (e.g., RTX 3080, RTX 4060, etc.)
- **NVIDIA Driver & CUDA:** (e.g., 550.54.14 / CUDA 12.4)
- **Inference Runtime:** (e.g., `llama-server` b3500)

## Validation

List the checks you executed locally across your workstation and target environment. Include terminal output or ASCII snippets for substantial UI changes, ensuring sensitive data (hostnames, tokens, model prompts) is redacted.

- [ ] Formatting (`cargo fmt --all -- --check`) passes cleanly.
- [ ] Clippy (`cargo clippy --all-targets -- -D warnings`) passes with zero warnings.
- [ ] Tests and release builds (`cargo test --all-targets && cargo build --release`) succeed.
- [ ] Target Linux cross-check (`cargo check --target x86_64-unknown-linux-gnu`) passes if developing on Windows/macOS.
- [ ] Interactive UI test (`cargo run`) exits cleanly on `q`/`Esc` and restores terminal raw mode.
- [ ] Documentation and CLI flags reflect changed behavior.
- [ ] I have the right to contribute this work under the MIT License.
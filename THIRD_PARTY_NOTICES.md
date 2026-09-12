# Third-party software notices

llamatop's first-party source is licensed under the MIT License. It also links
Rust crates that remain under their respective upstream licenses.

## Direct dependencies

| Package | Declared license | Purpose |
| --- | --- | --- |
| clap | MIT OR Apache-2.0 | Command-line argument parsing |
| crossterm | MIT | Cross-platform terminal raw mode and event stream handling |
| nvml-wrapper | MIT OR Apache-2.0 | NVIDIA Management Library (NVML) bindings for Linux telemetry |
| ratatui | MIT | Terminal user interface rendering and widgets |
| reqwest | MIT OR Apache-2.0 | Asynchronous HTTP client for `llama-server` endpoints |
| serde | MIT OR Apache-2.0 | Serialization/deserialization framework |
| serde_json | MIT OR Apache-2.0 | JSON response parsing for slot discovery |
| sysinfo | MIT | Host CPU, RAM, and Swap memory telemetry |
| tokio | MIT | Asynchronous runtime and timer scheduling |

The complete dependency graph and exact resolved versions are recorded in `Cargo.lock`.
Continuous integration audits every dependency in the tree using `cargo-deny`.
The accepted SPDX licenses and exceptions configured for this project are:

- MIT
- Apache-2.0
- Apache-2.0 WITH LLVM-exception
- BSD-2-Clause
- BSD-3-Clause
- BSL-1.0
- ISC
- Unicode-3.0
- Unicode-DFS-2016
- Unlicense
- Zlib

No third-party dependency is relicensed under MIT by this project. Source packages
downloaded by Cargo contain their original copyright notices and license texts.
Anyone distributing compiled binary releases of `llamatop` must preserve and
distribute the license texts and notices required by the dependencies included
within that binary; this summary does not replace those upstream legal texts.

License policies are enforced via `deny.toml`. Run `cargo deny check licenses`
whenever dependencies are updated and review `Cargo.lock` prior to tagging a
release.
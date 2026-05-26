# devinspect Tasks

## Phase 1: Core Architecture & CLI Setup
- [x] Set up Rust project for Single Binary CLI.
- [x] Configure modular plugin-ready architecture (Collectors, Analyzers, Renderers, Exporters).
- [x] Ensure cross-platform build pipelines (Linux, macOS, Windows).
- [x] Set up Config Management (TOML file support for themes, default outputs, enabled modules).
- [x] Implement detailed Logging (`--verbose`, `--debug`, `--trace`, timestamped/rotating logs).

## Phase 2: Functional Collectors (System & Hardware)
- [x] Implement System Summary (hostname, OS, kernel, arch, uptime, CPU, memory, storage, GPU, IP, network interfaces, virtualization).
- [x] Implement Hardware Inspection (CPU, RAM, storage, thermal data, BIOS, memory pressure, filesystem).
- [x] Implement GPU Diagnostics (vendor, model, VRAM, CUDA, Vulkan, OpenCL, TensorRT, ROCm, driver versions).
- [x] Implement Network Diagnostics (interfaces, MAC, IPs, DNS, routes, gateway, MTU, ports, latency, DNS timing, firewall).

## Phase 3: Specialized Environment Validation
- [ ] Implement Rust Environment Diagnostics (rustc, cargo, targets, clippy, rustfmt, cross-compilation).
- [ ] Implement Embedded Systems Diagnostics (Raspberry Pi/Jetson detection, serial, GPIO, SPI/I2C, CAN, USB).
- [ ] Implement Automotive Systems Diagnostics (SocketCAN, vcan, CAN interfaces, SOME/IP, V2X, automotive SDKs).
- [ ] Implement AI/ML Stacks Validation (Python, CUDA, cuDNN, TensorRT, Ollama, PyTorch/TF GPU, ONNX, llama.cpp).

## Phase 4: Diagnostics, UI & Reporting
- [ ] Implement Doctor Mode (analyze dependencies, GPU, compilers, Docker, networking, output severity/remediations).
- [ ] Build Terminal Dashboard using ratatui (live panels, gauges, tables, charts, keyboard navigation, theming).
- [ ] Implement AI-Friendly Output (`--llm` flag for root-cause hints and remediation suggestions).
- [ ] Implement Exporting Options (JSON, YAML, Markdown, HTML, PDF).

## Phase 5: Security & Performance Hardening
- [ ] Optimize for fast execution and offline-first operation.
- [ ] Verify zero-daemon, read-only minimal privilege execution.
- [ ] Verify deterministic structured data outputs.
- [ ] Implement Sensitive Data Handling (MAC/IP redaction, anonymized exports).
- [ ] Ensure strictly no cloud dependency or background telemetry.

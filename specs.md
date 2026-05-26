# devinspect Specifications

Based on the ideation document, here is the categorized list of features for `devinspect`.

## Functional Features

### System & Hardware Diagnostics
- **System Summary**: Display hostname, OS, kernel, architecture, uptime, CPU, memory, storage, GPU, public IP, network interfaces, and virtualization support.
- **Hardware Inspection**: Inspect CPU, RAM, storage, thermal data, BIOS information, memory pressure, and filesystem information.
- **GPU Diagnostics**: Detect GPU vendor, model, VRAM, CUDA, Vulkan, OpenCL, TensorRT, ROCm, and driver versions. Integrates with tools like `nvidia-smi` and `intel_gpu_top`.
- **Network Diagnostics**: Analyze interfaces, MAC addresses, IPv4/IPv6, DNS, routes, gateway, MTU, public IP, listening ports, latency tests, DNS timing, and firewall status.

### Specialized Environment Validation
- **Rust Environment**: Validate `rustc`/`cargo` versions, installed targets, clippy, rustfmt, rust-analyzer, cross-compilation readiness, and musl support.
- **Embedded Systems**: Detect devices (Raspberry Pi, Jetson, BeagleBone, etc.) and analyze serial devices, GPIO, SPI/I2C, CAN devices, and USB devices.
- **Automotive Systems**: Diagnose SocketCAN, vcan, CAN interfaces, SOME/IP tools, V2X dependencies, and automotive SDKs.
- **AI/ML Stacks**: Validate Python versions, CUDA, cuDNN, TensorRT, Ollama, PyTorch/TensorFlow GPU support, ONNX runtime, and llama.cpp.

### Diagnostics & UI
- **Doctor Mode**: Analyze and output issues, severity, probable causes, and remediations for packages, dependencies, GPU, compilers, Docker, Rust, and networking.
- **Terminal Dashboard**: A live, beautifully rendered terminal interface (ratatui-based) featuring live panels, colors, collapsible sections, gauges, tables, charts, sparklines, and keyboard navigation.
- **AI-Friendly Output**: Use the `--llm` flag to generate root-cause hints, remediation suggestions, and environment snapshots.

### Reporting & Configuration
- **Exporting Options**: Generate reports in JSON, YAML, Markdown, HTML, and PDF formats.
- **Config Management**: Support a TOML configuration file (`~/.config/devinspect/config.toml`) for managing themes, default outputs, and enabled modules.
- **Logging**: Detailed logging capabilities via `--verbose`, `--debug`, and `--trace` flags, with support for timestamped, rotating logs, and execution timing.

## Non-Functional Features

### Architecture & Execution
- **Single Binary CLI**: Compiled as a standalone executable in Rust.
- **Cross-Platform**: Fully supports all major platforms including Linux (Ubuntu, Debian, Fedora, Arch, Yocto, Alpine), macOS, and Windows.
- **Performance**: Extremely fast execution optimized for offline-first operation.
- **Zero-Daemon**: Operates strictly on-demand without background agents or kernel-level monitoring daemons.
- **Modular Design**: Features a highly decoupled, plugin-ready architecture consisting of Data Collectors, Analyzers, Renderers, and Exporters.

### Security & Privacy
- **Secure-by-Default**: Executes with minimal privilege and performs read-only diagnostics.
- **No Cloud Dependency**: Functions without remote telemetry servers, SaaS dashboards, or data exfiltration.
- **Sensitive Data Handling**: Built-in support for redaction (e.g., MAC/IP masking) and anonymized exports.

### User Experience (UX)
- **Design Philosophy**: Visually dense, readable, low-noise, and highly responsive.
- **Determinism**: Ensures consistent and deterministic structured data pipeline outputs.
- **Theming**: Includes native terminal themes like dark, light, hacker, solarized, and minimal.

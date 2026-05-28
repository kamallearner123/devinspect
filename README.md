# `devinspect(1)` — Developer & System Diagnostics Man Page

```text
DEVINSPECT(1)                 User Commands                 DEVINSPECT(1)
```

## NAME
**devinspect** — Modern offline-first system diagnostics, environment validation, and AI-ready diagnostics for developers, embedded engineers, and automated agents.

---

## SYNOPSIS
```bash
devinspect [GLOBAL_FLAGS] <COMMAND> [COMMAND_ARGS]
```

---

## DESCRIPTION
**devinspect** is a single-binary, secure-by-default engineering diagnostics platform. Unlike cosmetic system-fetch utilities, it is built to inspect, normalize, and validate system environments, hardware capabilities, and modern development stacks (such as Rust, embedded systems, automotive networks, and AI/ML pipelines). 

It resolves the fragmentation of traditional system commands by collecting diagnostic details, executing automated severity analysis, and exporting reports in highly readable CLI layouts, interactive TUI dashboards, or structured formats.

---

## TARGET AUDIENCE
* **AI & LLM Agents:** Autonomous coding agents or RAG pipelines that need to validate if environment tools (compilers, CUDA, local LLMs) are present or functioning.
* **Systems & Embedded Engineers:** Developers debugging hardware configurations, connected USB interfaces, system resource distribution, or network traffic spikes.
* **Automotive Engineers:** Teams working with SocketCAN, virtual CAN (vcan), or custom communication buses.
* **DevOps & Platform Teams:** Professionals requiring deterministic, offline-first diagnostics in server environments, CI/CD pipelines, and local machines.

---

## THE PROBLEM: THE FRAGMENTATION OF TRADITIONAL DEBUGGING
Traditional systems debugging is plagued by a major issue: **tool fragmentation and memory overload**. 

To get a complete view of a machine's health, a developer must remember and correctly parse the output of dozens of low-level tools, many of which use inconsistent flags and output formats:
* **Memory & Performance:** `top`, `htop`, `vmstat`, `free -h`
* **Process Details:** `ps aux`, `pidstat`, `lsof`, `strace`
* **Hardware & BIOS:** `lscpu`, `lshw`, `dmidecode`, `sensors`
* **Network Sockets & Packets:** `netstat`, `ss`, `ip a`, `ifconfig`, `tcpdump`
* **AI & GPUs:** `nvidia-smi`, `trtexec`, `nvcc --version`
* **USB & Buses:** `lsusb`, `lspci`

### Core Issues with Existing Approaches:
1. **Difficult to Remember:** Remembering exact CLI flags for dozens of utilities is mentally taxing.
2. **Parsing Overhead:** Each command prints a completely different format (tab-separated, column-oriented, unstructured blocks). Writing custom regex parsers is fragile and tedious.
3. **Dependency Hell:** Many low-level commands (e.g., `nvidia-smi` or `lsusb`) might be missing on minimal system installs, leading to broken scripts and failed validations.
4. **Token Heavy for AI:** Asking an AI agent to run 15 separate commands to understand your workspace consumes excessive API tokens and risks execution error.

---

## COMMAND USAGE

### 1. `devinspect summary`
Provides a unified, high-level overview of the operating system, kernel, host architecture, CPU, memory, storage volumes, GPU availability, and network state.
```bash
devinspect summary
```

### 2. `devinspect hardware`
Detailed physical hardware diagnostics including motherboard, BIOS version, memory pressure, active mount points, filesystems, and temperature sensors.
```bash
devinspect hardware
```

### 3. `devinspect network`
Comprehensive networking snapshot listing MAC addresses, active IP addresses, gateway, MTU, DNS servers, open local ports, and latency statistics.
```bash
devinspect network
```

### 4. `devinspect usb`
Lists connected USB devices classified by category (Hubs, Keyboards, Audio, Mass Storage, etc.) structured as a hierarchical bus tree.
```bash
devinspect usb
```

### 5. `devinspect top`
Monitors and displays the top 10 CPU-consuming and memory-consuming processes.
* **One-shot check:**
  ```bash
  devinspect top
  ```
* **Continuous sampling (e.g., for 5 seconds):**
  ```bash
  devinspect top -t 5
  ```

### 6. `devinspect packets`
Samples and measures network packet transmission/reception rates, drops, and errors over a configured time window.
```bash
devinspect packets -t 3   # Samples rate over 3 seconds
```

### 7. `devinspect pidstat`
Inspects a specific running process (PID) in surgical detail, showing resource consumption charts, file descriptors, open network sockets, thread lists, and child processes.
```bash
devinspect pidstat -p <PID> -t <SECONDS>
```

### 8. `devinspect ai`
Validates installed AI/ML tools, PyTorch, TensorFlow, CUDA, cuDNN, TensorRT, ONNX Runtime, and local LLM services (e.g., Ollama, llama.cpp).
```bash
devinspect ai
```

### 9. `devinspect doctor`
Runs a suite of sanity checks across all local compilers, system packages, and GPU acceleration drivers. It reports issues with graded severity levels along with remediation steps.
```bash
devinspect doctor
```

---

## AI & LLM AGENT INTEGRATION

`devinspect` is built with a first-class philosophy of **AI-driven automation**. AI agents can use this tool to bypass raw console commands and retrieve structured, clean, and normalized inputs.

### 1. How to Start This to Help AI Tools (Bootstrapping Context)
When an AI agent (such as a coding assistant, agentic workflow, or terminal companion) starts a task in a workspace, it needs to quickly assess the machine's capabilities. 

Rather than executing several individual discovery commands, the AI agent can bootstrap its system context with a single run:

```bash
devinspect doctor --llm
```

This commands output is formatted in **LLM-optimized, high-density Markdown**. It omits terminal escape sequences and ANSI color blocks, presenting a highly structured list of issues, system versions, and potential fixes that the AI can immediately ingest to guide its workflow.

### 2. How AI Tools Can Call `devinspect` as a Structured API
Even though `devinspect` is a CLI binary, **it serves as a local machine-readable API endpoint** for AI tools. By utilizing the global format flags, the AI can call commands and parse the output directly inside its agent environment.

#### **JSON API Calls:**
An AI tool can invoke:
```bash
devinspect summary --json
devinspect ai --json
devinspect doctor --json
```
**JSON API Output Example (`devinspect ai --json`):**
```json
{
  "python": {
    "installed": true,
    "version": "3.10.12"
  },
  "cuda": {
    "installed": true,
    "nvcc_version": "12.2",
    "driver_version": "535.129.03",
    "max_supported_cuda": "12.2"
  },
  "cudnn": {
    "installed": true,
    "version": "8.9.2"
  },
  "tensorrt": {
    "installed": false,
    "version": null
  },
  "ollama": {
    "installed": true,
    "version": "0.1.32",
    "service_running": true
  },
  "pytorch": {
    "installed": true,
    "version": "2.1.2+cu121",
    "cuda_available": true,
    "cuda_version": "12.1"
  },
  "tensorflow": {
    "installed": false,
    "version": null,
    "gpu_available": null
  },
  "onnxruntime": {
    "installed": true,
    "version": "1.16.3",
    "providers": ["CPUExecutionProvider", "CUDAExecutionProvider"]
  },
  "llamacpp": {
    "installed": false,
    "version": null
  }
}
```

#### **YAML API Calls:**
For agents that prefer lower-token YAML formats, they can invoke:
```bash
devinspect summary --yaml
devinspect ai --yaml
```

### **Flow of AI Agent Interaction:**
```text
┌─────────────────┐                  ┌─────────────┐                  ┌──────────────────┐
│  AI Agent / LLM │ ───( Exec CLI )──> │ devinspect  │ ──( Collects )──> │ OS, GPU, Net,    │
│  Function Call  │ <──( JSON API )─── │ CLI Engine  │ <─( & Normalizes) │ Hardware, AI/ML  │
└─────────────────┘                  └─────────────┘                  └──────────────────┘
         │
         ▼
[ AI parses JSON data ] ──> [ Makes deterministic code or setup decisions ]
```

---

## GLOBAL FLAGS
* `-v, --verbose` — Enable verbose logging.
* `-d, --debug` — Enable debug level tracing.
* `-T, --trace` — Enable deep trace logging.
* `--json` — Print output as machine-readable structured JSON.
* `--yaml` — Print output as structured YAML.
* `--llm` — Generate high-density Markdown formatted specifically for LLM system context.

---

## CONFIGURATION
`devinspect` reads default behaviors and setup values from:
`~/.config/devinspect/config.toml`

### **Example Configuration (`config.toml`):**
```toml
default_theme = "dark"
enable_telemetry = false
ollama_host = "127.0.0.1:11434"
```

---

## EXIT STATUS
* **0** — Success.
* **1** — Generic CLI or validation failure.
* **2** — Critical system resource or diagnostic error.

---

## BUGS & CONTRIBUTIONS
Report bugs or suggest custom collectors via the project issue tracker.

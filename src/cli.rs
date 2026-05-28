use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "devinspect - Modern diagnostics for developers, AI systems, and embedded platforms.",
    long_about = "devinspect is a modern engineering diagnostics and environment inspection tool.\n\
It provides detailed machine diagnostics, environment validation, and AI-friendly reports\n\
for modern developer ecosystems including Rust, AI/ML stacks, and embedded Linux.\n\
\n\
Built for offline-first, secure-by-default execution.",
    after_help = "EXAMPLES:\n  devinspect summary          # Display high-level system summary\n  devinspect hardware         # Detailed hardware diagnostics\n  devinspect --debug doctor   # Run doctor mode with debug logging\n  devinspect summary --json   # Export summary in JSON format\n  devinspect top              # Top CPU/memory processes\n  devinspect top -t 10        # Monitor processes for 10 seconds\n  devinspect packets          # Network packet statistics\n  devinspect packets -t 5     # Sample packets over 5 seconds\n  devinspect usb              # List connected USB devices"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Enable debug logging
    #[arg(short, long, global = true)]
    pub debug: bool,

    /// Enable trace logging
    #[arg(short = 'T', long, global = true)]
    pub trace: bool,

    /// Output in JSON format
    #[arg(long, global = true)]
    pub json: bool,

    /// Output in YAML format
    #[arg(long, global = true)]
    pub yaml: bool,

    /// Generate AI-friendly report with root-cause hints and remediation suggestions
    #[arg(long, global = true)]
    pub llm: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Validate AI/ML stacks (Python, CUDA, cuDNN, TensorRT, Ollama, PyTorch, TensorFlow, ONNX, llama.cpp)
    Ai,
    /// Display high-level system summary
    Summary,
    /// Detailed hardware diagnostics
    Hardware,
    /// Doctor mode for analyzing issues
    Doctor,
    /// Display network diagnostics
    Network,
    /// Launch the interactive colorful terminal dashboard
    Dashboard,
    /// Run all core diagnostic collectors
    All,

    /// Show top CPU/memory-consuming processes with usage summary
    ///
    /// Lists the 10 heaviest processes sorted by CPU and Memory.
    /// Use -t to continuously monitor for N seconds (refreshes every second).
    Top {
        /// Monitor for N seconds (e.g. -t 5 watches for 5 seconds)
        #[arg(short = 't', long = "time", value_name = "SECS")]
        monitor_secs: Option<u64>,
    },

    /// Show network packet statistics (RX/TX bytes, rates, errors, drops)
    ///
    /// Reads /proc/net/dev and samples over a configurable window.
    /// Use -t to set the sampling interval in seconds (default: 1).
    Packets {
        /// Sampling window in seconds for rate calculation (e.g. -t 5)
        #[arg(short = 't', long = "time", value_name = "SECS")]
        interval_secs: Option<u64>,
    },

    /// List connected USB devices with type classification and bus tree
    ///
    /// Requires `lsusb` (usbutils). Classifies devices by type
    /// (hub, keyboard, storage, audio, etc.) and shows a per-bus tree.
    Usb,

    /// Modern pidstat tool: Inspect a specific process (PID) in extreme detail
    ///
    /// Analyzes a PID's CPU/Mem charts, thread list, open files, sockets, disk, syscalls, and children.
    Pidstat {
        /// The PID of the process to inspect
        #[arg(short = 'p', long = "pid", value_name = "PID")]
        pid: u32,

        /// Monitoring window in seconds (default: 3)
        #[arg(short = 't', long = "time", value_name = "SECS", default_value = "3")]
        duration_secs: u64,
    },
}

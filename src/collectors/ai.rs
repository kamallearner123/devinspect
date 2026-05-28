use serde::{Serialize, Deserialize};
use owo_colors::OwoColorize;
use std::process::Command;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PythonInfo {
    pub installed: bool,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CudaInfo {
    pub installed: bool,
    pub nvcc_version: Option<String>,
    pub driver_version: Option<String>,
    pub max_supported_cuda: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CudnnInfo {
    pub installed: bool,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TensorRtInfo {
    pub installed: bool,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub service_running: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PyTorchInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub cuda_available: Option<bool>,
    pub cuda_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TensorFlowInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub gpu_available: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnnxRuntimeInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub providers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlamaCppInfo {
    pub installed: bool,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiDiagnosticReport {
    pub python: PythonInfo,
    pub cuda: CudaInfo,
    pub cudnn: CudnnInfo,
    pub tensorrt: TensorRtInfo,
    pub ollama: OllamaInfo,
    pub pytorch: PyTorchInfo,
    pub tensorflow: TensorFlowInfo,
    pub onnxruntime: OnnxRuntimeInfo,
    pub llamacpp: LlamaCppInfo,
}

fn run_command(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .ok()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    
    if output.status.success() && !stdout.is_empty() {
        Some(stdout)
    } else if !stdout.is_empty() {
        Some(stdout)
    } else if !stderr.is_empty() {
        Some(stderr)
    } else {
        None
    }
}

pub fn collect_ai_diagnostics() -> AiDiagnosticReport {
    // 1. Python info
    let mut python_installed = false;
    let mut python_version = None;
    if let Some(out) = run_command("python3", &["--version"]) {
        python_installed = true;
        // e.g. "Python 3.10.12"
        python_version = Some(out.replace("Python ", "").trim().to_string());
    } else if let Some(out) = run_command("python", &["--version"]) {
        python_installed = true;
        python_version = Some(out.replace("Python ", "").trim().to_string());
    }

    // 2. CUDA info
    let mut cuda_installed = false;
    let mut nvcc_version = None;
    let mut driver_version = None;
    let mut max_supported_cuda = None;

    if let Some(out) = run_command("nvcc", &["--version"]) {
        cuda_installed = true;
        // Parse version from nvcc output: e.g. "... release 12.2, V12.2.140"
        if let Some(idx) = out.find("release ") {
            let part = &out[idx + 8..];
            if let Some(comma_idx) = part.find(',') {
                nvcc_version = Some(part[..comma_idx].trim().to_string());
            }
        }
    } else if Path::new("/usr/local/cuda/bin/nvcc").exists() {
        cuda_installed = true;
        if let Some(out) = run_command("/usr/local/cuda/bin/nvcc", &["--version"]) {
            if let Some(idx) = out.find("release ") {
                let part = &out[idx + 8..];
                if let Some(comma_idx) = part.find(',') {
                    nvcc_version = Some(part[..comma_idx].trim().to_string());
                }
            }
        }
    }

    // Check nvidia-smi
    if let Some(out) = run_command("nvidia-smi", &[]) {
        cuda_installed = true;
        // Parse Driver Version and CUDA Version
        // Typical line: "NVIDIA-SMI 535.129.03             Driver Version: 535.129.03     CUDA Version: 12.2     "
        for line in out.lines() {
            if line.contains("Driver Version:") && line.contains("CUDA Version:") {
                if let Some(drv_idx) = line.find("Driver Version:") {
                    let drv_part = &line[drv_idx + 15..];
                    let drv_end = drv_part.find(' ').unwrap_or(drv_part.len());
                    driver_version = Some(drv_part[..drv_end].trim().to_string());
                }
                if let Some(cuda_idx) = line.find("CUDA Version:") {
                    let cuda_part = &line[cuda_idx + 13..];
                    let cuda_end = cuda_part.find(' ').unwrap_or(cuda_part.len());
                    max_supported_cuda = Some(cuda_part[..cuda_end].trim().to_string());
                }
                break;
            }
        }
    }

    // 3. cuDNN info
    let mut cudnn_installed = false;
    let mut cudnn_version = None;
    
    // Check common header paths for CUDNN version
    let cudnn_paths = [
        "/usr/include/cudnn_version.h",
        "/usr/local/cuda/include/cudnn_version.h",
        "/usr/include/x86_64-linux-gnu/cudnn_version.h",
    ];

    for path in &cudnn_paths {
        if Path::new(path).exists() {
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                let mut major = None;
                let mut minor = None;
                let mut patch = None;
                for line_res in reader.lines() {
                    if let Ok(line) = line_res {
                        if line.contains("#define CUDNN_MAJOR") {
                            major = line.split_whitespace().last().map(|s| s.to_string());
                        } else if line.contains("#define CUDNN_MINOR") {
                            minor = line.split_whitespace().last().map(|s| s.to_string());
                        } else if line.contains("#define CUDNN_PATCHLEVEL") {
                            patch = line.split_whitespace().last().map(|s| s.to_string());
                        }
                    }
                }
                if let (Some(maj), Some(min), Some(pat)) = (major, minor, patch) {
                    cudnn_installed = true;
                    cudnn_version = Some(format!("{}.{}.{}", maj, min, pat));
                    break;
                }
            }
        }
    }

    if !cudnn_installed {
        // Try searching for libcudnn shared object
        let lib_paths = [
            "/usr/lib/x86_64-linux-gnu/libcudnn.so",
            "/usr/local/cuda/lib64/libcudnn.so",
        ];
        for path in &lib_paths {
            if Path::new(path).exists() {
                cudnn_installed = true;
                // Follow symlink or run file to get version if possible, else mark installed
                cudnn_version = Some("Detected (Shared Library present)".to_string());
                break;
            }
        }
    }

    // 4. TensorRT info
    let mut trt_installed = false;
    let mut trt_version = None;
    if let Some(out) = run_command("trtexec", &["--version"]) {
        trt_installed = true;
        // Parse e.g. "TensorRT v8.6.1"
        trt_version = Some(out.trim().to_string());
    } else {
        let trt_libs = [
            "/usr/lib/x86_64-linux-gnu/libnvinfer.so",
            "/usr/local/TensorRT/lib/libnvinfer.so",
        ];
        for path in &trt_libs {
            if Path::new(path).exists() {
                trt_installed = true;
                trt_version = Some("Detected (libnvinfer present)".to_string());
                break;
            }
        }
    }

    // 5. Ollama info
    let mut ollama_installed = false;
    let mut ollama_version = None;
    let mut ollama_running = false;
    if let Some(out) = run_command("ollama", &["--version"]) {
        ollama_installed = true;
        // e.g. "ollama version is 0.1.32" or "ollama version 0.1.32"
        let ver = out.replace("ollama version is", "").replace("ollama version", "").trim().to_string();
        ollama_version = Some(ver);
    }
    // Resolve Ollama address (can be custom IP/port read from ~/.config/devinspect/config.toml)
    let ollama_host = crate::config::AppConfig::load()
        .ok()
        .and_then(|c| c.ollama_host)
        .unwrap_or_else(|| "127.0.0.1:11434".to_string());
        
    use std::net::ToSocketAddrs;
    if let Ok(mut addrs) = ollama_host.to_socket_addrs() {
        if let Some(addr) = addrs.next() {
            if TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok() {
                ollama_running = true;
                if !ollama_installed {
                    ollama_installed = true; // Service is running, so it is installed
                }
            }
        }
    }

    // 6. PyTorch info
    let mut torch_installed = false;
    let mut torch_version = None;
    let mut torch_cuda = None;
    let mut torch_cuda_ver = None;
    if let Some(out) = run_command("python3", &["-c", "import torch; print(f'OK|{torch.__version__}|{torch.cuda.is_available()}|{torch.version.cuda}')"]) {
        if out.starts_with("OK|") {
            let parts: Vec<&str> = out.split('|').collect();
            if parts.len() >= 4 {
                torch_installed = true;
                torch_version = Some(parts[1].to_string());
                torch_cuda = Some(parts[2] == "True");
                torch_cuda_ver = Some(parts[3].to_string());
            }
        }
    }

    // 7. TensorFlow info
    let mut tf_installed = false;
    let mut tf_version = None;
    let mut tf_gpu = None;
    if let Some(out) = run_command("python3", &["-c", "import tensorflow as tf; gpus=tf.config.list_physical_devices('GPU'); print(f'OK|{tf.__version__}|{len(gpus) > 0}')"]) {
        if out.starts_with("OK|") {
            let parts: Vec<&str> = out.split('|').collect();
            if parts.len() >= 3 {
                tf_installed = true;
                tf_version = Some(parts[1].to_string());
                tf_gpu = Some(parts[2] == "True");
            }
        }
    }

    // 8. ONNX Runtime info
    let mut ort_installed = false;
    let mut ort_version = None;
    let mut ort_providers = Vec::new();
    if let Some(out) = run_command("python3", &["-c", "import onnxruntime as ort; print(f'OK|{ort.__version__}|{\",\".join(ort.get_available_providers())}')"]) {
        if out.starts_with("OK|") {
            let parts: Vec<&str> = out.split('|').collect();
            if parts.len() >= 3 {
                ort_installed = true;
                ort_version = Some(parts[1].to_string());
                ort_providers = parts[2].split(',').map(|s| s.trim().to_string()).collect();
            }
        }
    }

    // 9. llama.cpp info
    let mut llama_installed = false;
    let mut llama_version = None;
    if let Some(out) = run_command("llama-cli", &["--version"]) {
        llama_installed = true;
        llama_version = Some(out.trim().to_string());
    } else if let Some(out) = run_command("llama-server", &["--version"]) {
        llama_installed = true;
        llama_version = Some(out.trim().to_string());
    } else {
        // check common binaries
        for bin in &["llama-cli", "llama-server", "llama-bench"] {
            if run_command("which", &[bin]).is_some() {
                llama_installed = true;
                llama_version = Some("Installed (binary in PATH)".to_string());
                break;
            }
        }
    }

    AiDiagnosticReport {
        python: PythonInfo { installed: python_installed, version: python_version },
        cuda: CudaInfo { installed: cuda_installed, nvcc_version, driver_version, max_supported_cuda },
        cudnn: CudnnInfo { installed: cudnn_installed, version: cudnn_version },
        tensorrt: TensorRtInfo { installed: trt_installed, version: trt_version },
        ollama: OllamaInfo { installed: ollama_installed, version: ollama_version, service_running: ollama_running },
        pytorch: PyTorchInfo { installed: torch_installed, version: torch_version, cuda_available: torch_cuda, cuda_version: torch_cuda_ver },
        tensorflow: TensorFlowInfo { installed: tf_installed, version: tf_version, gpu_available: tf_gpu },
        onnxruntime: OnnxRuntimeInfo { installed: ort_installed, version: ort_version, providers: ort_providers },
        llamacpp: LlamaCppInfo { installed: llama_installed, version: llama_version },
    }
}

pub fn display_ai(json: bool, yaml: bool, llm: bool) {
    let report = collect_ai_diagnostics();

    if json {
        match serde_json::to_string_pretty(&report) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error serializing AI diagnostics: {}", e),
        }
        return;
    }

    if yaml {
        match serde_yaml::to_string(&report) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error serializing AI diagnostics: {}", e),
        }
        return;
    }

    if llm {
        // Output in pure, high-density, concise Markdown without fancy formatting or sparklines
        println!("# AI/ML Diagnostic Report (LLM-Optimized)");
        println!();
        println!("- Python: installed={}, version={}", report.python.installed, report.python.version.as_deref().unwrap_or("None"));
        println!("- CUDA: installed={}, nvcc={}, driver={}, max_cuda={}", 
            report.cuda.installed, 
            report.cuda.nvcc_version.as_deref().unwrap_or("None"),
            report.cuda.driver_version.as_deref().unwrap_or("None"),
            report.cuda.max_supported_cuda.as_deref().unwrap_or("None")
        );
        println!("- cuDNN: installed={}, version={}", report.cudnn.installed, report.cudnn.version.as_deref().unwrap_or("None"));
        println!("- TensorRT: installed={}, version={}", report.tensorrt.installed, report.tensorrt.version.as_deref().unwrap_or("None"));
        println!("- Ollama: installed={}, version={}, running={}", report.ollama.installed, report.ollama.version.as_deref().unwrap_or("None"), report.ollama.service_running);
        println!("- PyTorch: installed={}, version={}, cuda_available={}, torch_cuda_ver={}", 
            report.pytorch.installed, 
            report.pytorch.version.as_deref().unwrap_or("None"),
            report.pytorch.cuda_available.unwrap_or(false),
            report.pytorch.cuda_version.as_deref().unwrap_or("None")
        );
        println!("- TensorFlow: installed={}, version={}, gpu_available={}", 
            report.tensorflow.installed, 
            report.tensorflow.version.as_deref().unwrap_or("None"),
            report.tensorflow.gpu_available.unwrap_or(false)
        );
        println!("- ONNX Runtime: installed={}, version={}, providers={:?}", 
            report.onnxruntime.installed, 
            report.onnxruntime.version.as_deref().unwrap_or("None"),
            report.onnxruntime.providers
        );
        println!("- llama.cpp: installed={}, version={}", report.llamacpp.installed, report.llamacpp.version.as_deref().unwrap_or("None"));
        return;
    }

    // Default beautiful visual standard TUI/CLI output
    println!("\n{}", "╭──────────────────────────────────────────────────╮".dimmed());
    println!("{} {}", "│".dimmed(), "🧠 AI/ML & LLM STACK VALIDATION".bold().bright_magenta());
    println!("{}", "╰──────────────────────────────────────────────────╯".dimmed());

    // 1. Core Runtimes
    println!("\n{}", "  [ Core Runtimes ]".bold().bright_cyan());
    if report.python.installed {
        println!("  ├─ 🐍 {:<12} {}", "Python3:".bright_green(), report.python.version.as_deref().unwrap_or("Unknown").bold().bright_yellow());
    } else {
        println!("  ├─ ❌ {:<12} {}", "Python3:".bright_red(), "Not Detected".dimmed());
    }

    // 2. Hardware Acceleration (CUDA/NVIDIA)
    println!("\n{}", "  [ Hardware Acceleration ]".bold().bright_cyan());
    if report.cuda.installed {
        println!("  ├─ 🟢 {:<12} Installed", "NVIDIA GPU:".bright_green());
        if let Some(drv) = &report.cuda.driver_version {
            println!("  ├─ 💾 {:<12} {}", "Driver Ver:".bright_green(), drv.bright_yellow());
        }
        if let Some(nvcc) = &report.cuda.nvcc_version {
            println!("  ├─ ⚙️  {:<12} CUDA Compiler v{}", "NVCC:".bright_green(), nvcc.bright_cyan().bold());
        } else {
            println!("  ├─ ⚙️  {:<12} {}", "NVCC:".bright_green(), "Not Detected (runtime driver only)".dimmed());
        }
        if let Some(max_cuda) = &report.cuda.max_supported_cuda {
            println!("  ├─ 🚀 {:<12} Supports up to CUDA {}", "Driver CUDA:".bright_green(), max_cuda.bright_magenta());
        }
    } else {
        println!("  ├─ ❌ {:<12} {}", "NVIDIA CUDA:".bright_red(), "No GPU driver or compiler detected".dimmed());
    }

    if report.cudnn.installed {
        println!("  ├─ 🧠 {:<12} v{}", "cuDNN:".bright_green(), report.cudnn.version.as_deref().unwrap_or("Detected").bright_yellow());
    } else {
        println!("  ├─ ❌ {:<12} {}", "cuDNN:".bright_red(), "Not Detected".dimmed());
    }

    if report.tensorrt.installed {
        println!("  └─ ⚡ {:<12} {}", "TensorRT:".bright_green(), report.tensorrt.version.as_deref().unwrap_or("Detected").bright_yellow());
    } else {
        println!("  └─ ❌ {:<12} {}", "TensorRT:".bright_red(), "Not Detected".dimmed());
    }

    // 3. Local LLM Services
    println!("\n{}", "  [ Local LLM Orchestration ]".bold().bright_cyan());
    if report.ollama.installed {
        let running_status = if report.ollama.service_running {
            "Active & Listening (127.0.0.1:11434)".bright_green().bold().to_string()
        } else {
            "Installed, but Service Offline".bright_red().to_string()
        };
        println!("  ├─ 🦙 {:<12} v{} ({})", "Ollama:".bright_green(), report.ollama.version.as_deref().unwrap_or("Unknown").bright_yellow(), running_status);
    } else {
        println!("  ├─ ❌ {:<12} {}", "Ollama:".bright_red(), "Not Detected".dimmed());
    }

    if report.llamacpp.installed {
        println!("  └─ 📦 {:<12} {}", "llama.cpp:", report.llamacpp.version.as_deref().unwrap_or("Detected (binaries in PATH)").bright_yellow());
    } else {
        println!("  └─ ❌ {:<12} {}", "llama.cpp:", "Not Detected (llama-cli / llama-server missing)".dimmed());
    }

    // 4. ML Frameworks
    println!("\n{}", "  [ Deep Learning & Inference Frameworks ]".bold().bright_cyan());
    
    if report.pytorch.installed {
        let cuda_support = match report.pytorch.cuda_available {
            Some(true) => format!("GPU Enabled (CUDA {})", report.pytorch.cuda_version.as_deref().unwrap_or("Unknown")).bright_green().bold().to_string(),
            _ => "CPU Only".bright_yellow().to_string(),
        };
        println!("  ├─ 🔥 {:<12} v{} ➜ {}", "PyTorch:", report.pytorch.version.as_deref().unwrap_or("Unknown").bright_yellow(), cuda_support);
    } else {
        println!("  ├─ ❌ {:<12} {}", "PyTorch:", "Not Installed".dimmed());
    }

    if report.tensorflow.installed {
        let gpu_support = match report.tensorflow.gpu_available {
            Some(true) => "GPU Enabled".bright_green().bold().to_string(),
            _ => "CPU Only".bright_yellow().to_string(),
        };
        println!("  ├─ 🍊 {:<12} v{} ➜ {}", "TensorFlow:", report.tensorflow.version.as_deref().unwrap_or("Unknown").bright_yellow(), gpu_support);
    } else {
        println!("  ├─ ❌ {:<12} {}", "TensorFlow:", "Not Installed".dimmed());
    }

    if report.onnxruntime.installed {
        let provs = report.onnxruntime.providers.join(", ");
        println!("  └─ 🌀 {:<12} v{} ➜ Providers: [{}]", "ONNX Runtime:", report.onnxruntime.version.as_deref().unwrap_or("Unknown").bright_yellow(), provs.bright_cyan());
    } else {
        println!("  └─ ❌ {:<12} {}", "ONNX Runtime:", "Not Installed".dimmed());
    }
    println!();
}

use serde::{Serialize, Deserialize};
use owo_colors::OwoColorize;
use std::process::Command;
use std::net::TcpStream;
use std::time::Duration;
use std::net::ToSocketAddrs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Severity {
    INFO,
    WARNING,
    ERROR,
    CRITICAL,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::INFO => write!(f, "INFO"),
            Severity::WARNING => write!(f, "WARNING"),
            Severity::ERROR => write!(f, "ERROR"),
            Severity::CRITICAL => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiagnosticIssue {
    pub id: String,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub probable_cause: String,
    pub remediation: String,
}

fn check_command_exists(cmd: &str) -> bool {
    Command::new("which").arg(cmd).output().map(|o| o.status.success()).unwrap_or(false)
}

pub fn run_doctor_analysis() -> Vec<DiagnosticIssue> {
    let mut issues = Vec::new();

    // 1. Check compiler dependencies
    if !check_command_exists("gcc") && !check_command_exists("clang") {
        issues.push(DiagnosticIssue {
            id: "missing_compiler".to_string(),
            severity: Severity::CRITICAL,
            title: "C/C++ Compiler Not Found".to_string(),
            description: "No native compiler (gcc/clang) was detected on the system PATH.".to_string(),
            probable_cause: "Development essential packages are not installed on this host.".to_string(),
            remediation: "Install development tools: 'sudo apt install build-essential' (Debian/Ubuntu) or 'sudo dnf groupinstall \"Development Tools\"' (RedHat/Fedora).".to_string(),
        });
    }

    if !check_command_exists("rustc") {
        issues.push(DiagnosticIssue {
            id: "missing_rust".to_string(),
            severity: Severity::INFO,
            title: "Rust Compiler (rustc) Not Found".to_string(),
            description: "Rust compiler and cargo are missing from your system PATH.".to_string(),
            probable_cause: "Rust toolchain is either not installed or not active in the current shell environment.".to_string(),
            remediation: "Install Rust via rustup: 'curl --proto \"=https\" --tlsv1.2 -sSf https://sh.rustup.rs | sh' and source ~/.cargo/env".to_string(),
        });
    }

    // 2. Check Docker / Container permissions
    if check_command_exists("docker") {
        let output = Command::new("docker").arg("ps").output();
        match output {
            Ok(out) if !out.status.success() => {
                let err_str = String::from_utf8_lossy(&out.stderr);
                if err_str.contains("permission denied") {
                    issues.push(DiagnosticIssue {
                        id: "docker_permission_denied".to_string(),
                        severity: Severity::ERROR,
                        title: "Docker Socket Permission Denied".to_string(),
                        description: "Docker command-line tool is installed, but your current user cannot access /var/run/docker.sock.".to_string(),
                        probable_cause: "Your user is not a member of the 'docker' system group.".to_string(),
                        remediation: "Add your user to the docker group: 'sudo usermod -aG docker $USER' and restart your session.".to_string(),
                    });
                }
            }
            Err(_) => {}
            _ => {}
        }
    } else {
        issues.push(DiagnosticIssue {
            id: "missing_docker".to_string(),
            severity: Severity::INFO,
            title: "Docker Container Runtime Missing".to_string(),
            description: "No local docker or podman execution binary detected.".to_string(),
            probable_cause: "Container tooling is not configured or installed on this workstation.".to_string(),
            remediation: "Install Docker Engine from: https://docs.docker.com/engine/install/".to_string(),
        });
    }

    // 3. Check GPU / AI & ML Stack integration
    let has_nvidia_gpu = check_command_exists("nvidia-smi");
    
    // Check if CUDA resides in standard directories but compiler nvcc is missing
    if has_nvidia_gpu {
        if !check_command_exists("nvcc") && !Path::new("/usr/local/cuda/bin/nvcc").exists() {
            issues.push(DiagnosticIssue {
                id: "missing_cuda_nvcc".to_string(),
                severity: Severity::WARNING,
                title: "CUDA Compiler (NVCC) Not in PATH".to_string(),
                description: "NVIDIA driver is active, but CUDA Compiler (nvcc) is not accessible.".to_string(),
                probable_cause: "CUDA Toolkit is either not installed, or /usr/local/cuda/bin is missing from your PATH environment variable.".to_string(),
                remediation: "Add CUDA to PATH: 'export PATH=/usr/local/cuda/bin:$PATH' and 'export LD_LIBRARY_PATH=/usr/local/cuda/lib64:$LD_LIBRARY_PATH' in ~/.bashrc or ~/.zshrc.".to_string(),
            });
        }
    }

    // Check PyTorch/TensorFlow runtime status (GPU vs CPU mismatch)
    if has_nvidia_gpu {
        // Test PyTorch CUDA availability
        let py_torch_test = Command::new("python3")
            .args(&["-c", "import torch; print(torch.cuda.is_available())"])
            .output();
        if let Ok(out) = py_torch_test {
            let res = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if res.contains("False") {
                issues.push(DiagnosticIssue {
                    id: "pytorch_cpu_only_with_gpu".to_string(),
                    severity: Severity::ERROR,
                    title: "PyTorch Using CPU on GPU Host".to_string(),
                    description: "An NVIDIA GPU and driver are present, but PyTorch is running in CPU-only mode.".to_string(),
                    probable_cause: "PyTorch was installed without CUDA acceleration support (likely via standard pip install of default CPU wheels).".to_string(),
                    remediation: "Reinstall PyTorch with CUDA: 'pip3 install torch torchvision --index-url https://download.pytorch.org/whl/cu121'".to_string(),
                });
            }
        }
    }

    // 4. Local LLM Service (Ollama Port checking)
    if check_command_exists("ollama") {
        let ollama_host = crate::config::AppConfig::load()
            .ok()
            .and_then(|c| c.ollama_host)
            .unwrap_or_else(|| "127.0.0.1:11434".to_string());

        let mut is_port_open = false;
        if let Ok(mut addrs) = ollama_host.to_socket_addrs() {
            if let Some(addr) = addrs.next() {
                is_port_open = TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok();
            }
        }
        
        if !is_port_open {
            issues.push(DiagnosticIssue {
                id: "ollama_service_stopped".to_string(),
                severity: Severity::WARNING,
                title: "Ollama Local Service Stopped".to_string(),
                description: format!("Ollama CLI is installed, but the local background server on {} is unreachable.", ollama_host),
                probable_cause: "Ollama background service is either stopped or has not been started yet.".to_string(),
                remediation: "Start the Ollama background service: 'systemctl start ollama' (Linux) or run the 'ollama serve' command manually.".to_string(),
            });
        }
    }

    // 5. Network access & DNS check
    let has_dns = ("github.com", 80).to_socket_addrs()
        .map(|mut addrs| addrs.next().is_some())
        .unwrap_or(false);

    if !has_dns {
        // check if 8.8.8.8 (IP directly) connects to distinguish DNS vs total disconnect
        let has_ip_con = TcpStream::connect_timeout(
            &"8.8.8.8:53".parse().unwrap(),
            Duration::from_millis(1000)
        ).is_ok();

        if has_ip_con {
            issues.push(DiagnosticIssue {
                id: "dns_resolution_failure".to_string(),
                severity: Severity::ERROR,
                title: "DNS Resolution Fails".to_string(),
                description: "IP routing is online, but hostname resolution is currently failing on this machine.".to_string(),
                probable_cause: "Configured local DNS nameservers (in /etc/resolv.conf) are unresponsive or misconfigured.".to_string(),
                remediation: "Check your DNS settings or temporarily add Google DNS to '/etc/resolv.conf': 'nameserver 8.8.8.8'".to_string(),
            });
        } else {
            issues.push(DiagnosticIssue {
                id: "network_fully_offline".to_string(),
                severity: Severity::ERROR,
                title: "Host System Fully Offline".to_string(),
                description: "Cannot connect to external IPs or resolve domain names.".to_string(),
                probable_cause: "Network interface is disconnected, Wi-Fi is disabled, or gateway routing is down.".to_string(),
                remediation: "Verify physical connections, run 'ping -c 3 192.168.1.1' to check your local gateway, or run 'ip route' to verify interfaces.".to_string(),
            });
        }
    }

    issues
}

pub fn display_doctor(json: bool, yaml: bool, llm: bool) {
    let issues = run_doctor_analysis();

    if json {
        match serde_json::to_string_pretty(&issues) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error serializing Doctor report: {}", e),
        }
        return;
    }

    if yaml {
        match serde_yaml::to_string(&issues) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error serializing Doctor report: {}", e),
        }
        return;
    }

    if llm {
        println!("# System Diagnostic Analysis (LLM-Optimized)");
        println!();
        if issues.is_empty() {
            println!("No environmental issues detected. System health check is green.");
            return;
        }
        for (i, issue) in issues.iter().enumerate() {
            println!("## [{}] Issue (Severity: {}): {}", i + 1, issue.severity, issue.title);
            println!("- **Description:** {}", issue.description);
            println!("- **Probable Cause:** {}", issue.probable_cause);
            println!("- **Remediation:** {}", issue.remediation);
            println!();
        }
        return;
    }

    println!("\n{}", "╭──────────────────────────────────────────────────╮".dimmed());
    println!("{} {}", "│".dimmed(), "🩺 SYSTEM DOCTOR MODE: DIAGNOSTICS".bold().bright_cyan());
    println!("{}", "╰──────────────────────────────────────────────────╯".dimmed());
    println!("  ⏳  Running environment and dependency diagnostics...\n");

    if issues.is_empty() {
        println!("  🎉  {}", "All diagnostic checks passed successfully! Your workstation is in perfect shape.".bright_green().bold());
        println!();
        return;
    }

    for issue in &issues {
        let (icon, color_title) = match issue.severity {
            Severity::CRITICAL => ("🚨", issue.title.bold().bright_red().to_string()),
            Severity::ERROR => ("❌", issue.title.bold().bright_red().to_string()),
            Severity::WARNING => ("⚠️ ", issue.title.bold().bright_yellow().to_string()),
            Severity::INFO => ("ℹ️ ", issue.title.bold().bright_blue().to_string()),
        };

        println!("  {}  [ {} ] {}", icon, issue.severity.to_string().bold(), color_title);
        println!("  ├─ 📝 {:<14} {}", "Description:", issue.description);
        println!("  ├─ ⚙️  {:<14} {}", "Probable Cause:", issue.probable_cause.dimmed());
        println!("  └─ 🛠️  {:<14} {}", "Remediation:", issue.remediation.bright_green().bold());
        println!();
    }
}

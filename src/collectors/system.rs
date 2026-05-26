use sysinfo::System;
use owo_colors::OwoColorize;
use std::process::Command;

pub fn display_summary() {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    println!("\n{}", "╭──────────────────────────────────────────────────╮".dimmed());
    println!("{} {}", "│".dimmed(), "⚡ SYSTEM SUMMARY".bold().bright_cyan());
    println!("{}", "╰──────────────────────────────────────────────────╯".dimmed());
    
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    println!("  💻 {:<12} {}", "Hostname:".bright_green(), hostname.bright_blue().bold());
    
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_ver = System::os_version().unwrap_or_else(|| "".to_string());
    println!("  🐧 {:<12} {} {}", "OS:".bright_green(), os_name.bright_blue(), os_ver.bright_blue());
    
    let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    println!("  ⚙️  {:<12} {}", "Kernel:".bright_green(), kernel.bright_blue());
    
    let uptime = System::uptime();
    let uptime_str = format!("{}h {}m {}s", uptime / 3600, (uptime % 3600) / 60, uptime % 60);
    println!("  ⏱️  {:<12} {}", "Uptime:".bright_green(), uptime_str.bright_yellow());
    
    let total_memory = sys.total_memory() / 1024 / 1024;
    let used_memory = sys.used_memory() / 1024 / 1024;
    println!("  🧠 {:<12} {} MB used / {} MB total", "Memory:".bright_green(), used_memory.bright_magenta(), total_memory.bright_cyan());
    
    println!("  🚀 {:<12} {} Cores Active", "CPU Cores:".bright_green(), sys.cpus().len().to_string().bright_blue().bold());
    println!();
}

pub fn display_hardware() {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    println!("\n{}", "╭──────────────────────────────────────────────────╮".dimmed());
    println!("{} {}", "│".dimmed(), "🔧 HARDWARE DIAGNOSTICS".bold().bright_blue());
    println!("{}", "╰──────────────────────────────────────────────────╯".dimmed());
    
    println!("\n{}", "  [ CPUs ]".bold().bright_magenta());
    for (i, cpu) in sys.cpus().iter().enumerate() {
        println!("  ├─ 🚀 CPU {:<2}: {} ({} MHz)", i.to_string().bright_blue(), cpu.brand().bright_yellow(), cpu.frequency().to_string().bright_cyan());
    }
    
    let total_memory = sys.total_memory() / 1024 / 1024;
    let used_memory = sys.used_memory() / 1024 / 1024;
    let total_swap = sys.total_swap() / 1024 / 1024;
    let used_swap = sys.used_swap() / 1024 / 1024;
    
    println!("\n{}", "  [ Memory ]".bold().bright_magenta());
    println!("  ├─ 🧠 {:<6} {} MB / {} MB", "RAM:".bright_green(), used_memory.bright_yellow(), total_memory.bright_cyan());
    println!("  ├─ 🔄 {:<6} {} MB / {} MB", "Swap:".bright_green(), used_swap.bright_yellow(), total_swap.bright_cyan());
    
    println!("\n{}", "  [ Storage Disks ]".bold().bright_magenta());
    let disks = sysinfo::Disks::new_with_refreshed_list();
    for disk in &disks {
        let total_gb = disk.total_space() / 1024 / 1024 / 1024;
        let available_gb = disk.available_space() / 1024 / 1024 / 1024;
        println!("  ├─ 💾 {:<18} ({:<5}) ➜ {} GB available / {} GB total", 
            disk.name().to_string_lossy().bright_yellow().bold(), 
            disk.file_system().to_string_lossy().bright_blue(), 
            available_gb.bright_green(), 
            total_gb.bright_cyan());
    }

    println!("\n{}", "  [ Graphics Processors (GPU) ]".bold().bright_magenta());
    let mut gpu_found = false;
    if let Ok(output) = Command::new("lspci").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.to_lowercase().contains("vga") || line.to_lowercase().contains("3d controller") {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    println!("  ├─ 🎮 {}", parts[1].trim().bright_blue().bold());
                    gpu_found = true;
                }
            }
        }
    }
    
    if let Ok(output) = Command::new("nvidia-smi").arg("--query-gpu=name,memory.total").arg("--format=csv,noheader").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if !line.trim().is_empty() {
                println!("  ├─ 🟢 NVIDIA: {}", line.trim().bright_green().bold());
                gpu_found = true;
            }
        }
    }

    if !gpu_found {
        println!("  └─ {}", "No discrete GPU detected or required tools missing.".dimmed().italic());
    }
    println!();
}

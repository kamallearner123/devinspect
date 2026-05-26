use sysinfo::{System, Pid, ProcessesToUpdate};
use owo_colors::OwoColorize;
use std::fs;
use std::time::Duration;

// PidDetails is kept here as reference documentation for the struct schema
/*
#[derive(Debug)]
struct PidDetails {
    pid: u32,
    name: String,
    status: String,
    parent_pid: Option<u32>,
    cpu_usage: f32,
    memory_bytes: u64,
    virtual_memory_bytes: u64,
    thread_count: usize,
    open_fd_count: usize,
    read_bytes: u64,
    write_bytes: u64,
    rchar: u64,
    wchar: u64,
    start_time: u64,
    exe_path: String,
    cmdline: Vec<String>,
}
*/

fn read_proc_file(pid: u32, file: &str) -> Option<String> {
    fs::read_to_string(format!("/proc/{}/{}", pid, file)).ok()
}

fn get_open_fds(pid: u32) -> usize {
    let fd_dir = format!("/proc/{}/fd", pid);
    fs::read_dir(fd_dir).map(|dir| dir.count()).unwrap_or(0)
}

fn get_io_stats(pid: u32) -> (u64, u64, u64, u64) {
    if let Some(content) = read_proc_file(pid, "io") {
        let mut rchar = 0;
        let mut wchar = 0;
        let mut read_bytes = 0;
        let mut write_bytes = 0;
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                let val: u64 = parts[1].parse().unwrap_or(0);
                match parts[0] {
                    "rchar:" => rchar = val,
                    "wchar:" => wchar = val,
                    "read_bytes:" => read_bytes = val,
                    "write_bytes:" => write_bytes = val,
                    _ => {}
                }
            }
        }
        (rchar, wchar, read_bytes, write_bytes)
    } else {
        (0, 0, 0, 0)
    }
}

fn get_threads(pid: u32) -> Vec<(u32, String)> {
    let mut threads = Vec::new();
    let task_dir = format!("/proc/{}/task", pid);
    if let Ok(entries) = fs::read_dir(task_dir) {
        for entry in entries.flatten() {
            if let Ok(tid_str) = entry.file_name().into_string() {
                if let Ok(tid) = tid_str.parse::<u32>() {
                    let comm = fs::read_to_string(format!("/proc/{}/task/{}/comm", pid, tid))
                        .map(|c| c.trim().to_string())
                        .unwrap_or_else(|_| "unknown".to_string());
                    threads.push((tid, comm));
                }
            }
        }
    }
    threads
}

fn get_network_sockets(pid: u32) -> usize {
    // Count open sockets in /proc/PID/net/tcp and udp (or looking at fd targets like socket:[12345])
    let mut count = 0;
    let fd_dir = format!("/proc/{}/fd", pid);
    if let Ok(entries) = fs::read_dir(fd_dir) {
        for entry in entries.flatten() {
            if let Ok(target) = fs::read_link(entry.path()) {
                let target_str = target.to_string_lossy();
                if target_str.contains("socket:") || target_str.contains("TCP") || target_str.contains("UDP") {
                    count += 1;
                }
            }
        }
    }
    count
}

fn get_open_files_list(pid: u32) -> Vec<String> {
    let mut files = Vec::new();
    let fd_dir = format!("/proc/{}/fd", pid);
    if let Ok(entries) = fs::read_dir(fd_dir) {
        for entry in entries.flatten() {
            if let Ok(target) = fs::read_link(entry.path()) {
                let target_str = target.to_string_lossy().to_string();
                if target_str.starts_with('/') && !files.contains(&target_str) {
                    files.push(target_str);
                }
            }
        }
    }
    files
}

fn get_child_processes(pid: u32, sys: &System) -> Vec<(u32, String)> {
    let mut children = Vec::new();
    for (p, proc) in sys.processes() {
        if let Some(ppid) = proc.parent() {
            if ppid.as_u32() == pid {
                children.push((p.as_u32(), proc.name().to_string_lossy().to_string()));
            }
        }
    }
    children
}

fn get_syscall(pid: u32) -> String {
    read_proc_file(pid, "syscall")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "N/A (requires privileges or process finished)".to_string())
}

fn get_crashes_signals(pid: u32) -> String {
    if let Some(status) = read_proc_file(pid, "status") {
        let mut sig_blk = "N/A";
        let mut sig_ign = "N/A";
        let mut sig_cgt = "N/A";
        for line in status.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let val = parts[1].trim();
                match key {
                    "SigBlk" => sig_blk = val,
                    "SigIgn" => sig_ign = val,
                    "SigCgt" => sig_cgt = val,
                    _ => {}
                }
            }
        }
        format!("Blocked: {}, Ignored: {}, Caught: {}", sig_blk.bright_yellow(), sig_ign.bright_green(), sig_cgt.bright_cyan())
    } else {
        "N/A".to_string()
    }
}

fn human_bytes(b: u64) -> String {
    if b >= 1_073_741_824 {
        format!("{:.2} GB", b as f64 / 1_073_741_824.0)
    } else if b >= 1_048_576 {
        format!("{:.2} MB", b as f64 / 1_048_576.0)
    } else if b >= 1_024 {
        format!("{:.2} KB", b as f64 / 1_024.0)
    } else {
        format!("{} B", b)
    }
}

fn make_sparkline(history: &[f32]) -> String {
    let sparks = [" ", " ", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    let max = history.iter().cloned().fold(0.0f32, f32::max);
    if max == 0.0 {
        return " ".repeat(history.len());
    }
    history.iter().map(|&v| {
        let idx = ((v / max) * 8.0).round() as usize;
        sparks[idx.min(8)]
    }).collect()
}

pub fn inspect_pid(pid_val: u32, duration_secs: u64) {
    println!("\n{}", "╭──────────────────────────────────────────────────────────────╮".dimmed());
    println!("{} {}", "│".dimmed(), format!("🔍 PID DETAILED DIAGNOSTICS: {}", pid_val).bold().bright_cyan());
    println!("{}", "╰──────────────────────────────────────────────────────────────╯".dimmed());

    let sys_pid = Pid::from(pid_val as usize);
    let mut sys = System::new_all();
    sys.refresh_all();

    if sys.process(sys_pid).is_none() {
        println!("\n  ❌  {}", format!("Process with PID {} not found or access denied.", pid_val).bright_red().bold());
        println!();
        return;
    }

    println!("\n  ⏳  Analyzing process activity over {} seconds...", duration_secs.to_string().bright_blue().bold());

    // Record historical data for graphs and timeline
    let mut cpu_history = Vec::new();
    let mut mem_history = Vec::new();
    let mut read_history = Vec::new();
    let mut write_history = Vec::new();

    let interval = Duration::from_millis(500);
    let steps = (duration_secs * 2).max(1);

    let mut last_io = get_io_stats(pid_val);

    for _ in 0..steps {
        sys.refresh_processes(ProcessesToUpdate::All, true);
        if let Some(proc) = sys.process(sys_pid) {
            cpu_history.push(proc.cpu_usage());
            mem_history.push((proc.memory() / 1024 / 1024) as f32);
        } else {
            cpu_history.push(0.0);
            mem_history.push(0.0);
        }

        let io = get_io_stats(pid_val);
        let read_diff = io.2.saturating_sub(last_io.2);
        let write_diff = io.3.saturating_sub(last_io.3);
        read_history.push(read_diff as f32 / 1024.0); // KB
        write_history.push(write_diff as f32 / 1024.0); // KB
        last_io = io;

        std::thread::sleep(interval);
    }

    // Final refresh
    sys.refresh_all();
    let proc = match sys.process(sys_pid) {
        Some(p) => p,
        None => {
            println!("  ❌  {}","Process exited during inspection.".bright_red().bold());
            return;
        }
    };

    let p_pid = proc.parent().map(|p| p.as_u32());
    let exe_path = proc.exe().map(|e| e.to_string_lossy().to_string()).unwrap_or_else(|| "N/A".to_string());
    let cmdline = proc.cmd().iter().map(|c| c.to_string_lossy().to_string()).collect::<Vec<String>>();
    let threads = get_threads(pid_val);
    let open_fds = get_open_fds(pid_val);
    let net_sockets = get_network_sockets(pid_val);
    let open_files = get_open_files_list(pid_val);
    let children = get_child_processes(pid_val, &sys);
    let syscall = get_syscall(pid_val);
    let signals = get_crashes_signals(pid_val);
    let (rchar, wchar, read_bytes, write_bytes) = get_io_stats(pid_val);

    let avg_cpu: f32 = cpu_history.iter().sum::<f32>() / cpu_history.len() as f32;
    let max_cpu: f32 = cpu_history.iter().cloned().fold(0.0f32, f32::max);
    let latest_mem = proc.memory();

    // ─── Render Process Hierarchy & Details ───
    println!("\n{}", "  [ PROCESS INFO ]".bold().bright_magenta());
    println!("  ├─ 📝 {:<16} {}", "Name:".bright_green(), proc.name().to_string_lossy().bright_yellow().bold());
    println!("  ├─ 🆔 {:<16} {}", "PID:".bright_green(), pid_val.to_string().bright_blue());
    println!("  ├─ 👥 {:<16} {}", "Parent PID:".bright_green(), p_pid.map(|p| p.to_string()).unwrap_or_else(|| "None (Root)".to_string()).bright_blue());
    println!("  ├─ 🚦 {:<16} {:?}", "Status:".bright_green(), proc.status());
    println!("  ├─ ⚙️  {:<16} {}", "Executable:".bright_green(), exe_path.bright_blue());
    if !cmdline.is_empty() {
        println!("  └─ 🚀 {:<16} {}", "Command Line:".bright_green(), cmdline.join(" ").dimmed());
    }

    // ─── Render CPU Graph & Stats ───
    println!("\n{}", "  [ CPU ACTIVITY ]".bold().bright_magenta());
    println!("  ├─ 📊 {:<16} {}", "CPU Sparkline:".bright_green(), make_sparkline(&cpu_history).bright_cyan());
    println!("  ├─ 📉 {:<16} {:.1}%", "Current Usage:".bright_green(), proc.cpu_usage().bright_yellow());
    println!("  ├─ 📈 {:<16} {:.1}%", "Peak CPU Usage:".bright_green(), max_cpu.bright_yellow().bold());
    println!("  └─ ⏱️  {:<16} {:.1}%", "Average CPU:".bright_green(), avg_cpu.bright_cyan());

    // ─── Render Memory Graph & Stats ───
    println!("\n{}", "  [ MEMORY DIAGNOSTICS ]".bold().bright_magenta());
    println!("  ├─ 📊 {:<16} {}", "Memory Sparkline:".bright_green(), make_sparkline(&mem_history).bright_cyan());
    println!("  ├─ 🧠 {:<16} {}", "RSS (Physical):".bright_green(), human_bytes(latest_mem).bright_yellow());
    println!("  ├─ 🔄 {:<16} {}", "Virtual Memory:".bright_green(), human_bytes(proc.virtual_memory()).bright_cyan());
    println!("  └─ 📈 {:<16} {}", "Peak Resident:".bright_green(), human_bytes(mem_history.iter().cloned().fold(0.0f32, f32::max) as u64 * 1024 * 1024).bright_cyan().bold());

    // ─── Threads Info ───
    println!("\n{}", "  [ THREADS ]".bold().bright_magenta());
    println!("  ├─ 🔢 {:<16} {}", "Total Threads:".bright_green(), threads.len().to_string().bright_blue().bold());
    if !threads.is_empty() {
        let list_len = threads.len().min(5);
        println!("  ├─ 📋 {:<16}", "Active Thread list (top 5):".bright_green());
        for t in threads.iter().take(list_len) {
            println!("  │   ├── TID: {:<8} Comm: {}", t.0.to_string().bright_blue(), t.1.bright_yellow());
        }
        if threads.len() > 5 {
            println!("  │   └── ... and {} more threads", threads.len() - 5);
        }
    } else {
        println!("  └─ ❌ No thread details available in /proc");
    }

    // ─── Open Files & Network Sockets ───
    println!("\n{}", "  [ STORAGE & NETWORK IO ]".bold().bright_magenta());
    println!("  ├─ 📁 {:<16} {}", "Open FDs:".bright_green(), open_fds.to_string().bright_blue());
    println!("  ├─ 🌐 {:<16} {}", "Network Sockets:".bright_green(), net_sockets.to_string().bright_blue().bold());
    println!("  ├─ 📥 {:<16} {} (total: {})", "Read Bandwidth:".bright_green(), human_bytes(rchar), human_bytes(read_bytes).bright_cyan());
    println!("  └─ 📤 {:<16} {} (total: {})", "Write Bandwidth:".bright_green(), human_bytes(wchar), human_bytes(write_bytes).bright_cyan());

    if !open_files.is_empty() {
        let list_len = open_files.len().min(5);
        println!("  └── 📂 {:<16}", "Open Files (sample):".bright_green());
        for file in open_files.iter().take(list_len) {
            println!("      ├── {}", file.bright_yellow());
        }
        if open_files.len() > 5 {
            println!("      └── ... and {} more files", open_files.len() - 5);
        }
    }

    // ─── Syscalls & Signal/Crash Diagnostics ───
    println!("\n{}", "  [ CRASHES & SECURITY SYSCALLS ]".bold().bright_magenta());
    println!("  ├─ ⚙️  {:<16} {}", "Current Syscall:".bright_green(), syscall.bright_yellow());
    println!("  └─ 🔔 {:<16} {}", "Signals:".bright_green(), signals);

    // ─── Child Processes ───
    println!("\n{}", "  [ CHILD PROCESSES ]".bold().bright_magenta());
    if !children.is_empty() {
        println!("  ├── Total Active Children: {}", children.len().to_string().bright_blue().bold());
        for (i, child) in children.iter().enumerate() {
            let conn = if i == children.len() - 1 { "└──" } else { "├──" };
            println!("  {} PID: {:<8} Name: {}", conn, child.0.to_string().bright_blue(), child.1.bright_yellow());
        }
    } else {
        println!("  └── No child processes active.");
    }

    // ─── GPU & Timeline / Historical Charts summary ───
    println!("\n{}", "  [ PERFORMANCE TIMELINE & GPU ]".bold().bright_magenta());
    println!("  ├─ 📅 {:<16} Started at system uptime + {}s", "Timeline:".bright_green(), proc.start_time().to_string().bright_cyan());
    println!("  ├─ 🎮 {:<16} {}", "GPU Usage:".bright_green(), "Integrated/No GPU bindings active for this PID".dimmed());
    println!("  └─ 📈 {:<16} Measured over {} steps (half-second resolution)", "Historical Logs:".bright_green(), steps.to_string().bright_blue());
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_bytes() {
        assert_eq!(human_bytes(500), "500 B");
        assert_eq!(human_bytes(1024), "1.00 KB");
        assert_eq!(human_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(human_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_make_sparkline() {
        let history = vec![0.0, 2.5, 5.0, 7.5, 10.0];
        let spark = make_sparkline(&history);
        assert_eq!(spark.chars().count(), 5);
        assert_eq!(spark, " ▂▄▆█"); // 0.0 -> " ", 2.5 -> "▂", 5.0 -> "▄", 7.5 -> "▆", 10.0 -> "█"

        let all_zero = vec![0.0, 0.0, 0.0];
        assert_eq!(make_sparkline(&all_zero), "   ");
    }
}

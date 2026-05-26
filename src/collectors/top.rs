use sysinfo::{System, ProcessesToUpdate};
use owo_colors::OwoColorize;
use std::time::Duration;

/// Render a usage bar  ████░░░░  with colour based on percentage
fn bar(pct: f32, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    let empty  = width - filled;
    let bar_str = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
    if pct >= 80.0 {
        bar_str.bright_red().to_string()
    } else if pct >= 50.0 {
        bar_str.bright_yellow().to_string()
    } else {
        bar_str.bright_green().to_string()
    }
}

/// sysinfo requires TWO refreshes separated by a sleep to compute CPU%.
/// Returns a fully-populated System after both passes.
fn two_pass_refresh(sample_ms: u64) -> System {
    let mut sys = System::new_all();
    sys.refresh_all();
    std::thread::sleep(Duration::from_millis(sample_ms));
    sys.refresh_processes(ProcessesToUpdate::All, true);
    sys.refresh_cpu_all();
    sys
}

/// Display the top-N processes sorted by CPU and Memory.
/// `monitor_secs` — if Some(n), refresh every second for n seconds.
pub fn display_top(monitor_secs: Option<u64>) {
    let duration   = monitor_secs.unwrap_or(0);
    let iterations = if duration == 0 { 1 } else { duration };

    for tick in 0..iterations {
        // ── always sample with two-pass so CPU% is real ───────────────────
        // In monitor mode use 800 ms window; one-shot uses 1 s for accuracy.
        let sample_ms = if duration > 0 { 800 } else { 1000 };
        let sys = two_pass_refresh(sample_ms);

        // ── global CPU (average across all logical cores) ──────────────────
        let cpu_count = sys.cpus().len().max(1);
        let global_cpu: f32 = sys.global_cpu_usage();

        // ── memory ────────────────────────────────────────────────────────
        let total_mem_mb  = sys.total_memory() / 1024 / 1024;
        let used_mem_mb   = sys.used_memory()  / 1024 / 1024;
        let used_swap_mb  = sys.used_swap()    / 1024 / 1024;
        let total_swap_mb = sys.total_swap()   / 1024 / 1024;
        let mem_pct = if total_mem_mb > 0 {
            (used_mem_mb as f32 / total_mem_mb as f32) * 100.0
        } else { 0.0 };

        // ── clear screen only in live-monitor mode ─────────────────────────
        if duration > 0 {
            print!("\x1B[2J\x1B[H");
        }

        // ── header ─────────────────────────────────────────────────────────
        println!("\n{}", "╭──────────────────────────────────────────────────────────────╮".dimmed());
        if duration > 0 {
            println!("{} {} {}",
                "│".dimmed(),
                "📊 PROCESS MONITOR".bold().bright_cyan(),
                format!("  tick {}/{}", tick + 1, iterations).dimmed()
            );
        } else {
            println!("{} {}", "│".dimmed(), "📊 PROCESS MONITOR".bold().bright_cyan());
        }
        println!("{}", "╰──────────────────────────────────────────────────────────────╯".dimmed());

        // ── system summary ─────────────────────────────────────────────────
        println!("\n{}", "  [ System Summary ]".bold().bright_magenta());
        println!("  ├─ 🖥️  {:<10} {} {}",
            "CPU Usage:".bright_green(),
            bar(global_cpu, 20),
            format!("{:.1}%", global_cpu).bright_yellow().bold()
        );
        println!("  ├─ 🧠 {:<10} {} {}",
            "Memory:".bright_green(),
            bar(mem_pct, 20),
            format!("{} MB / {} MB  ({:.1}%)", used_mem_mb, total_mem_mb, mem_pct).bright_cyan()
        );
        println!("  ├─ 🔄 {:<10} {} MB / {} MB",
            "Swap:".bright_green(),
            used_swap_mb.to_string().bright_yellow(),
            total_swap_mb.to_string().bright_cyan()
        );
        println!("  └─ ⚙️  {:<10} {} logical cores",
            "CPUs:".bright_green(),
            cpu_count.to_string().bright_blue().bold()
        );

        // ── collect all processes ──────────────────────────────────────────
        let all_procs: Vec<_> = sys.processes().values().collect();

        // ── top 10 by CPU ─────────────────────────────────────────────────
        println!("\n{}", "  [ Top Processes — CPU ]".bold().bright_magenta());
        println!("  {:<8} {:<28} {:>8}  {:>10}  {}",
            "PID".dimmed(), "NAME".dimmed(), "CPU%".dimmed(),
            "MEM(MB)".dimmed(), "STATUS".dimmed()
        );

        let mut by_cpu = all_procs.clone();
        by_cpu.sort_by(|a, b| b.cpu_usage()
            .partial_cmp(&a.cpu_usage())
            .unwrap_or(std::cmp::Ordering::Equal)
        );

        for proc in by_cpu.iter().take(10) {
            let cpu    = proc.cpu_usage();
            let mem_mb = proc.memory() / 1024 / 1024;
            let cpu_str = if cpu >= 50.0 {
                format!("{:.1}%", cpu).bright_red().bold().to_string()
            } else if cpu >= 10.0 {
                format!("{:.1}%", cpu).bright_yellow().to_string()
            } else {
                format!("{:.1}%", cpu).bright_green().to_string()
            };
            println!("  {:<8} {:<28} {:>8}  {:>10}  {}",
                proc.pid().to_string().bright_blue(),
                proc.name().to_string_lossy().chars().take(27).collect::<String>().bright_yellow(),
                cpu_str,
                format!("{} MB", mem_mb).bright_cyan(),
                format!("{:?}", proc.status()).dimmed(),
            );
        }

        // ── top 10 by Memory — deduplicated by exe path ───────────────────
        // Linux threads all report the same RSS as their parent process.
        // We group by (exe_path, memory) so all threads of zoom / brave / etc.
        // collapse to a single entry showing the lowest PID (the process root).
        println!("\n{}", "  [ Top Processes — Memory (unique processes) ]".bold().bright_magenta());
        println!("  {:<8} {:<28} {:>10}  {:>8}  {}",
            "PID".dimmed(), "NAME".dimmed(), "MEM(MB)".dimmed(),
            "CPU%".dimmed(), "STATUS".dimmed()
        );

        // Build one representative per (exe, memory_bucket) group.
        // Sort ascending by PID first so the root process wins the group.
        let mut by_mem = all_procs.clone();
        by_mem.sort_by(|a, b| a.pid().cmp(&b.pid()));

        // Group key: exe path string + memory value.
        let mut seen_keys: std::collections::HashSet<(String, u64)> = std::collections::HashSet::new();
        let mut unique_by_mem: Vec<_> = Vec::new();
        for proc in &by_mem {
            let exe_key = proc.exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| proc.name().to_string_lossy().to_string());
            let key = (exe_key, proc.memory());
            if seen_keys.insert(key) {
                unique_by_mem.push(*proc);
            }
        }
        // Now sort the deduplicated list by memory descending.
        unique_by_mem.sort_by(|a, b| b.memory().cmp(&a.memory()));

        for proc in unique_by_mem.iter().take(10) {
            let cpu    = proc.cpu_usage();
            let mem    = proc.memory();
            let mem_mb = mem / 1024 / 1024;
            let mem_str = if mem_mb >= 500 {
                format!("{} MB", mem_mb).bright_red().bold().to_string()
            } else if mem_mb >= 200 {
                format!("{} MB", mem_mb).bright_yellow().to_string()
            } else {
                format!("{} MB", mem_mb).bright_cyan().to_string()
            };
            println!("  {:<8} {:<28} {:>10}  {:>8}  {}",
                proc.pid().to_string().bright_blue(),
                proc.name().to_string_lossy().chars().take(27).collect::<String>().bright_yellow(),
                mem_str,
                format!("{:.1}%", cpu).bright_green(),
                format!("{:?}", proc.status()).dimmed(),
            );
        }

        println!();

        // In monitor mode the sample already consumed ~800 ms; no extra sleep.
    }

    if duration > 0 {
        println!("{}", format!("  ✅  Monitoring finished after {} second(s).", duration)
            .bright_green().bold());
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bar_rendering() {
        // Since bar returns colorized strings, we can strip ANSI codes or just check length and characters
        let bar_low = bar(20.0, 10);
        // Should contain 2 filled blocks and 8 empty blocks + owo-colors ANSI sequences
        assert!(bar_low.contains("██░░░░░░░░"));

        let bar_med = bar(60.0, 10);
        assert!(bar_med.contains("██████░░░░"));

        let bar_high = bar(90.0, 10);
        assert!(bar_high.contains("█████████░"));
    }
}

use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use std::thread;

#[derive(Debug, Clone)]
struct IfaceStats {
    rx_bytes: u64,
    tx_bytes: u64,
    rx_packets: u64,
    tx_packets: u64,
    rx_errors: u64,
    tx_errors: u64,
    rx_drop: u64,
    tx_drop: u64,
}

/// Parse /proc/net/dev into a map of interface → stats
fn read_proc_net_dev() -> HashMap<String, IfaceStats> {
    let mut map = HashMap::new();
    let content = match fs::read_to_string("/proc/net/dev") {
        Ok(c) => c,
        Err(_) => return map,
    };
    // Skip 2 header lines
    for line in content.lines().skip(2) {
        let line = line.trim();
        let colon_pos = match line.find(':') {
            Some(p) => p,
            None => continue,
        };
        let iface = line[..colon_pos].trim().to_string();
        let parts: Vec<u64> = line[colon_pos + 1..]
            .split_whitespace()
            .filter_map(|v| v.parse().ok())
            .collect();
        if parts.len() < 16 {
            continue;
        }
        map.insert(iface, IfaceStats {
            rx_bytes:   parts[0],
            rx_packets: parts[1],
            rx_errors:  parts[2],
            rx_drop:    parts[3],
            tx_bytes:   parts[8],
            tx_packets: parts[9],
            tx_errors:  parts[10],
            tx_drop:    parts[11],
        });
    }
    map
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

fn human_rate(bps: f64) -> String {
    if bps >= 1_073_741_824.0 {
        format!("{:.2} GB/s", bps / 1_073_741_824.0)
    } else if bps >= 1_048_576.0 {
        format!("{:.2} MB/s", bps / 1_048_576.0)
    } else if bps >= 1_024.0 {
        format!("{:.2} KB/s", bps / 1_024.0)
    } else {
        format!("{:.0} B/s", bps)
    }
}

/// Print a single snapshot of packet stats for all interfaces.
/// `interval_secs` — optional sample window for computing rates.
pub fn display_packets(interval_secs: Option<u64>) {
    println!("\n{}", "╭──────────────────────────────────────────────────────────────╮".dimmed());
    println!("{} {}", "│".dimmed(), "📦 NETWORK PACKET STATISTICS".bold().bright_cyan());
    println!("{}", "╰──────────────────────────────────────────────────────────────╯".dimmed());

    let sample_secs = interval_secs.unwrap_or(1).max(1);

    println!("\n  {} Sampling over {} second(s)…",
        "⏳".bright_yellow(),
        sample_secs.to_string().bright_blue().bold()
    );

    let before = read_proc_net_dev();
    thread::sleep(Duration::from_secs(sample_secs));
    let after = read_proc_net_dev();

    // Collect interface names and sort
    let mut ifaces: Vec<&String> = after.keys().collect();
    ifaces.sort();

    println!("\n  {:<16} {:>14} {:>14} {:>12} {:>12} {:>10} {:>10}",
        "INTERFACE".dimmed(),
        "↓ RX TOTAL".dimmed(),
        "↑ TX TOTAL".dimmed(),
        "↓ RX RATE".dimmed(),
        "↑ TX RATE".dimmed(),
        "ERR ↓/↑".dimmed(),
        "DROP ↓/↑".dimmed(),
    );
    println!("  {}", "─".repeat(102).dimmed());

    for iface in &ifaces {
        let a = match after.get(*iface) { Some(s) => s, None => continue };
        let b = before.get(*iface);

        let rx_rate = b.map(|prev| {
            (a.rx_bytes.saturating_sub(prev.rx_bytes)) as f64 / sample_secs as f64
        }).unwrap_or(0.0);
        let tx_rate = b.map(|prev| {
            (a.tx_bytes.saturating_sub(prev.tx_bytes)) as f64 / sample_secs as f64
        }).unwrap_or(0.0);

        let iface_colored = if iface.starts_with("lo") {
            iface.dimmed().to_string()
        } else if iface.starts_with("wl") {
            iface.bright_cyan().bold().to_string()
        } else {
            iface.bright_yellow().bold().to_string()
        };

        let err_str  = format!("{}/{}", a.rx_errors, a.tx_errors);
        let drop_str = format!("{}/{}", a.rx_drop, a.tx_drop);

        let rx_rate_str = if rx_rate > 100_000.0 {
            human_rate(rx_rate).bright_green().bold().to_string()
        } else {
            human_rate(rx_rate).dimmed().to_string()
        };
        let tx_rate_str = if tx_rate > 100_000.0 {
            human_rate(tx_rate).bright_magenta().bold().to_string()
        } else {
            human_rate(tx_rate).dimmed().to_string()
        };

        println!("  {:<16} {:>14} {:>14} {:>12} {:>12} {:>10} {:>10}",
            iface_colored,
            human_bytes(a.rx_bytes).bright_cyan(),
            human_bytes(a.tx_bytes).bright_yellow(),
            rx_rate_str,
            tx_rate_str,
            err_str.bright_red(),
            drop_str.bright_red(),
        );
        println!("  {:<16} {}  packets rx: {}  tx: {}",
            "",
            "└─".dimmed(),
            a.rx_packets.to_string().bright_blue(),
            a.tx_packets.to_string().bright_blue(),
        );
    }

    // ─── totals ─────────────────────────────────────────────────────────────
    let total_rx: u64 = after.values().map(|s| s.rx_bytes).sum();
    let total_tx: u64 = after.values().map(|s| s.tx_bytes).sum();
    let total_rx_pkts: u64 = after.values().map(|s| s.rx_packets).sum();
    let total_tx_pkts: u64 = after.values().map(|s| s.tx_packets).sum();

    println!("\n{}", "  ─── Totals ────────────────────────────────────────────────────".dimmed());
    println!("  📥 Total RX : {}  ({} packets)", human_bytes(total_rx).bright_cyan().bold(), total_rx_pkts.to_string().bright_blue());
    println!("  📤 Total TX : {}  ({} packets)", human_bytes(total_tx).bright_yellow().bold(), total_tx_pkts.to_string().bright_blue());
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
        assert_eq!(human_bytes(1500 * 1024), "1.46 MB");
    }

    #[test]
    fn test_human_rate() {
        assert_eq!(human_rate(500.0), "500 B/s");
        assert_eq!(human_rate(1024.0), "1.00 KB/s");
        assert_eq!(human_rate(1024.0 * 1024.0), "1.00 MB/s");
        assert_eq!(human_rate(1024.0 * 1024.0 * 1024.0), "1.00 GB/s");
    }
}

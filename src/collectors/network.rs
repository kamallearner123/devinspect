use netdev;
use owo_colors::OwoColorize;

pub fn display_network() {
    println!("\n{}", "╭──────────────────────────────────────────────────╮".dimmed());
    println!("{} {}", "│".dimmed(), "🌐 NETWORK DIAGNOSTICS".bold().bright_cyan());
    println!("{}", "╰──────────────────────────────────────────────────╯".dimmed());
    
    match netdev::get_default_gateway() {
        Ok(gw) => {
            println!("\n{}", "  [ Default Gateway ]".bold().bright_magenta());
            println!("  ├─ 📡 {:<10} {}", "MAC:".bright_green(), gw.mac_addr.bright_yellow());
            println!("  ├─ 🌍 {:<10} {}", "IPv4:".bright_green(), format!("{:?}", gw.ipv4).bright_cyan().bold());
        }
        Err(e) => {
            println!("\n{}", "  [ Default Gateway ]".bold().bright_magenta());
            println!("  └─ ❌ {:<10} {}", "Gateway:".bright_red(), format!("Could not retrieve ({})", e).dimmed());
        }
    }

    println!("\n{}", "  [ Network Interfaces ]".bold().bright_magenta());
    let interfaces = netdev::get_interfaces();
    for interface in interfaces {
        println!("\n  🔌 {}", interface.name.bright_yellow().bold().underline());
        println!("    ├─ 📡 {:<10} {}", "MAC:", interface.mac_addr.map(|m| m.to_string()).unwrap_or_else(|| "N/A".to_string()).bright_blue());
        
        let ipv4s: Vec<String> = interface.ipv4.iter().map(|ip| ip.addr().to_string()).collect();
        if !ipv4s.is_empty() {
            println!("    ├─ 🌍 {:<10} {}", "IPv4:", ipv4s.join(", ").bright_cyan().bold());
        }
        
        println!("    └─ 📦 {:<10} {}", "MTU:", interface.mtu.map(|m| m.to_string()).unwrap_or_else(|| "N/A".to_string()).bright_blue());
    }
    println!();
}

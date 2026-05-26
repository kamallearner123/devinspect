mod cli;
mod logging;
mod config;
mod collectors;

use clap::{Parser, CommandFactory};
use cli::Cli;
use logging::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    init_logging(cli.trace, cli.debug, cli.verbose);
    
    tracing::debug!("Starting devinspect");
    
    // Load config if exists
    let _app_config = match config::AppConfig::load() {
        Ok(c) => {
            tracing::debug!("Loaded config: {:?}", c);
            c
        },
        Err(_) => {
            tracing::debug!("No valid config found, using defaults");
            config::AppConfig {
                default_theme: Some("dark".to_string()),
                enable_telemetry: Some(false),
            }
        }
    };

    match &cli.command {
        Some(cli::Commands::Summary) => {
            collectors::system::display_summary();
        }
        Some(cli::Commands::Hardware) => {
            collectors::system::display_hardware();
        }
        Some(cli::Commands::Doctor) => {
            println!("Doctor mode (Not yet implemented)");
        }
        Some(cli::Commands::Network) => {
            collectors::network::display_network();
        }
        Some(cli::Commands::All) => {
            collectors::system::display_summary();
            collectors::system::display_hardware();
            collectors::network::display_network();
        }
        Some(cli::Commands::Dashboard) => {
            println!("Launching colorful interactive terminal dashboard... (Ratatui TUI coming in Phase 4!)");
        }
        Some(cli::Commands::Top { monitor_secs }) => {
            collectors::top::display_top(*monitor_secs);
        }
        Some(cli::Commands::Packets { interval_secs }) => {
            collectors::packets::display_packets(*interval_secs);
        }
        Some(cli::Commands::Usb) => {
            collectors::usb::display_usb();
        }
        Some(cli::Commands::Pidstat { pid, duration_secs }) => {
            collectors::pidstat::inspect_pid(*pid, *duration_secs);
        }
        None => {
            let mut cmd = Cli::command();
            let _ = cmd.print_help();
            println!("\n\nExamples:");
            println!("  devinspect summary          # Display system summary");
            println!("  devinspect hardware         # Detailed hardware diagnostics");
            println!("  devinspect --debug doctor   # Run doctor mode with debug logging");
            println!("  devinspect top              # Show top CPU/memory processes");
            println!("  devinspect top -t 5         # Monitor processes for 5 seconds");
            println!("  devinspect packets          # Network packet statistics");
            println!("  devinspect packets -t 3     # Sample packets over 3 seconds");
            println!("  devinspect usb              # List USB devices");
            println!("  devinspect pidstat -p 1200  # Inspect PID 1200");
            println!("  devinspect pidstat -p 1200 -t 5 # Inspect PID 1200 for 5 seconds");
        }
    }

    Ok(())
}

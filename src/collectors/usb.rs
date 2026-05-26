use owo_colors::OwoColorize;
use std::process::Command;

#[derive(Debug)]
struct UsbDevice {
    bus: String,
    device: String,
    id: String,
    vendor_id: String,
    product_id: String,
    description: String,
    speed: Option<String>,
}

/// Try to parse lsusb output line:
///  Bus 001 Device 002: ID 8087:0024 Intel Corp. Integrated Rate Matching Hub
fn parse_lsusb_line(line: &str) -> Option<UsbDevice> {
    // Expected format: "Bus XXX Device YYY: ID VVVV:PPPP Description..."
    let bus_pos   = line.find("Bus ")?;
    let dev_pos   = line.find("Device ")?;
    let id_pos    = line.find(": ID ")?;

    let bus    = line[bus_pos + 4..dev_pos].trim().to_string();
    let device = line[dev_pos + 7..id_pos].trim().split(':').next().unwrap_or("").trim().to_string();

    let rest   = &line[id_pos + 5..]; // after ": ID "
    let mut parts = rest.splitn(2, ' ');
    let id_str    = parts.next().unwrap_or("").trim();
    let description = parts.next().unwrap_or("Unknown").trim().to_string();

    let mut vid_pid = id_str.splitn(2, ':');
    let vendor_id  = vid_pid.next().unwrap_or("????").to_string();
    let product_id = vid_pid.next().unwrap_or("????").to_string();
    let id         = id_str.to_string();

    Some(UsbDevice {
        bus,
        device,
        id,
        vendor_id,
        product_id,
        description,
        speed: None,
    })
}

/// Fetch optional speed from lsusb -v (only for root or permitted users)
fn fetch_speed(bus: &str, device: &str) -> Option<String> {
    let bus_num: u32 = bus.parse().ok()?;
    let dev_num: u32 = device.parse().ok()?;
    let output = Command::new("lsusb")
        .args(["-s", &format!("{}:{}", bus_num, dev_num), "-v"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let l = line.trim();
        if l.starts_with("bcdUSB") {
            let val = l.split_whitespace().nth(1)?.trim().to_string();
            let speed = match val.as_str() {
                v if v.starts_with("3.2") => "SuperSpeed+ (20 Gbps)",
                v if v.starts_with("3.1") => "SuperSpeed+ (10 Gbps)",
                v if v.starts_with("3.0") => "SuperSpeed (5 Gbps)",
                v if v.starts_with("2.0") => "Hi-Speed (480 Mbps)",
                v if v.starts_with("1.1") => "Full-Speed (12 Mbps)",
                v if v.starts_with("1.0") => "Low-Speed (1.5 Mbps)",
                _ => "Unknown",
            };
            return Some(format!("USB {} — {}", val, speed));
        }
    }
    None
}

/// Classify device type from description keywords
fn classify_device(desc: &str) -> &'static str {
    let d = desc.to_lowercase();
    if d.contains("intel") && (d.contains("integrated") || d.contains("rate matching")) { return "🔧 Internal Hub" }
    if d.contains("hub") { return "🔌 Hub" }
    if d.contains("keyboard") { return "⌨️  Keyboard" }
    if d.contains("mouse") || d.contains("touchpad") { return "🖱️  Mouse/Touchpad" }
    if d.contains("camera") || d.contains("webcam") { return "📷 Camera" }
    if d.contains("bluetooth") { return "📶 Bluetooth" }
    if d.contains("audio") || d.contains("headset") || d.contains("sound") { return "🎧 Audio" }
    if d.contains("storage") || d.contains("flash") || d.contains("disk") || d.contains("drive") { return "💾 Storage" }
    if d.contains("network") || d.contains("ethernet") || d.contains("wifi") { return "🌐 Network" }
    if d.contains("printer") { return "🖨️  Printer" }
    if d.contains("serial") || d.contains("uart") || d.contains("ftdi") || d.contains("cp210") { return "🔗 Serial/UART" }
    "📦 Device"
}

pub fn display_usb() {
    println!("\n{}", "╭──────────────────────────────────────────────────────────────╮".dimmed());
    println!("{} {}", "│".dimmed(), "🔌 USB DEVICES".bold().bright_cyan());
    println!("{}", "╰──────────────────────────────────────────────────────────────╯".dimmed());

    // Check if lsusb is available
    let lsusb_check = Command::new("which").arg("lsusb").output();
    if lsusb_check.map(|o| !o.status.success()).unwrap_or(true) {
        println!("\n  {}", "❌  `lsusb` not found. Install usbutils:  sudo apt install usbutils".bright_red());
        println!();
        return;
    }

    let output = match Command::new("lsusb").output() {
        Ok(o) => o,
        Err(e) => {
            println!("\n  {} {}", "❌  Failed to run lsusb:".bright_red(), e.to_string().dimmed());
            println!();
            return;
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices: Vec<UsbDevice> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(parse_lsusb_line)
        .collect();

    if devices.is_empty() {
        println!("\n  {}", "No USB devices detected.".dimmed().italic());
        println!();
        return;
    }

    // Try to enrich with speed info (best-effort, may need sudo)
    for dev in devices.iter_mut() {
        dev.speed = fetch_speed(&dev.bus, &dev.device);
    }

    // Sort by bus then device
    devices.sort_by(|a, b| {
        a.bus.cmp(&b.bus).then(a.device.cmp(&b.device))
    });

    // Count by type
    let mut type_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for dev in &devices {
        let t = classify_device(&dev.description);
        *type_counts.entry(t).or_insert(0) += 1;
    }

    println!("\n{}", "  [ Summary ]".bold().bright_magenta());
    println!("  Total USB devices detected: {}", devices.len().to_string().bright_yellow().bold());
    let mut type_list: Vec<(&str, usize)> = type_counts.into_iter().collect();
    type_list.sort_by(|a, b| b.1.cmp(&a.1));
    for (t, cnt) in &type_list {
        println!("  ├─ {:<22} {}", t.bright_blue(), cnt.to_string().bright_cyan().bold());
    }

    // ─── per-bus tree ─────────────────────────────────────────────────────
    println!("\n{}", "  [ Device List ]".bold().bright_magenta());

    let mut current_bus = String::new();
    let last_idx = devices.len().saturating_sub(1);
    for (i, dev) in devices.iter().enumerate() {
        if dev.bus != current_bus {
            current_bus = dev.bus.clone();
            println!("\n  🚌 {}", format!("Bus {:>3}", dev.bus).bright_yellow().bold().underline());
        }
        let is_last = i == last_idx || devices.get(i + 1).map(|d| d.bus != dev.bus).unwrap_or(true);
        let connector = if is_last { "└─" } else { "├─" };

        let device_type = classify_device(&dev.description);

        let desc_colored = if dev.description.to_lowercase().contains("hub") {
            dev.description.dimmed().italic().to_string()
        } else {
            dev.description.bright_blue().bold().to_string()
        };

        println!("  {} {} {} [{}]",
            connector.dimmed(),
            device_type,
            desc_colored,
            dev.id.bright_cyan()
        );
        println!("  {}   Device {:>3}  VID:{} PID:{}{}",
            if is_last { " " } else { "│" }.dimmed(),
            dev.device.bright_blue(),
            dev.vendor_id.bright_yellow(),
            dev.product_id.bright_yellow(),
            dev.speed.as_deref().map(|s| format!("  ⚡ {}", s.bright_green())).unwrap_or_default(),
        );
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_device() {
        assert_eq!(classify_device("Standard USB Hub"), "🔌 Hub");
        assert_eq!(classify_device("Apple Keyboard"), "⌨️  Keyboard");
        assert_eq!(classify_device("Logitech USB Optical Mouse"), "🖱️  Mouse/Touchpad");
        assert_eq!(classify_device("HD Web Camera"), "📷 Camera");
        assert_eq!(classify_device("Intel Wireless Bluetooth Adapter"), "📶 Bluetooth");
        assert_eq!(classify_device("USB Audio Device"), "🎧 Audio");
        assert_eq!(classify_device("Sandisk Flash Drive"), "💾 Storage");
        assert_eq!(classify_device("Realtek USB Ethernet"), "🌐 Network");
        assert_eq!(classify_device("HP LaserJet Printer"), "🖨️  Printer");
        assert_eq!(classify_device("FTDI USB Serial Port"), "🔗 Serial/UART");
        assert_eq!(classify_device("Intel Corporation Integrated Rate Matching Hub"), "🔧 Internal Hub");
        assert_eq!(classify_device("Random gadget"), "📦 Device");
    }

    #[test]
    fn test_parse_lsusb_line() {
        let line = "Bus 001 Device 002: ID 8087:0024 Intel Corp. Integrated Rate Matching Hub";
        let dev = parse_lsusb_line(line).unwrap();
        assert_eq!(dev.bus, "001");
        assert_eq!(dev.device, "002");
        assert_eq!(dev.vendor_id, "8087");
        assert_eq!(dev.product_id, "0024");
        assert_eq!(dev.description, "Intel Corp. Integrated Rate Matching Hub");
        assert_eq!(dev.id, "8087:0024");

        let invalid_line = "Random line with no matching pattern";
        assert!(parse_lsusb_line(invalid_line).is_none());
    }
}

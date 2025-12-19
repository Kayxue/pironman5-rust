use crate::pironman5::Pironman5;
use crate::version::VERSION;
use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Parser)]
#[command(name = "pironman5")]
#[command(about = "Pironman 5 command line interface", long_about = None)]
#[command(version = VERSION)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Show version
    #[arg(short, long)]
    pub version: bool,

    /// Show config
    #[arg(short, long)]
    pub config: bool,

    /// Debug level
    #[arg(short = 'd', long = "debug-level", value_name = "LEVEL")]
    pub debug_level: Option<String>,

    /// Config path
    #[arg(long = "config-path", value_name = "PATH")]
    pub config_path: Option<String>,

    /// RGB color in hex format without # (e.g. 00aabb)
    #[arg(long = "rgb-color", value_name = "COLOR")]
    pub rgb_color: Option<String>,

    /// RGB brightness 0-100
    #[arg(long = "rgb-brightness", value_name = "BRIGHTNESS")]
    pub rgb_brightness: Option<u8>,

    /// RGB style
    #[arg(long = "rgb-style", value_name = "STYLE")]
    pub rgb_style: Option<String>,

    /// RGB speed 0-100
    #[arg(long = "rgb-speed", value_name = "SPEED")]
    pub rgb_speed: Option<u8>,

    /// RGB enable True/False
    #[arg(long = "rgb-enable", value_name = "ENABLE")]
    pub rgb_enable: Option<String>,

    /// RGB LED count
    #[arg(long = "rgb-led-count", value_name = "COUNT")]
    pub rgb_led_count: Option<u32>,

    /// Temperature unit (C or F)
    #[arg(short = 'u', long = "temperature-unit", value_name = "UNIT")]
    pub temperature_unit: Option<String>,

    /// GPIO fan mode
    #[arg(long = "gpio-fan-mode", value_name = "MODE")]
    pub gpio_fan_mode: Option<u8>,

    /// GPIO fan pin
    #[arg(long = "gpio-fan-pin", value_name = "PIN")]
    pub gpio_fan_pin: Option<u8>,

    /// GPIO fan LED state (on/off/follow)
    #[arg(long = "gpio-fan-led", value_name = "STATE")]
    pub gpio_fan_led: Option<String>,

    /// GPIO fan LED pin
    #[arg(long = "gpio-fan-led-pin", value_name = "PIN")]
    pub gpio_fan_led_pin: Option<u8>,

    /// OLED enable
    #[arg(long = "oled-enable", value_name = "ENABLE")]
    pub oled_enable: Option<String>,

    /// OLED disk to display
    #[arg(long = "oled-disk", value_name = "DISK")]
    pub oled_disk: Option<String>,

    /// OLED network interface
    #[arg(long = "oled-network-interface", value_name = "INTERFACE")]
    pub oled_network_interface: Option<String>,

    /// OLED rotation (0 or 180)
    #[arg(long = "oled-rotation", value_name = "ROTATION")]
    pub oled_rotation: Option<u16>,

    /// Vibration switch pin
    #[arg(long = "vibration-switch-pin", value_name = "PIN")]
    pub vibration_switch_pin: Option<u8>,

    /// Vibration switch pull up
    #[arg(long = "vibration-switch-pull-up", value_name = "ENABLE")]
    pub vibration_switch_pull_up: Option<String>,

    /// OLED sleep timeout in seconds
    #[arg(long = "oled-sleep-timeout", value_name = "TIMEOUT")]
    pub oled_sleep_timeout: Option<u32>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the Pironman5 service
    Start,
    /// Restart the Pironman5 service
    Restart,
    /// Stop the Pironman5 service
    Stop,
}

/// Parse boolean string
fn parse_bool(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "on" | "yes" => Some(true),
        "false" | "0" | "off" | "no" => Some(false),
        _ => None,
    }
}

/// Load config from file
fn load_config(config_path: &str) -> Result<Value> {
    if !Path::new(config_path).exists() {
        let config = json!({"system": {}});
        let mut file = File::create(config_path)?;
        file.write_all(serde_json::to_string_pretty(&config)?.as_bytes())?;
        return Ok(config);
    }

    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    if contents.is_empty() {
        return Ok(json!({"system": {}}));
    }

    Ok(serde_json::from_str(&contents)?)
}

/// Update config file
fn update_config_file(config_path: &str, new_config: Value) -> Result<()> {
    let mut current = load_config(config_path)?;
    
    if let (Some(current_sys), Some(new_sys)) = (
        current.get_mut("system").and_then(|v| v.as_object_mut()),
        new_config.get("system").and_then(|v| v.as_object())
    ) {
        for (key, value) in new_sys {
            current_sys.insert(key.clone(), value.clone());
        }
    }

    let mut file = File::create(config_path)?;
    file.write_all(serde_json::to_string_pretty(&current)?.as_bytes())?;
    Ok(())
}

/// Run the CLI
pub fn run(cli: Cli) -> Result<()> {
    // Show version
    if cli.version {
        println!("{}", VERSION);
        return Ok(());
    }

    let config_path = cli.config_path
        .clone()
        .unwrap_or_else(|| "/opt/pironman5/config.json".to_string());

    // Show config
    if cli.config {
        let config = load_config(&config_path)?;
        println!("{}", serde_json::to_string_pretty(&config)?);
        return Ok(());
    }

    // Build new config from CLI arguments
    let mut new_sys_config = json!({});

    // Debug level
    if let Some(debug_level) = &cli.debug_level {
        new_sys_config["debug_level"] = json!(debug_level.to_uppercase());
        println!("Set debug level: {}", debug_level.to_uppercase());
    }

    // RGB settings
    if let Some(color) = &cli.rgb_color {
        if color.len() != 6 {
            anyhow::bail!("Invalid RGB color format. Use hex format without # (e.g. 00aabb)");
        }
        new_sys_config["rgb_color"] = json!(format!("#{}", color));
        println!("Set RGB color: #{}", color);
    }

    if let Some(brightness) = cli.rgb_brightness {
        if brightness > 100 {
            anyhow::bail!("RGB brightness must be between 0 and 100");
        }
        new_sys_config["rgb_brightness"] = json!(brightness);
        println!("Set RGB brightness: {}", brightness);
    }

    if let Some(style) = &cli.rgb_style {
        new_sys_config["rgb_style"] = json!(style);
        println!("Set RGB style: {}", style);
    }

    if let Some(speed) = cli.rgb_speed {
        if speed > 100 {
            anyhow::bail!("RGB speed must be between 0 and 100");
        }
        new_sys_config["rgb_speed"] = json!(speed);
        println!("Set RGB speed: {}", speed);
    }

    if let Some(enable) = &cli.rgb_enable {
        if let Some(enable_bool) = parse_bool(enable) {
            new_sys_config["rgb_enable"] = json!(enable_bool);
            println!("Set RGB enable: {}", enable_bool);
        } else {
            anyhow::bail!("Invalid value for RGB enable. Use true/false");
        }
    }

    if let Some(count) = cli.rgb_led_count {
        if count < 1 {
            anyhow::bail!("RGB LED count must be greater than 0");
        }
        new_sys_config["rgb_led_count"] = json!(count);
        println!("Set RGB LED count: {}", count);
    }

    // Temperature unit
    if let Some(unit) = &cli.temperature_unit {
        if unit != "C" && unit != "F" {
            anyhow::bail!("Temperature unit must be C or F");
        }
        new_sys_config["temperature_unit"] = json!(unit);
        println!("Set temperature unit: {}", unit);
    }

    // GPIO fan settings
    if let Some(mode) = cli.gpio_fan_mode {
        new_sys_config["gpio_fan_mode"] = json!(mode);
        println!("Set GPIO fan mode: {}", mode);
    }

    if let Some(pin) = cli.gpio_fan_pin {
        new_sys_config["gpio_fan_pin"] = json!(pin);
        println!("Set GPIO fan pin: {}", pin);
    }

    if let Some(state) = &cli.gpio_fan_led {
        let state_lower = state.to_lowercase();
        if state_lower != "on" && state_lower != "off" && state_lower != "follow" {
            anyhow::bail!("GPIO fan LED state must be on, off, or follow");
        }
        new_sys_config["gpio_fan_led"] = json!(state_lower);
        println!("Set GPIO fan LED state: {}", state_lower);
    }

    if let Some(pin) = cli.gpio_fan_led_pin {
        new_sys_config["gpio_fan_led_pin"] = json!(pin);
        println!("Set GPIO fan LED pin: {}", pin);
    }

    // OLED settings
    if let Some(enable) = &cli.oled_enable {
        if let Some(enable_bool) = parse_bool(enable) {
            new_sys_config["oled_enable"] = json!(enable_bool);
            println!("Set OLED enable: {}", enable_bool);
        } else {
            anyhow::bail!("Invalid value for OLED enable. Use true/false");
        }
    }

    if let Some(disk) = &cli.oled_disk {
        new_sys_config["oled_disk"] = json!(disk);
        println!("Set OLED disk: {}", disk);
    }

    if let Some(interface) = &cli.oled_network_interface {
        new_sys_config["oled_network_interface"] = json!(interface);
        println!("Set OLED network interface: {}", interface);
    }

    if let Some(rotation) = cli.oled_rotation {
        if rotation != 0 && rotation != 180 {
            anyhow::bail!("OLED rotation must be 0 or 180");
        }
        new_sys_config["oled_rotation"] = json!(rotation);
        println!("Set OLED rotation: {}", rotation);
    }

    // Vibration switch settings
    if let Some(pin) = cli.vibration_switch_pin {
        if pin > 40 {
            anyhow::bail!("Vibration switch pin must be between 0 and 40");
        }
        new_sys_config["vibration_switch_pin"] = json!(pin);
        println!("Set vibration switch pin: {}", pin);
    }

    if let Some(pull_up) = &cli.vibration_switch_pull_up {
        if let Some(pull_up_bool) = parse_bool(pull_up) {
            new_sys_config["vibration_switch_pull_up"] = json!(pull_up_bool);
            println!("Set vibration switch pull up: {}", pull_up_bool);
        } else {
            anyhow::bail!("Invalid value for vibration switch pull up. Use true/false");
        }
    }

    if let Some(timeout) = cli.oled_sleep_timeout {
        new_sys_config["oled_sleep_timeout"] = json!(timeout);
        println!("Set OLED sleep timeout: {}", timeout);
    }

    // Update config file if any settings were changed
    if !new_sys_config.as_object().unwrap().is_empty() {
        let new_config = json!({"system": new_sys_config});
        update_config_file(&config_path, new_config)?;
    }

    // Handle commands
    match &cli.command {
        Some(Commands::Start) => {
            let mut pironman5 = Pironman5::new(Some(config_path))?;
            pironman5.start()?;
        }
        Some(Commands::Restart) => {
            println!("Stopping Pironman5...");
            // In a real implementation, this would send a signal to the running process
            std::process::Command::new("pkill")
                .args(&["-15", "-f", "pironman5 start"])
                .status()
                .ok();
            std::thread::sleep(std::time::Duration::from_secs(5));
            
            println!("Starting Pironman5...");
            let mut pironman5 = Pironman5::new(Some(config_path))?;
            pironman5.start()?;
        }
        Some(Commands::Stop) => {
            println!("Stopping Pironman5...");
            // Send termination signal to running process
            std::process::Command::new("pkill")
                .args(&["-15", "-f", "pironman5 start"])
                .status()
                .ok();
            std::process::Command::new("pkill")
                .args(&["-15", "-f", "pironman5-service start"])
                .status()
                .ok();
            
            let mut pironman5 = Pironman5::new(Some(config_path))?;
            pironman5.stop()?;
        }
        None => {
            // No command specified and no flags - show help
            if !cli.version && !cli.config && new_sys_config.as_object().unwrap().is_empty() {
                println!("pironman5 {}", VERSION);
                println!("\nUSAGE:");
                println!("    pironman5 [OPTIONS] [COMMAND]");
                println!("\nCOMMANDS:");
                println!("    start     Start the Pironman5 service");
                println!("    restart   Restart the Pironman5 service");
                println!("    stop      Stop the Pironman5 service");
                println!("\nOPTIONS:");
                println!("    -h, --help              Print help information");
                println!("    -v, --version           Show version");
                println!("    -c, --config            Show config");
                println!("    -d, --debug-level <LEVEL>    Set debug level");
                println!("\nFor more options, run: pironman5 --help");
            }
        }
    }

    Ok(())
}


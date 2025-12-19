use crate::hardware::{HardwareManager, system_monitor::SystemMonitor};
use crate::logger::Logger;
use crate::utils::merge_dict;
use crate::variants::{get_active_variant, Variant};
use crate::version::VERSION;
use anyhow::{Context, Result};
use log::LevelFilter;
use serde_json::{json, Value};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const APP_NAME: &str = "pironman5";
const DEFAULT_DEBUG_LEVEL: &str = "INFO";

/// Main Pironman5 application struct
pub struct Pironman5 {
    config: Value,
    #[allow(dead_code)]
    config_path: String,
    logger: Logger,
    variant: Box<dyn Variant>,
    running: Arc<AtomicBool>,
    hardware: Option<HardwareManager>,
    system_monitor: Option<SystemMonitor>,
}

impl Pironman5 {
    /// Create a new Pironman5 instance
    pub fn new(config_path: Option<String>) -> Result<Self> {
        let variant = get_active_variant();
        
        // Determine config path
        let config_path = config_path.unwrap_or_else(|| {
            // Default config path - in a real implementation, this would use resource files
            format!("/opt/pironman5/config.json")
        });

        // Initialize logger
        let logger = Logger::new(APP_NAME, "main", LevelFilter::Info)
            .context("Failed to create logger")?;

        // Load config
        let mut config = json!({
            "system": variant.system_default_config(),
        });
        config["system"]["debug_level"] = json!(DEFAULT_DEBUG_LEVEL);

        // Load existing config if present
        if Path::new(&config_path).exists() {
            let mut file = File::open(&config_path)
                .context("Failed to open config file")?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .context("Failed to read config file")?;
            
            if !contents.is_empty() {
                let loaded_config: Value = serde_json::from_str(&contents)
                    .context("Failed to parse config file")?;
                let loaded_config = Self::upgrade_config(loaded_config);
                config = merge_dict(&config, &loaded_config);
            }
        }

        // Save config
        let mut file = File::create(&config_path)
            .context("Failed to create config file")?;
        file.write_all(serde_json::to_string_pretty(&config)?.as_bytes())
            .context("Failed to write config file")?;

        // Log header
        println!("");
        println!("{}", "#".repeat(60));
        println!("Config path: {}", config_path);
        println!("Pironman5 version: {}", VERSION);
        println!("Variant: {} {}", variant.name(), variant.product_version());
        println!("Config: {}", serde_json::to_string_pretty(&config)?);

        let running = Arc::new(AtomicBool::new(false));

        // Initialize hardware (this will fail gracefully if not on Pi hardware)
        let peripherals = variant.peripherals();
        let hardware = match HardwareManager::new(&config, &peripherals) {
            Ok(hw) => {
                println!("Hardware initialization successful");
                Some(hw)
            }
            Err(e) => {
                println!("Warning: Hardware initialization failed ({})", e);
                println!("Running in simulation mode without hardware control");
                None
            }
        };

        // Initialize system monitor
        let system_monitor = Some(SystemMonitor::new());

        let mut pironman5 = Pironman5 {
            config,
            config_path,
            logger,
            variant,
            running,
            hardware,
            system_monitor,
        };

        // Set debug level
        let debug_level = pironman5.config["system"]["debug_level"]
            .as_str()
            .unwrap_or(DEFAULT_DEBUG_LEVEL)
            .to_string();
        pironman5.set_debug_level(&debug_level)?;

        Ok(pironman5)
    }

    /// Set debug level
    pub fn set_debug_level(&mut self, level: &str) -> Result<()> {
        let level_filter = match level.to_uppercase().as_str() {
            "DEBUG" => LevelFilter::Debug,
            "INFO" => LevelFilter::Info,
            "WARNING" | "WARN" => LevelFilter::Warn,
            "ERROR" => LevelFilter::Error,
            "CRITICAL" => LevelFilter::Error,
            _ => {
                println!("Invalid debug level '{}', using default '{}'", level, DEFAULT_DEBUG_LEVEL);
                LevelFilter::Info
            }
        };
        
        self.logger.set_level(level_filter);
        Ok(())
    }

    /// Upgrade old config format to new format
    fn upgrade_config(config: Value) -> Value {
        if let Some(auto_config) = config.get("auto") {
            json!({ "system": auto_config })
        } else {
            config
        }
    }

    /// Update configuration
    #[allow(dead_code)]
    pub fn update_config(&mut self, new_config: Value) -> Result<()> {
        self.config = merge_dict(&self.config, &new_config);
        
        let mut file = File::create(&self.config_path)
            .context("Failed to create config file")?;
        file.write_all(serde_json::to_string_pretty(&self.config)?.as_bytes())
            .context("Failed to write config file")?;
        
        Ok(())
    }

    /// Start the Pironman5 service
    pub fn start(&mut self) -> Result<()> {
        println!("Starting Pironman5...");
        
        self.running.store(true, Ordering::SeqCst);
        
        // Setup signal handlers
        let running = Arc::clone(&self.running);
        ctrlc::set_handler(move || {
            println!("Received interrupt signal, shutting down...");
            running.store(false, Ordering::SeqCst);
        })
        .context("Error setting Ctrl-C handler")?;

        if self.hardware.is_some() {
            println!("Hardware control active");
        } else {
            println!("Running in simulation mode (no hardware)");
        }
        
        // Main loop - monitor system and update hardware
        let mut loop_counter = 0;
        while self.running.load(Ordering::SeqCst) {
            // Update system status every second
            if let Some(monitor) = &mut self.system_monitor {
                if let Ok(status) = monitor.get_status() {
                    // Update hardware with current status
                    if let Some(hardware) = &mut self.hardware {
                        if let Err(e) = hardware.update(&status) {
                            eprintln!("Hardware update error: {}", e);
                        }
                    }

                    // Log status every 10 seconds
                    if loop_counter % 10 == 0 {
                        println!("CPU: {:.1}% | MEM: {:.1}% | TEMP: {:.1}°C",
                            status.cpu_usage, status.memory_usage, status.cpu_temperature);
                    }
                }
            }

            loop_counter += 1;
            thread::sleep(Duration::from_secs(1));
        }

        self.stop()?;
        Ok(())
    }

    /// Stop the Pironman5 service
    pub fn stop(&mut self) -> Result<()> {
        println!("Stopping Pironman5");
        
        // Shutdown hardware gracefully
        if let Some(hardware) = &mut self.hardware {
            if let Err(e) = hardware.shutdown() {
                eprintln!("Error during hardware shutdown: {}", e);
            }
        }
        
        println!("Pironman5 stopped");
        Ok(())
    }

    /// Get current config
    #[allow(dead_code)]
    pub fn get_config(&self) -> &Value {
        &self.config
    }

    /// Get variant name
    #[allow(dead_code)]
    pub fn variant_name(&self) -> &str {
        self.variant.name()
    }

    /// Get peripherals
    #[allow(dead_code)]
    pub fn peripherals(&self) -> Vec<String> {
        self.variant.peripherals()
    }
}


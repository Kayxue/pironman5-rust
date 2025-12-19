/// Hardware control module for Pironman5
/// 
/// This module provides hardware abstraction for:
/// - RGB LEDs (WS2812 via SPI)
/// - OLED display (SSD1306 via I2C)
/// - PWM fan control
/// - Temperature sensors
/// - GPIO operations

// Hardware control modules - only compile on Linux
#[cfg(target_os = "linux")]
pub mod rgb;
#[cfg(target_os = "linux")]
pub mod oled;
#[cfg(target_os = "linux")]
pub mod fan;
#[cfg(target_os = "linux")]
pub mod temperature;

// System monitoring works on all platforms
pub mod system_monitor;

use anyhow::Result;
use serde_json::Value;

/// Hardware manager that coordinates all hardware components
#[cfg(target_os = "linux")]
pub struct HardwareManager {
    pub rgb: Option<rgb::RgbController>,
    pub oled: Option<oled::OledController>,
    pub fan: Option<fan::FanController>,
    pub temperature: Option<temperature::TemperatureMonitor>,
}

/// Stub hardware manager for non-Linux platforms
#[cfg(not(target_os = "linux"))]
pub struct HardwareManager;

#[cfg(target_os = "linux")]
impl HardwareManager {

    /// Initialize hardware components based on configuration
    pub fn new(config: &Value, peripherals: &[String]) -> Result<Self> {
        println!("Initializing hardware components...");
        
        // Initialize RGB controller if ws2812 peripheral is present
        let rgb = if peripherals.contains(&"ws2812".to_string()) {
            match rgb::RgbController::new(config) {
                Ok(controller) => {
                    println!("✓ RGB controller initialized");
                    Some(controller)
                }
                Err(e) => {
                    eprintln!("✗ RGB controller failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Initialize OLED controller if oled peripheral is present
        let oled = if peripherals.contains(&"oled".to_string()) {
            match oled::OledController::new(config) {
                Ok(controller) => {
                    println!("✓ OLED display initialized");
                    Some(controller)
                }
                Err(e) => {
                    eprintln!("✗ OLED display failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Initialize fan controller
        let fan = match fan::FanController::new(config) {
            Ok(controller) => {
                println!("✓ Fan controller initialized");
                Some(controller)
            }
            Err(e) => {
                eprintln!("✗ Fan controller failed: {}", e);
                None
            }
        };

        // Initialize temperature monitor
        let temperature = match temperature::TemperatureMonitor::new(config) {
            Ok(monitor) => {
                println!("✓ Temperature monitor initialized");
                Some(monitor)
            }
            Err(e) => {
                eprintln!("✗ Temperature monitor failed: {}", e);
                None
            }
        };

        Ok(HardwareManager {
            rgb,
            oled,
            fan,
            temperature,
        })
    }

    /// Update hardware state based on system status
    pub fn update(&mut self, status: &SystemStatus) -> Result<()> {
        // Update RGB LEDs
        if let Some(rgb) = &mut self.rgb {
            rgb.update(status)?;
        }

        // Update OLED display
        if let Some(oled) = &mut self.oled {
            oled.update(status)?;
        }

        // Update fan speed based on temperature
        if let Some(fan) = &mut self.fan {
            if let Some(temp_monitor) = &self.temperature {
                let cpu_temp = temp_monitor.read_cpu_temperature()?;
                fan.set_speed_for_temperature(cpu_temp)?;
            }
        }

        Ok(())
    }

    /// Clean shutdown of hardware
    pub fn shutdown(&mut self) -> Result<()> {
        println!("Shutting down hardware...");
        
        if let Some(rgb) = &mut self.rgb {
            rgb.shutdown()?;
        }
        
        if let Some(oled) = &mut self.oled {
            oled.shutdown()?;
        }
        
        if let Some(fan) = &mut self.fan {
            fan.shutdown()?;
        }
        
        println!("Hardware shutdown complete");
        Ok(())
    }
}

// Stub implementation for non-Linux platforms
#[cfg(not(target_os = "linux"))]
impl HardwareManager {
    pub fn new(_config: &Value, _peripherals: &[String]) -> Result<Self> {
        anyhow::bail!("Hardware control only available on Linux/Raspberry Pi")
    }

    pub fn update(&mut self, _status: &SystemStatus) -> Result<()> {
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// System status information passed to hardware components
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub cpu_temperature: f32,
    pub gpu_temperature: Option<f32>,
    pub disk_usage: f32,
    pub network_active: bool,
    pub network_ip: Option<String>,
    pub network_interface: String,
}

impl Default for SystemStatus {
    fn default() -> Self {
        SystemStatus {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            cpu_temperature: 0.0,
            gpu_temperature: None,
            disk_usage: 0.0,
            network_active: false,
            network_ip: None,
            network_interface: String::from("unknown"),
        }
    }
}


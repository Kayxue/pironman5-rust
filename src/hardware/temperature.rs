/// Temperature monitoring for Raspberry Pi
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;

const THERMAL_ZONE_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";
const VCGENCMD_PATH: &str = "/usr/bin/vcgencmd";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

impl TemperatureUnit {
    fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "F" => TemperatureUnit::Fahrenheit,
            _ => TemperatureUnit::Celsius,
        }
    }

    fn convert(&self, celsius: f32) -> f32 {
        match self {
            TemperatureUnit::Celsius => celsius,
            TemperatureUnit::Fahrenheit => celsius * 9.0 / 5.0 + 32.0,
        }
    }
}

pub struct TemperatureMonitor {
    unit: TemperatureUnit,
}

impl TemperatureMonitor {
    pub fn new(config: &Value) -> Result<Self> {
        let system = &config["system"];
        let unit_str = system["temperature_unit"].as_str().unwrap_or("C");
        let unit = TemperatureUnit::from_str(unit_str);

        Ok(TemperatureMonitor { unit })
    }

    /// Read CPU temperature from thermal zone
    pub fn read_cpu_temperature(&self) -> Result<f32> {
        let temp_str = fs::read_to_string(THERMAL_ZONE_PATH)
            .context("Failed to read CPU temperature")?;
        
        let temp_millidegrees: i32 = temp_str
            .trim()
            .parse()
            .context("Failed to parse temperature")?;
        
        let temp_celsius = temp_millidegrees as f32 / 1000.0;
        Ok(self.unit.convert(temp_celsius))
    }

    /// Read GPU temperature using vcgencmd
    pub fn read_gpu_temperature(&self) -> Result<f32> {
        let output = std::process::Command::new(VCGENCMD_PATH)
            .arg("measure_temp")
            .output()
            .context("Failed to execute vcgencmd")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse output like "temp=45.0'C"
        let temp_str = output_str
            .trim()
            .strip_prefix("temp=")
            .and_then(|s| s.strip_suffix("'C"))
            .context("Failed to parse vcgencmd output")?;
        
        let temp_celsius: f32 = temp_str
            .parse()
            .context("Failed to parse GPU temperature")?;
        
        Ok(self.unit.convert(temp_celsius))
    }

    /// Read both CPU and GPU temperatures
    pub fn read_all_temperatures(&self) -> Result<(f32, Option<f32>)> {
        let cpu_temp = self.read_cpu_temperature()?;
        let gpu_temp = self.read_gpu_temperature().ok();
        Ok((cpu_temp, gpu_temp))
    }

    pub fn get_unit(&self) -> TemperatureUnit {
        self.unit
    }

    pub fn set_unit(&mut self, unit: TemperatureUnit) {
        self.unit = unit;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_conversion() {
        let unit_c = TemperatureUnit::Celsius;
        let unit_f = TemperatureUnit::Fahrenheit;

        assert_eq!(unit_c.convert(0.0), 0.0);
        assert_eq!(unit_f.convert(0.0), 32.0);
        assert_eq!(unit_c.convert(100.0), 100.0);
        assert!((unit_f.convert(100.0) - 212.0).abs() < 0.01);
    }
}


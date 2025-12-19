/// Fan controller for PWM and GPIO fan control
use anyhow::{Context, Result};
use rppal::gpio::{Gpio, OutputPin};
use rppal::pwm::{Channel, Polarity, Pwm};
use serde_json::Value;

const PWM_FREQUENCY: f64 = 25000.0; // 25 kHz for fan PWM
const MIN_DUTY_CYCLE: f64 = 0.0;
const MAX_DUTY_CYCLE: f64 = 1.0;

pub struct FanController {
    pwm: Option<Pwm>,
    gpio_pin: Option<OutputPin>,
    gpio_led_pin: Option<OutputPin>,
    mode: FanMode,
    current_speed: f64,
    temperature_thresholds: Vec<(f32, f64)>, // (temp, speed) pairs
}

#[derive(Debug, Clone, Copy)]
pub enum FanMode {
    AlwaysOn,
    Temperature,
    Quiet,
    Performance,
}

impl FanMode {
    fn from_value(mode: u8) -> Self {
        match mode {
            0 => FanMode::AlwaysOn,
            1 => FanMode::Temperature,
            2 => FanMode::Quiet,
            3 => FanMode::Performance,
            _ => FanMode::Temperature,
        }
    }

    fn get_thresholds(&self) -> Vec<(f32, f64)> {
        match self {
            FanMode::AlwaysOn => vec![(0.0, 1.0)],
            FanMode::Temperature => vec![
                (40.0, 0.3),
                (50.0, 0.5),
                (60.0, 0.7),
                (70.0, 1.0),
            ],
            FanMode::Quiet => vec![
                (50.0, 0.3),
                (60.0, 0.5),
                (70.0, 0.7),
                (80.0, 1.0),
            ],
            FanMode::Performance => vec![
                (35.0, 0.5),
                (45.0, 0.7),
                (55.0, 0.85),
                (65.0, 1.0),
            ],
        }
    }
}

impl FanController {
    pub fn new(config: &Value) -> Result<Self> {
        let system = &config["system"];
        
        // Parse fan mode
        let mode_value = system["gpio_fan_mode"].as_u64().unwrap_or(1) as u8;
        let mode = FanMode::from_value(mode_value);
        
        // Try to initialize PWM (hardware PWM on Pi 5)
        let pwm = match Pwm::with_frequency(
            Channel::Pwm0,
            PWM_FREQUENCY,
            0.0,
            Polarity::Normal,
            true,
        ) {
            Ok(pwm) => {
                println!("Using hardware PWM for fan control");
                Some(pwm)
            }
            Err(e) => {
                println!("Hardware PWM not available ({}), using GPIO fallback", e);
                None
            }
        };

        // Initialize GPIO pin for on/off control (fallback or secondary control)
        let gpio_pin = if let Some(pin_num) = system["gpio_fan_pin"].as_u64() {
            let gpio = Gpio::new().context("Failed to initialize GPIO")?;
            let pin = gpio.get(pin_num as u8)
                .context("Failed to get GPIO pin")?
                .into_output();
            Some(pin)
        } else {
            None
        };

        // Initialize GPIO LED pin if present (for Mini/Max variants)
        let gpio_led_pin = if let Some(led_pin_num) = system["gpio_fan_led_pin"].as_u64() {
            let gpio = Gpio::new().context("Failed to initialize GPIO for LED")?;
            let pin = gpio.get(led_pin_num as u8)
                .context("Failed to get GPIO LED pin")?
                .into_output();
            Some(pin)
        } else {
            None
        };

        let temperature_thresholds = mode.get_thresholds();

        Ok(FanController {
            pwm,
            gpio_pin,
            gpio_led_pin,
            mode,
            current_speed: 0.0,
            temperature_thresholds,
        })
    }

    pub fn set_speed_for_temperature(&mut self, temperature: f32) -> Result<()> {
        // Determine target speed based on temperature and thresholds
        let target_speed = self.calculate_target_speed(temperature);
        
        // Apply hysteresis to prevent rapid switching
        let speed_change = (target_speed - self.current_speed).abs();
        if speed_change < 0.05 && self.current_speed > 0.0 {
            return Ok(()); // Don't change if difference is small
        }

        self.set_speed(target_speed)?;
        Ok(())
    }

    fn calculate_target_speed(&self, temperature: f32) -> f64 {
        for (i, &(temp_threshold, speed)) in self.temperature_thresholds.iter().enumerate() {
            if temperature < temp_threshold {
                if i == 0 {
                    return 0.0; // Below first threshold
                }
                // Interpolate between previous and current threshold
                let (prev_temp, prev_speed) = self.temperature_thresholds[i - 1];
                let temp_range = temp_threshold - prev_temp;
                let temp_progress = (temperature - prev_temp) / temp_range;
                return prev_speed + (speed - prev_speed) * temp_progress as f64;
            }
        }
        
        // Above all thresholds, use maximum speed
        self.temperature_thresholds.last().unwrap().1
    }

    pub fn set_speed(&mut self, speed: f64) -> Result<()> {
        let speed = speed.clamp(MIN_DUTY_CYCLE, MAX_DUTY_CYCLE);
        
        // Set PWM duty cycle if available
        if let Some(pwm) = &self.pwm {
            pwm.set_duty_cycle(speed)
                .context("Failed to set PWM duty cycle")?;
        } else if let Some(pin) = &mut self.gpio_pin {
            // Fallback: simple on/off control
            if speed > 0.5 {
                pin.set_high();
            } else {
                pin.set_low();
            }
        }

        // Update LED if present (follows fan state by default)
        if let Some(led_pin) = &mut self.gpio_led_pin {
            if speed > 0.1 {
                led_pin.set_high();
            } else {
                led_pin.set_low();
            }
        }

        self.current_speed = speed;
        Ok(())
    }

    pub fn get_speed(&self) -> f64 {
        self.current_speed
    }

    pub fn shutdown(&mut self) -> Result<()> {
        // Turn off fan
        self.set_speed(0.0)?;
        
        if let Some(pwm) = &self.pwm {
            pwm.disable()?;
        }
        
        Ok(())
    }
}


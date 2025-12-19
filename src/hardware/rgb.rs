/// RGB LED controller for WS2812 LEDs via SPI
use anyhow::{Context, Result};
use palette::{FromColor, Hsv, Srgb};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use serde_json::Value;
use smart_leds::RGB8;
use std::time::{Duration, Instant};
use ws2812_spi::Ws2812;

use super::SystemStatus;

const DEFAULT_LED_COUNT: usize = 4;
const DEFAULT_BRIGHTNESS: u8 = 50;

pub struct RgbController {
    ws2812: Ws2812<Spi>,
    led_count: usize,
    brightness: u8,
    color: RGB8,
    style: RgbStyle,
    speed: u8,
    enabled: bool,
    animation_state: AnimationState,
}

#[derive(Debug, Clone, Copy)]
pub enum RgbStyle {
    Solid,
    Breathing,
    Rainbow,
    Chase,
    Pulse,
}

impl RgbStyle {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "solid" => RgbStyle::Solid,
            "breathing" => RgbStyle::Breathing,
            "rainbow" => RgbStyle::Rainbow,
            "chase" => RgbStyle::Chase,
            "pulse" => RgbStyle::Pulse,
            _ => RgbStyle::Breathing,
        }
    }
}

struct AnimationState {
    start_time: Instant,
    phase: f32,
}

impl RgbController {
    pub fn new(config: &Value) -> Result<Self> {
        // Initialize SPI for WS2812 control
        // WS2812 requires 6.4 MHz SPI clock for proper timing
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 6_400_000, Mode::Mode0)
            .context("Failed to initialize SPI for RGB LEDs")?;

        let ws2812 = Ws2812::new(spi);

        // Parse configuration
        let system = &config["system"];
        let led_count = system["rgb_led_count"]
            .as_u64()
            .unwrap_or(DEFAULT_LED_COUNT as u64) as usize;
        let brightness = system["rgb_brightness"]
            .as_u64()
            .unwrap_or(DEFAULT_BRIGHTNESS as u64) as u8;
        let enabled = system["rgb_enable"].as_bool().unwrap_or(true);
        
        // Parse color (hex string like "#0a1aff")
        let color_str = system["rgb_color"]
            .as_str()
            .unwrap_or("#0a1aff");
        let color = parse_hex_color(color_str)?;
        
        let style = RgbStyle::from_str(
            system["rgb_style"].as_str().unwrap_or("breathing")
        );
        
        let speed = system["rgb_speed"]
            .as_u64()
            .unwrap_or(50) as u8;

        let animation_state = AnimationState {
            start_time: Instant::now(),
            phase: 0.0,
        };

        Ok(RgbController {
            ws2812,
            led_count,
            brightness,
            color,
            style,
            speed,
            enabled,
            animation_state,
        })
    }

    pub fn update(&mut self, _status: &SystemStatus) -> Result<()> {
        if !self.enabled {
            // Turn off all LEDs
            let black = vec![RGB8::new(0, 0, 0); self.led_count];
            self.ws2812.write(black.iter().cloned())
                .context("Failed to write to RGB LEDs")?;
            return Ok(());
        }

        // Generate LED colors based on style
        let colors = match self.style {
            RgbStyle::Solid => self.generate_solid(),
            RgbStyle::Breathing => self.generate_breathing(),
            RgbStyle::Rainbow => self.generate_rainbow(),
            RgbStyle::Chase => self.generate_chase(),
            RgbStyle::Pulse => self.generate_pulse(),
        };

        // Write to LEDs
        self.ws2812.write(colors.iter().cloned())
            .context("Failed to write to RGB LEDs")?;

        Ok(())
    }

    fn generate_solid(&self) -> Vec<RGB8> {
        let scaled = self.scale_brightness(self.color);
        vec![scaled; self.led_count]
    }

    fn generate_breathing(&mut self) -> Vec<RGB8> {
        let elapsed = self.animation_state.start_time.elapsed().as_secs_f32();
        let speed_factor = self.speed as f32 / 50.0;
        let phase = (elapsed * speed_factor).sin() * 0.5 + 0.5; // 0.0 to 1.0
        
        let brightness = (self.brightness as f32 * phase) as u8;
        let scaled = RGB8::new(
            (self.color.r as f32 * phase) as u8,
            (self.color.g as f32 * phase) as u8,
            (self.color.b as f32 * phase) as u8,
        );
        
        vec![scaled; self.led_count]
    }

    fn generate_rainbow(&mut self) -> Vec<RGB8> {
        let elapsed = self.animation_state.start_time.elapsed().as_secs_f32();
        let speed_factor = self.speed as f32 / 50.0;
        
        (0..self.led_count)
            .map(|i| {
                let hue = ((elapsed * speed_factor * 60.0) + (i as f32 * 360.0 / self.led_count as f32)) % 360.0;
                let hsv = Hsv::new(hue, 1.0, self.brightness as f32 / 100.0);
                let rgb = Srgb::from_color(hsv);
                RGB8::new(
                    (rgb.red * 255.0) as u8,
                    (rgb.green * 255.0) as u8,
                    (rgb.blue * 255.0) as u8,
                )
            })
            .collect()
    }

    fn generate_chase(&mut self) -> Vec<RGB8> {
        let elapsed = self.animation_state.start_time.elapsed().as_secs_f32();
        let speed_factor = self.speed as f32 / 50.0;
        let position = ((elapsed * speed_factor * 2.0) as usize) % self.led_count;
        
        (0..self.led_count)
            .map(|i| {
                if i == position {
                    self.scale_brightness(self.color)
                } else {
                    RGB8::new(0, 0, 0)
                }
            })
            .collect()
    }

    fn generate_pulse(&mut self) -> Vec<RGB8> {
        let elapsed = self.animation_state.start_time.elapsed().as_secs_f32();
        let speed_factor = self.speed as f32 / 50.0;
        let phase = ((elapsed * speed_factor * 2.0).sin() * 0.5 + 0.5).powi(2);
        
        let scaled = RGB8::new(
            (self.color.r as f32 * phase) as u8,
            (self.color.g as f32 * phase) as u8,
            (self.color.b as f32 * phase) as u8,
        );
        
        vec![scaled; self.led_count]
    }

    fn scale_brightness(&self, color: RGB8) -> RGB8 {
        let scale = self.brightness as f32 / 100.0;
        RGB8::new(
            (color.r as f32 * scale) as u8,
            (color.g as f32 * scale) as u8,
            (color.b as f32 * scale) as u8,
        )
    }

    pub fn shutdown(&mut self) -> Result<()> {
        // Turn off all LEDs
        let black = vec![RGB8::new(0, 0, 0); self.led_count];
        self.ws2812.write(black.iter().cloned())
            .context("Failed to turn off RGB LEDs")?;
        Ok(())
    }
}

fn parse_hex_color(hex: &str) -> Result<RGB8> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        anyhow::bail!("Invalid hex color format: {}", hex);
    }
    
    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;
    
    Ok(RGB8::new(r, g, b))
}


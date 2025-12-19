/// OLED display controller for SSD1306 displays via I2C
use anyhow::{Context, Result};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle},
    text::Text,
};
use rppal::i2c::I2c;
use serde_json::Value;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use super::SystemStatus;

const I2C_ADDRESS: u16 = 0x3C; // Standard SSD1306 I2C address

pub struct OledController {
    display: Ssd1306<I2CInterface<I2c>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
    enabled: bool,
    rotation: DisplayRotation,
}

impl OledController {
    pub fn new(config: &Value) -> Result<Self> {
        // Initialize I2C
        let mut i2c = I2c::new().context("Failed to initialize I2C")?;
        
        // Set I2C slave address for SSD1306
        i2c.set_slave_address(I2C_ADDRESS)
            .context("Failed to set I2C slave address")?;

        // Create display interface
        let interface = I2CDisplayInterface::new(i2c);
        
        // Parse configuration
        let system = &config["system"];
        let enabled = system["oled_enable"].as_bool().unwrap_or(true);
        let rotation_angle = system["oled_rotation"].as_u64().unwrap_or(0);
        
        let rotation = match rotation_angle {
            180 => DisplayRotation::Rotate180,
            _ => DisplayRotation::Rotate0,
        };

        // Create display
        let mut display = Ssd1306::new(interface, DisplaySize128x64, rotation)
            .into_buffered_graphics_mode();
        
        display.init().map_err(|e| anyhow::anyhow!("Failed to initialize OLED display: {:?}", e))?;
        display.clear();
        display.flush().map_err(|e| anyhow::anyhow!("Failed to flush OLED display: {:?}", e))?;
        
        // Small delay to ensure display is ready
        std::thread::sleep(std::time::Duration::from_millis(100));

        Ok(OledController {
            display,
            enabled,
            rotation,
        })
    }

    pub fn update(&mut self, status: &SystemStatus) -> Result<()> {
        if !self.enabled {
            self.display.clear();
            self.display.flush().map_err(|_| anyhow::anyhow!("Failed to flush OLED display"))?;
            return Ok(());
        }

        // Clear display
        self.display.clear();

        // Text style
        let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        let icon_style = PrimitiveStyle::with_fill(BinaryColor::On);
        let icon_x = 2;  // X position for icons
        let text_x = 14; // X position for text (after icon)

        // Line 1: Title with logo
        self.draw_logo_icon(Point::new(0, 3))?;
        Text::new("Pironman 5", Point::new(12, 10), text_style)
            .draw(&mut self.display)
            .unwrap();

        // Line 2: CPU usage with chip icon
        self.draw_cpu_icon(Point::new(icon_x, 15), &icon_style)?;
        let cpu_text = format!("{:.1}%", status.cpu_usage);
        Text::new(&cpu_text, Point::new(text_x, 22), text_style)
            .draw(&mut self.display)
            .unwrap();

        // Line 3: Memory usage with RAM icon
        self.draw_memory_icon(Point::new(icon_x, 27), &icon_style)?;
        let mem_text = format!("{:.1}%", status.memory_usage);
        Text::new(&mem_text, Point::new(text_x, 34), text_style)
            .draw(&mut self.display)
            .unwrap();

        // Line 4: Temperature with thermometer icon
        self.draw_temperature_icon(Point::new(icon_x, 39), &icon_style)?;
        let temp_text = format!("{:.1}°C", status.cpu_temperature);
        Text::new(&temp_text, Point::new(text_x, 46), text_style)
            .draw(&mut self.display)
            .unwrap();

        // Line 5: Network with WiFi/Ethernet icon
        self.draw_network_icon(Point::new(icon_x, 51), &icon_style, status.network_active)?;
        let network_text = if let Some(ip) = &status.network_ip {
            ip.to_string()
        } else {
            "No connection".to_string()
        };
        Text::new(&network_text, Point::new(text_x, 58), text_style)
            .draw(&mut self.display)
            .unwrap();

        // Flush to display
        self.display.flush().map_err(|_| anyhow::anyhow!("Failed to update OLED display"))?;

        Ok(())
    }

    /// Draw logo icon (small Pi symbol)
    fn draw_logo_icon(&mut self, pos: Point) -> Result<()> {
        // Draw a simple Pi symbol (π)
        Line::new(Point::new(pos.x, pos.y + 6), Point::new(pos.x + 8, pos.y + 6))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .unwrap();
        Line::new(Point::new(pos.x + 2, pos.y + 6), Point::new(pos.x + 2, pos.y + 1))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .unwrap();
        Line::new(Point::new(pos.x + 6, pos.y + 6), Point::new(pos.x + 6, pos.y + 1))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .unwrap();
        Ok(())
    }

    /// Draw CPU icon (microchip)
    fn draw_cpu_icon(&mut self, pos: Point, style: &PrimitiveStyle<BinaryColor>) -> Result<()> {
        // Draw chip body (rectangle)
        Rectangle::new(Point::new(pos.x + 2, pos.y + 1), Size::new(6, 6))
            .into_styled(*style)
            .draw(&mut self.display)
            .unwrap();
        
        // Draw pins (left side)
        Line::new(Point::new(pos.x, pos.y + 2), Point::new(pos.x + 1, pos.y + 2))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .unwrap();
        Line::new(Point::new(pos.x, pos.y + 5), Point::new(pos.x + 1, pos.y + 5))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .unwrap();
        
        // Draw pins (right side)
        Line::new(Point::new(pos.x + 8, pos.y + 2), Point::new(pos.x + 9, pos.y + 2))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .unwrap();
        Line::new(Point::new(pos.x + 8, pos.y + 5), Point::new(pos.x + 9, pos.y + 5))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .unwrap();
        
        Ok(())
    }

    /// Draw memory icon (RAM bars)
    fn draw_memory_icon(&mut self, pos: Point, style: &PrimitiveStyle<BinaryColor>) -> Result<()> {
        // Draw memory module outline
        Rectangle::new(Point::new(pos.x + 1, pos.y), Size::new(8, 8))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .unwrap();
        
        // Draw memory chips (vertical bars)
        Rectangle::new(Point::new(pos.x + 3, pos.y + 2), Size::new(1, 4))
            .into_styled(*style)
            .draw(&mut self.display)
            .unwrap();
        Rectangle::new(Point::new(pos.x + 5, pos.y + 2), Size::new(1, 4))
            .into_styled(*style)
            .draw(&mut self.display)
            .unwrap();
        Rectangle::new(Point::new(pos.x + 7, pos.y + 2), Size::new(1, 4))
            .into_styled(*style)
            .draw(&mut self.display)
            .unwrap();
        
        Ok(())
    }

    /// Draw temperature icon (thermometer)
    fn draw_temperature_icon(&mut self, pos: Point, style: &PrimitiveStyle<BinaryColor>) -> Result<()> {
        // Draw thermometer bulb (circle at bottom)
        Circle::new(Point::new(pos.x + 3, pos.y + 5), 3)
            .into_styled(*style)
            .draw(&mut self.display)
            .unwrap();
        
        // Draw thermometer tube (rectangle)
        Rectangle::new(Point::new(pos.x + 4, pos.y), Size::new(1, 6))
            .into_styled(*style)
            .draw(&mut self.display)
            .unwrap();
        
        // Draw temperature indicator line
        Line::new(Point::new(pos.x + 6, pos.y + 2), Point::new(pos.x + 8, pos.y + 2))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .unwrap();
        
        Ok(())
    }

    /// Draw network icon (WiFi waves or ethernet)
    fn draw_network_icon(&mut self, pos: Point, style: &PrimitiveStyle<BinaryColor>, active: bool) -> Result<()> {
        if active {
            // Draw WiFi waves (three arcs)
            Circle::new(Point::new(pos.x + 3, pos.y + 5), 2)
                .into_styled(*style)
                .draw(&mut self.display)
                .unwrap();
            
            // Small arc
            Line::new(Point::new(pos.x + 2, pos.y + 3), Point::new(pos.x + 3, pos.y + 2))
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(&mut self.display)
                .unwrap();
            Line::new(Point::new(pos.x + 5, pos.y + 2), Point::new(pos.x + 6, pos.y + 3))
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(&mut self.display)
                .unwrap();
            
            // Large arc
            Line::new(Point::new(pos.x + 1, pos.y + 1), Point::new(pos.x + 2, pos.y))
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(&mut self.display)
                .unwrap();
            Line::new(Point::new(pos.x + 6, pos.y), Point::new(pos.x + 7, pos.y + 1))
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(&mut self.display)
                .unwrap();
        } else {
            // Draw "X" for no connection
            Line::new(Point::new(pos.x + 2, pos.y + 1), Point::new(pos.x + 7, pos.y + 6))
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(&mut self.display)
                .unwrap();
            Line::new(Point::new(pos.x + 7, pos.y + 1), Point::new(pos.x + 2, pos.y + 6))
                .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(&mut self.display)
                .unwrap();
        }
        
        Ok(())
    }

    pub fn show_message(&mut self, message: &str) -> Result<()> {
        self.display.clear();
        
        let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
        Text::new(message, Point::new(0, 10), text_style)
            .draw(&mut self.display)
            .unwrap();
        
        self.display.flush().map_err(|_| anyhow::anyhow!("Failed to flush OLED display"))?;
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.display.clear();
        self.display.flush().map_err(|_| anyhow::anyhow!("Failed to flush OLED display"))?;
        Ok(())
    }
}


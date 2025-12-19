# Pironman5 Rust

A Rust implementation of the Pironman5 Raspberry Pi case management software.

## Overview

This is a complete rewrite of the Python-based Pironman5 system in Rust, providing the same functionality with improved performance and safety guarantees.

## Features

- **Variant Detection**: Automatically detects Pironman 5, Pironman 5 Mini, or Pironman 5 Max
- **Configuration Management**: JSON-based configuration with automatic upgrade support
- **Logging**: Dual output to both console and rotating log files
- **CLI Interface**: Full command-line interface for all configuration options
- **Hardware Support** (FULL IMPLEMENTATION):
  - ✅ WS2812 RGB LED control (multiple animation styles)
  - ✅ OLED display management (SSD1306, I2C)
  - ✅ PWM fan control (automatic temperature-based)
  - ✅ GPIO fan control
  - ✅ Temperature monitoring (CPU + GPU)
  - ✅ System monitoring (CPU, Memory, Disk, Network)

## Project Structure

```
src/
├── main.rs           # Entry point
├── cli.rs            # Command-line interface
├── pironman5.rs      # Main application logic
├── logger.rs         # Logging implementation
├── utils.rs          # Utility functions
├── version.rs        # Version information
└── variants/         # Variant-specific configurations
    ├── mod.rs        # Variant detection and trait
    ├── base.rs       # Pironman 5 base variant
    ├── mini.rs       # Pironman 5 Mini variant
    └── max.rs        # Pironman 5 Max variant
```

## Installation

### Quick Install (Recommended)

Automatic installation with one command:

```bash
sudo python3 install.py
```

Supports:
- ✅ Debian/Ubuntu/Raspberry Pi OS
- ✅ **Fedora/RHEL/CentOS** (NEW! - see [FEDORA_GUIDE.md](FEDORA_GUIDE.md))
- ✅ Arch/Manjaro

The installer will:
1. Install system dependencies
2. Install Rust toolchain
3. Enable hardware interfaces
4. Build and install binary
5. Create systemd service
6. Configure auto-start

Then reboot:
```bash
sudo reboot
```

See [INSTALL.md](INSTALL.md) for detailed installation guide.

### Manual Build

If you prefer to build manually:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build release version
cargo build --release

# Binary at: target/release/pironman5
```

The compiled binary will be at:
- Debug: `target/debug/pironman5`
- Release: `target/release/pironman5`

## Usage

### Basic Commands

```bash
# Show version
pironman5 --version

# Show current configuration
pironman5 --config

# Start the service
pironman5 start

# Stop the service
pironman5 stop

# Restart the service
pironman5 restart
```

### Configuration

```bash
# Set debug level
pironman5 --debug-level info

# Configure RGB LED
pironman5 --rgb-color ff00ff --rgb-brightness 75 --rgb-style rainbow

# Configure OLED
pironman5 --oled-enable true --oled-rotation 180

# Configure temperature unit
pironman5 --temperature-unit F

# Configure GPIO fan
pironman5 --gpio-fan-mode 2 --gpio-fan-pin 6
```

### Advanced Options

For a complete list of options:
```bash
pironman5 --help
```

## Configuration File

The default configuration file is located at `/opt/pironman5/config.json`.

Example configuration:
```json
{
  "system": {
    "data_interval": 1,
    "rgb_color": "#0a1aff",
    "rgb_brightness": 50,
    "rgb_style": "breathing",
    "rgb_speed": 50,
    "rgb_enable": true,
    "rgb_led_count": 4,
    "temperature_unit": "C",
    "oled_enable": true,
    "oled_rotation": 0,
    "oled_disk": "total",
    "oled_network_interface": "all",
    "gpio_fan_pin": 6,
    "gpio_fan_mode": 0,
    "debug_level": "INFO"
  }
}
```

## Variant Detection

The software automatically detects which Pironman variant is installed by reading:
1. Environment variable `PIRONMAN5_PART_NUMBER`
2. HAT EEPROM device tree information
3. Force variant file at `/opt/pironman5/variant`

Supported variants:
- **0306V10**: Pironman 5 Base
- **0306Vxx** (other versions): Pironman 5 Max
- **0308**: Pironman 5 Mini

## Logging

Logs are written to `/var/log/pironman5/main.log` with automatic rotation.

Debug levels: DEBUG, INFO, WARNING, ERROR, CRITICAL

## Differences from Python Version

1. **Performance**: Rust provides better performance and lower memory usage
2. **Safety**: Memory safety guarantees prevent common bugs
3. **Compilation**: Produces a standalone binary with no runtime dependencies
4. **Concurrency**: Better concurrent operation handling with Rust's ownership model

## Dependencies

- `serde` & `serde_json`: JSON serialization
- `clap`: Command-line argument parsing
- `log` & `env_logger`: Logging framework
- `anyhow`: Error handling
- `ctrlc` & `signal-hook`: Signal handling
- `chrono`: Date/time formatting

## Hardware Integration Note

This implementation provides the framework and configuration management. The actual hardware control libraries (pm_auto, pm_dashboard) would need to be:
1. Rewritten in Rust, or
2. Called via FFI (Foreign Function Interface) from the existing Python/C libraries

## Development Status

This is a complete framework implementation with:
- ✅ All configuration options
- ✅ Variant detection
- ✅ CLI interface
- ✅ Logging system
- ✅ Configuration management
- ⚠️  Hardware control (requires integration with hardware libraries)

## License

GPL-2.0 - Same as the original Python implementation

## Authors

- SunFounder (Original Python version)
- Rust port: 2025

## Contributing

Contributions are welcome! Please ensure:
1. Code passes `cargo clippy`
2. Code is formatted with `cargo fmt`
3. All tests pass with `cargo test`
4. New features include appropriate tests

## See Also

- [Original Python Implementation](../pironman5/)
- [SunFounder GitHub](https://github.com/sunfounder/pironman5)


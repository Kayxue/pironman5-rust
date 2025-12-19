# Installation Guide

Complete installation guide for Pironman5 Rust on various Linux distributions.

## Supported Systems

✅ **Debian-based**
- Debian 10+
- Ubuntu 20.04+
- Raspberry Pi OS (Bullseye, Bookworm)
- Linux Mint

✅ **Fedora-based** (NEW!)
- Fedora 36+
- RHEL 8+
- CentOS 8+
- Rocky Linux
- AlmaLinux

✅ **Arch-based**
- Arch Linux
- Manjaro

## Quick Install

### Automatic Installation (Recommended)

```bash
# Download and run installer
cd pironman5_rust
sudo python3 install.py
```

The installer will:
1. ✅ Install system dependencies
2. ✅ Install Rust toolchain (if needed)
3. ✅ Enable hardware interfaces (SPI, I2C, PWM)
4. ✅ Build optimized binary (takes 5-10 min)
5. ✅ Install to `/usr/local/bin/pironman5`
6. ✅ Create systemd service
7. ✅ Configure auto-start on boot

**Then reboot:**
```bash
sudo reboot
```

After reboot, check status:
```bash
sudo systemctl status pironman5
```

## Manual Installation

### 1. Install System Dependencies

#### Debian/Ubuntu/Raspberry Pi OS
```bash
sudo apt update
sudo apt install -y curl build-essential pkg-config libudev-dev i2c-tools
```

#### Fedora
```bash
sudo dnf install -y curl gcc gcc-c++ make pkgconfig systemd-devel i2c-tools
```

#### RHEL/CentOS/Rocky/AlmaLinux
```bash
# Enable EPEL
sudo yum install -y epel-release

# Install packages
sudo yum install -y curl gcc gcc-c++ make pkgconfig systemd-devel i2c-tools
```

#### Arch/Manjaro
```bash
sudo pacman -Sy --noconfirm curl base-devel pkg-config systemd i2c-tools
```

### 2. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify:
```bash
cargo --version
rustc --version
```

### 3. Enable Hardware Interfaces (Raspberry Pi only)

#### Raspberry Pi OS (Debian-based)

```bash
# Enable I2C
sudo raspi-config nonint do_i2c 0

# Enable SPI
sudo raspi-config nonint do_spi 0

# Enable hardware PWM
echo "dtoverlay=pwm-2chan" | sudo tee -a /boot/config.txt
# Or for newer Pi OS:
echo "dtoverlay=pwm-2chan" | sudo tee -a /boot/firmware/config.txt
```

#### Fedora on Raspberry Pi

For Fedora, edit `/boot/efi/config.txt` directly:

```bash
sudo nano /boot/efi/config.txt
```

Add these lines (if not already present):

```ini
# Pironman5 hardware interfaces
dtparam=i2c_arm=on
dtparam=spi=on
dtoverlay=pwm-2chan
```

**Note:** Based on [RPM Fusion Raspberry Pi documentation](https://rpmfusion.org/Howto/RaspberryPi), Fedora 42+ already includes basic overlays, but Pironman5-specific interfaces still need manual configuration.

### 4. Build Binary

```bash
cd pironman5_rust
cargo build --release
```

This takes 5-10 minutes. Output: `target/release/pironman5`

### 5. Install Binary

```bash
sudo cp target/release/pironman5 /usr/local/bin/
sudo chmod +x /usr/local/bin/pironman5
```

Verify:
```bash
pironman5 --version
```

### 6. Create Configuration Directory

```bash
sudo mkdir -p /opt/pironman5
sudo mkdir -p /var/log/pironman5
sudo chmod 755 /var/log/pironman5
```

### 7. Create Systemd Service

Create `/etc/systemd/system/pironman5.service`:

```ini
[Unit]
Description=Pironman5 Hardware Control Service
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/pironman5 start
ExecStop=/usr/local/bin/pironman5 stop
Restart=on-failure
RestartSec=5s

NoNewPrivileges=true
PrivateTmp=true

StandardOutput=journal
StandardError=journal
SyslogIdentifier=pironman5

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable pironman5
sudo systemctl start pironman5
```

### 8. Add User to Groups (Optional)

```bash
sudo usermod -a -G gpio,i2c,spi $USER
```

### 9. Reboot

```bash
sudo reboot
```

## Verification

After reboot:

```bash
# Check service status
sudo systemctl status pironman5

# View logs
sudo journalctl -u pironman5 -f

# Test hardware
pironman5 --config
```

Expected log output:
```
Initializing hardware components...
✓ RGB controller initialized
✓ OLED display initialized
✓ Fan controller initialized
✓ Temperature monitor initialized
Hardware control active
```

## Troubleshooting

### Build Fails

**Error**: "cargo: command not found"
```bash
source $HOME/.cargo/env
# Or
export PATH="$HOME/.cargo/bin:$PATH"
```

**Error**: "failed to run custom build command"
```bash
# Install missing dependencies
sudo apt install build-essential pkg-config libudev-dev  # Debian
sudo dnf install gcc gcc-c++ systemd-devel              # Fedora
```

### Hardware Not Working

**Check interfaces enabled:**
```bash
# I2C
ls /dev/i2c-*

# SPI
ls /dev/spidev*

# Scan I2C devices
i2cdetect -y 1
```

**Check permissions:**
```bash
sudo chmod 666 /dev/i2c-1
sudo chmod 666 /dev/spidev0.0
```

**Check PWM:**
```bash
ls /sys/class/pwm/
```

### Service Won't Start

```bash
# Check logs
sudo journalctl -u pironman5 -n 50

# Test manually
sudo /usr/local/bin/pironman5 start

# Check binary permissions
ls -l /usr/local/bin/pironman5
```

### RGB LEDs Not Working

```bash
# Check SPI
lsmod | grep spi

# Load SPI module
sudo modprobe spi_bcm2835

# Test with different speed
# Edit rgb.rs and rebuild if needed
```

### OLED Blank

```bash
# Check I2C address
i2cdetect -y 1
# Should show 0x3C

# Test I2C communication
i2cget -y 1 0x3c 0x00
```

## Uninstallation

### Automatic
```bash
sudo python3 uninstall.py
```

### Manual
```bash
# Stop service
sudo systemctl stop pironman5
sudo systemctl disable pironman5

# Remove files
sudo rm /usr/local/bin/pironman5
sudo rm /etc/systemd/system/pironman5.service
sudo systemctl daemon-reload

# Optional: Remove configuration
sudo rm -rf /opt/pironman5
sudo rm -rf /var/log/pironman5
```

## Cross-Compilation

Build on x86_64 for Raspberry Pi (ARM64):

```bash
# Install cross-compilation tools
rustup target add aarch64-unknown-linux-gnu

# Install ARM linker (Debian/Ubuntu)
sudo apt install gcc-aarch64-linux-gnu

# Build
cargo build --release --target aarch64-unknown-linux-gnu

# Binary at: target/aarch64-unknown-linux-gnu/release/pironman5
```

## Platform-Specific Notes

### Raspberry Pi 5
- Full hardware support ✅
- Uses hardware PWM
- SPI at 6.4 MHz for WS2812
- I2C for OLED (address 0x3C)

### Raspberry Pi 4
- Full hardware support ✅
- Same configuration as Pi 5

### Raspberry Pi 3
- Hardware support ✅
- May need different GPIO pins
- Check pinout diagram

### Fedora on Raspberry Pi
- **Config location**: `/boot/efi/config.txt` (not `/boot/config.txt`)
- **No raspi-config**: Must edit config.txt manually
- **Fedora 42+**: Basic overlays included, but Pironman5 interfaces need manual setup
- **Kernel**: Uses downstream RPi kernel from [Fedora Copr](https://copr.fedorainfracloud.org/coprs/dwrobel/kernel-rpi/)
- See [RPM Fusion Raspberry Pi guide](https://rpmfusion.org/Howto/RaspberryPi) for more details

### Other ARM boards
- Partial support
- GPIO/SPI/I2C may work
- May need pin mapping changes

### x86_64 Linux
- Configuration management works ✅
- Hardware control disabled
- Useful for testing configuration

## Advanced Configuration

### Custom Install Location

```bash
# Build
cargo build --release

# Install to custom location
sudo mkdir -p /opt/pironman5/bin
sudo cp target/release/pironman5 /opt/pironman5/bin/

# Update service file ExecStart path
sudo nano /etc/systemd/system/pironman5.service
# Change to: ExecStart=/opt/pironman5/bin/pironman5 start
```

### Multiple Instances

Run multiple instances with different configs:

```bash
# Create configs
sudo mkdir -p /opt/pironman5/instance1
sudo mkdir -p /opt/pironman5/instance2

# Run with different configs
pironman5 --config-path /opt/pironman5/instance1/config.json start
pironman5 --config-path /opt/pironman5/instance2/config.json start
```

### Development Build

For faster compilation during development:

```bash
cargo build  # Debug build (faster compile, slower runtime)
./target/debug/pironman5 start
```

## Distribution Packages

### Create Debian Package

```bash
cargo install cargo-deb
cargo deb

# Install
sudo dpkg -i target/debian/pironman5_*.deb
```

### Create RPM Package

```bash
cargo install cargo-generate-rpm
cargo build --release
cargo generate-rpm

# Install
sudo rpm -i target/generate-rpm/pironman5-*.rpm
```

## Update

To update to a new version:

```bash
# Pull latest code
git pull

# Rebuild
cargo build --release

# Stop service
sudo systemctl stop pironman5

# Install new binary
sudo cp target/release/pironman5 /usr/local/bin/

# Start service
sudo systemctl start pironman5
```

## Support

For issues:
1. Check logs: `sudo journalctl -u pironman5 -f`
2. Test manually: `sudo pironman5 start`
3. Check hardware: `i2cdetect -y 1`, `ls /dev/spi*`
4. Review documentation: `HARDWARE.md`
5. Check permissions: `ls -l /dev/i2c-1 /dev/spidev0.0`

## Performance Tuning

### Reduce CPU Usage

Edit `/opt/pironman5/config.json`:
```json
{
  "system": {
    "data_interval": 2,  // Update every 2 seconds instead of 1
    ...
  }
}
```

### Disable Features

Turn off unused features:
```bash
pironman5 --rgb-enable false
pironman5 --oled-enable false
```

## Next Steps

After installation:
1. Configure settings: `pironman5 --help`
2. Check status: `sudo systemctl status pironman5`
3. View logs: `sudo journalctl -u pironman5 -f`
4. Read hardware guide: `HARDWARE.md`
5. Customize configuration: `/opt/pironman5/config.json`


# Fedora on Raspberry Pi - Pironman5 Guide

Complete guide for installing and configuring Pironman5 on Fedora (Raspberry Pi).

## Overview

Fedora on Raspberry Pi uses a different configuration approach than Raspberry Pi OS:
- ✅ No `raspi-config` utility
- ✅ Config file at `/boot/efi/config.txt` (UEFI boot)
- ✅ Uses downstream Raspberry Pi kernel
- ✅ Manual device tree overlay configuration

Based on [RPM Fusion Raspberry Pi documentation](https://rpmfusion.org/Howto/RaspberryPi).

## Supported Versions

| Fedora Version | Status | Notes |
|---------------|--------|-------|
| Fedora 43 | ✅ Recommended | Includes basic overlays |
| Fedora 42 | ✅ Supported | Includes basic overlays |
| Fedora 41 | ⚠️ Requires manual config | Need to add VC4 driver manually |

## Quick Install

```bash
# Download Fedora for Raspberry Pi
# From: https://rpmfusion.org/Howto/RaspberryPi

# Flash to SD card, boot, then:
sudo dnf update -y
cd /path/to/pironman5_rust
sudo python3 install.py
```

The installer will:
1. Detect Fedora automatically
2. Install dependencies via `dnf`
3. Configure `/boot/efi/config.txt` for hardware
4. Build and install Pironman5

## Manual Configuration

### 1. Install System Dependencies

```bash
sudo dnf install -y curl gcc gcc-c++ make pkgconfig systemd-devel i2c-tools
```

### 2. Enable Hardware Interfaces

Edit `/boot/efi/config.txt`:

```bash
sudo nano /boot/efi/config.txt
```

Add these lines for Pironman5:

```ini
# Pironman5 hardware interfaces
dtparam=i2c_arm=on      # For OLED display (I2C)
dtparam=spi=on          # For RGB LEDs (SPI)
dtoverlay=pwm-2chan     # For fan control (PWM)
```

For Raspberry Pi 5 on Fedora 41, also add:

```ini
[pi5]
# Enable DRM VC4 V3D driver
dtoverlay=vc4-kms-v3d
```

**Save and reboot:**
```bash
sudo reboot
```

### 3. Verify Hardware Interfaces

After reboot:

```bash
# Check I2C
ls /dev/i2c-*
# Should show: /dev/i2c-1

# Check SPI  
ls /dev/spidev*
# Should show: /dev/spidev0.0

# Scan I2C devices (OLED should be at 0x3C)
sudo i2cdetect -y 1

# Check PWM
ls /sys/class/pwm/
```

### 4. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 5. Build and Install

```bash
cd pironman5_rust
cargo build --release
sudo cp target/release/pironman5 /usr/local/bin/
sudo chmod +x /usr/local/bin/pironman5
```

### 6. Create Systemd Service

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

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable pironman5
sudo systemctl start pironman5
```

## Fedora-Specific Configuration

### Config File Locations

| OS | Config Location |
|----|----------------|
| Raspberry Pi OS | `/boot/config.txt` or `/boot/firmware/config.txt` |
| Fedora | `/boot/efi/config.txt` |

### Package Management

```bash
# Update system
sudo dnf update -y

# Install package
sudo dnf install <package>

# Search package
sudo dnf search <package>

# Remove package
sudo dnf remove <package>
```

### Kernel Information

Fedora on Raspberry Pi uses a downstream kernel:

```bash
# Check kernel version
uname -r

# Should show something like:
# 6.6.x-xxx.rpifork.fc43.aarch64
```

Source: [Fedora Copr - kernel-rpi](https://copr.fedorainfracloud.org/coprs/dwrobel/kernel-rpi/)

### SELinux Considerations

Fedora has SELinux enabled by default. If you encounter permission issues:

```bash
# Check SELinux status
sudo getenforce

# Temporarily disable (testing only)
sudo setenforce 0

# Permanently disable (not recommended)
sudo nano /etc/selinux/config
# Change: SELINUX=disabled

# Better: Add SELinux policy for Pironman5
sudo ausearch -c 'pironman5' --raw | audit2allow -M pironman5-policy
sudo semodule -i pironman5-policy.pp
```

## Troubleshooting

### Config.txt Not Found

```bash
# Check mount points
mount | grep boot

# Should show:
# /dev/mmcblk0p1 on /boot/efi type vfat ...

# If not mounted:
sudo mount /dev/mmcblk0p1 /boot/efi
```

### I2C/SPI Not Available

```bash
# Load modules manually
sudo modprobe i2c-dev
sudo modprobe spi-bcm2835

# Add to /etc/modules-load.d/pironman5.conf
echo "i2c-dev" | sudo tee /etc/modules-load.d/pironman5.conf
echo "spi-bcm2835" | sudo tee -a /etc/modules-load.d/pironman5.conf
```

### Permission Denied on Hardware

```bash
# Add user to groups
sudo usermod -a -G i2c,spi,gpio $USER

# Or run as root
sudo pironman5 start
```

### Build Fails with "cargo not found"

```bash
# Add cargo to PATH
source $HOME/.cargo/env

# Or add to .bashrc
echo 'source $HOME/.cargo/env' >> ~/.bashrc
```

### Kernel Panic or Boot Issues

If hardware configuration causes boot issues:

1. Remove SD card
2. Mount on another system
3. Edit `/boot/efi/config.txt`
4. Comment out problematic lines with `#`
5. Reboot

## Performance Comparison

| Metric | Raspberry Pi OS | Fedora |
|--------|----------------|--------|
| Boot Time | ~30s | ~40s |
| Memory Usage | ~200 MB | ~300 MB |
| Package Manager | apt | dnf |
| Init System | systemd | systemd |
| SELinux | Optional | Default |

## Fedora Workstation vs Server

### Workstation
- Desktop environment (GNOME)
- Recommended for Pi 4B/5/400 with ≥4GB RAM
- Graphical configuration tools
- ~2 GB disk space

### Server
- Minimal installation
- Works from Pi 3 B/B+
- Command-line only
- ~1 GB disk space

**For Pironman5**, Server edition is sufficient.

## Updating Fedora

```bash
# Update all packages
sudo dnf update -y

# Upgrade to new Fedora version
sudo dnf system-upgrade download --releasever=43
sudo dnf system-upgrade reboot
```

## Getting Fedora Images

Pre-built images available at:
- [Fedora 43 Server](https://bit.ly/42XXroz)
- [Fedora 43 Workstation](https://bit.ly/4p0tJIf)
- [Fedora 42 Server](https://bit.ly/4jw0xFU)
- [Fedora 42 Workstation](https://bit.ly/4j9B0CG)

Source: [RPM Fusion - Raspberry Pi](https://rpmfusion.org/Howto/RaspberryPi)

## Resources

- **Official Fedora ARM**: https://arm.fedoraproject.org/
- **RPM Fusion Guide**: https://rpmfusion.org/Howto/RaspberryPi
- **Kernel Repository**: https://github.com/dwrobel/kernel
- **Fedora Copr**: https://copr.fedorainfracloud.org/coprs/dwrobel/kernel-rpi/

## Common Commands

```bash
# Check service status
sudo systemctl status pironman5

# View logs
sudo journalctl -u pironman5 -f

# Restart service
sudo systemctl restart pironman5

# Check Fedora version
cat /etc/fedora-release

# Check available disk space
df -h

# Check system info
hostnamectl

# Network configuration
nmcli device status
```

## Known Issues

1. **Slower boot than Raspberry Pi OS**
   - Fedora is a general-purpose distro
   - More services start at boot
   - Still acceptable for Pironman5

2. **Larger disk footprint**
   - Fedora: ~2-3 GB
   - Pi OS Lite: ~1 GB
   - Ensure adequate SD card space

3. **SELinux can block GPIO access**
   - May need policy adjustments
   - Or temporarily disable for testing

4. **No raspi-config**
   - Must edit config.txt manually
   - More technical approach

## Advantages of Fedora

✅ **Latest packages** - Fedora gets new software faster  
✅ **SELinux** - Better security out of the box  
✅ **DNF package manager** - Powerful dependency resolution  
✅ **Upstream focus** - Close to mainline kernel (with RPi patches)  
✅ **Professional environment** - Same as Fedora x86_64  

## Disadvantages

⚠️ **Manual configuration** - No raspi-config convenience  
⚠️ **Larger footprint** - More disk space and RAM  
⚠️ **Slower updates** - 6-month release cycle vs rolling Pi OS  
⚠️ **Less Pi-specific tools** - Need to adapt some workflows  

## Recommendation

**Use Fedora if:**
- You're familiar with Fedora/RHEL ecosystem
- You want latest software packages
- You need SELinux security
- You're comfortable with manual configuration

**Use Raspberry Pi OS if:**
- You want easiest setup
- You need maximum performance
- You want best hardware compatibility
- You prefer GUI configuration tools

## Summary

Fedora on Raspberry Pi is **fully supported** by Pironman5 Rust with these key points:

✅ Automatic detection and configuration by installer  
✅ Config file at `/boot/efi/config.txt`  
✅ Manual hardware interface setup required  
✅ DNF package manager support  
✅ Full hardware control works after proper config  
⚠️ Requires reboot after config changes  
⚠️ No raspi-config utility  

**The automatic installer handles all Fedora-specific configuration!**


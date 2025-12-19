#!/usr/bin/env python3
"""
Pironman5 Rust Installer
Supports: Debian, Ubuntu, Raspberry Pi OS, Fedora, RHEL, CentOS
"""

import os
import sys
import subprocess
import platform
import shutil
import time

# Colors for output
class Color:
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    CYAN = '\033[96m'
    WHITE = '\033[97m'
    BOLD = '\033[1m'
    END = '\033[0m'

def print_info(msg):
    print(f"{Color.CYAN}[INFO]{Color.END} {msg}")

def print_success(msg):
    print(f"{Color.GREEN}[SUCCESS]{Color.END} {msg}")

def print_warning(msg):
    print(f"{Color.YELLOW}[WARNING]{Color.END} {msg}")

def print_error(msg):
    print(f"{Color.RED}[ERROR]{Color.END} {msg}")

def print_step(step, total, msg):
    print(f"\n{Color.BOLD}{Color.MAGENTA}[Step {step}/{total}]{Color.END} {Color.BOLD}{msg}{Color.END}")

def run_command(cmd, check=True, shell=False):
    """Run a command and return the result"""
    try:
        if shell:
            result = subprocess.run(cmd, shell=True, check=check, 
                                  capture_output=True, text=True)
        else:
            result = subprocess.run(cmd, check=check, 
                                  capture_output=True, text=True)
        return result
    except subprocess.CalledProcessError as e:
        if check:
            print_error(f"Command failed: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
            print_error(f"Error: {e.stderr}")
            raise
        return e

def detect_os():
    """Detect the operating system and package manager"""
    if not sys.platform.startswith('linux'):
        print_error("This installer only supports Linux systems")
        sys.exit(1)
    
    # Try to read /etc/os-release
    distro = "unknown"
    pkg_manager = None
    
    if os.path.exists('/etc/os-release'):
        with open('/etc/os-release', 'r') as f:
            for line in f:
                if line.startswith('ID='):
                    distro = line.split('=')[1].strip().strip('"').lower()
                    break
    
    # Determine package manager
    if distro in ['debian', 'ubuntu', 'raspbian', 'linuxmint']:
        pkg_manager = 'apt'
    elif distro in ['fedora', 'rhel', 'centos', 'rocky', 'almalinux']:
        pkg_manager = 'dnf' if shutil.which('dnf') else 'yum'
    elif distro in ['arch', 'manjaro']:
        pkg_manager = 'pacman'
    else:
        # Try to detect by available commands
        if shutil.which('apt'):
            pkg_manager = 'apt'
            distro = 'debian-based'
        elif shutil.which('dnf'):
            pkg_manager = 'dnf'
            distro = 'fedora-based'
        elif shutil.which('yum'):
            pkg_manager = 'yum'
            distro = 'rhel-based'
        elif shutil.which('pacman'):
            pkg_manager = 'pacman'
            distro = 'arch-based'
    
    return distro, pkg_manager

def check_root():
    """Check if running as root"""
    if os.geteuid() != 0:
        print_error("This script must be run as root")
        print_info("Please run: sudo python3 install.py")
        sys.exit(1)

def check_architecture():
    """Check if running on ARM architecture (Raspberry Pi)"""
    arch = platform.machine()
    print_info(f"Detected architecture: {arch}")
    
    if arch not in ['aarch64', 'armv7l', 'armv8']:
        print_warning("This appears to be a non-ARM system")
        print_warning("Hardware control will only work on Raspberry Pi")
        response = input("Continue anyway? (y/n): ")
        if response.lower() != 'y':
            print_info("Installation cancelled")
            sys.exit(0)

def install_dependencies(pkg_manager):
    """Install system dependencies"""
    print_step(1, 7, "Installing system dependencies")
    
    if pkg_manager == 'apt':
        packages = [
            'curl',
            'build-essential',
            'pkg-config',
            'libudev-dev',
            'i2c-tools',
        ]
        
        print_info("Updating package lists...")
        run_command(['apt', 'update'])
        
        print_info("Installing packages...")
        run_command(['apt', 'install', '-y'] + packages)
        
    elif pkg_manager in ['dnf', 'yum']:
        packages = [
            'curl',
            'gcc',
            'gcc-c++',
            'make',
            'pkgconfig',
            'systemd-devel',
            'i2c-tools',
        ]
        
        print_info("Installing packages...")
        run_command([pkg_manager, 'install', '-y'] + packages)
        
        # Enable EPEL for RHEL/CentOS
        if pkg_manager == 'yum':
            try:
                run_command([pkg_manager, 'install', '-y', 'epel-release'], check=False)
            except:
                pass
    
    elif pkg_manager == 'pacman':
        packages = [
            'curl',
            'base-devel',
            'pkg-config',
            'systemd',
            'i2c-tools',
        ]
        
        print_info("Installing packages...")
        run_command(['pacman', '-Sy', '--noconfirm'] + packages)
    
    else:
        print_error(f"Unsupported package manager: {pkg_manager}")
        sys.exit(1)
    
    print_success("System dependencies installed")

def install_rust():
    """Install Rust toolchain if not present"""
    print_step(2, 7, "Installing Rust toolchain")
    
    # Check if Rust is already installed
    if shutil.which('cargo'):
        result = run_command(['cargo', '--version'], check=False)
        if result.returncode == 0:
            print_info(f"Rust already installed: {result.stdout.strip()}")
            return
    
    print_info("Installing Rust via rustup...")
    
    # Download and run rustup installer
    rustup_init = '/tmp/rustup-init.sh'
    run_command(['curl', '--proto', '=https', '--tlsv1.2', '-sSf', 
                'https://sh.rustup.rs', '-o', rustup_init])
    run_command(['chmod', '+x', rustup_init])
    run_command([rustup_init, '-y', '--default-toolchain', 'stable'])
    
    # Source cargo env
    cargo_env = os.path.expanduser('~/.cargo/env')
    if os.path.exists(cargo_env):
        os.environ['PATH'] = f"{os.path.expanduser('~/.cargo/bin')}:{os.environ['PATH']}"
    
    # Verify installation
    if shutil.which('cargo'):
        result = run_command(['cargo', '--version'])
        print_success(f"Rust installed: {result.stdout.strip()}")
    else:
        print_error("Rust installation failed")
        sys.exit(1)

def enable_hardware_interfaces():
    """Enable SPI and I2C interfaces on Raspberry Pi"""
    print_step(3, 7, "Enabling hardware interfaces (SPI, I2C, PWM)")
    
    # Detect if using Raspberry Pi OS or Fedora
    is_fedora = os.path.exists('/etc/fedora-release')
    
    # Find config.txt location
    config_locations = [
        '/boot/efi/config.txt',      # Fedora (UEFI boot)
        '/boot/config.txt',          # Raspberry Pi OS (older)
        '/boot/firmware/config.txt', # Raspberry Pi OS (newer)
    ]
    
    config_file = None
    for location in config_locations:
        if os.path.exists(location):
            config_file = location
            print_info(f"Found config.txt at: {config_file}")
            break
    
    if not config_file:
        print_warning("config.txt not found")
        print_warning("Hardware interfaces may need manual configuration")
        return
    
    # Read current config
    with open(config_file, 'r') as f:
        content = f.read()
    
    # Check what needs to be added
    additions = []
    
    if is_fedora:
        print_info("Detected Fedora - using config.txt method")
        
        # For Fedora, we need to manually enable interfaces in config.txt
        if 'dtparam=i2c_arm=on' not in content and 'dtparam=i2c_arm = on' not in content:
            additions.append('dtparam=i2c_arm=on')
            print_info("Will enable I2C in config.txt")
        else:
            print_info("I2C already enabled")
        
        if 'dtparam=spi=on' not in content and 'dtparam=spi = on' not in content:
            additions.append('dtparam=spi=on')
            print_info("Will enable SPI in config.txt")
        else:
            print_info("SPI already enabled")
    else:
        print_info("Detected Raspberry Pi OS - using raspi-config")
        
        # Use raspi-config if available (Raspberry Pi OS)
        if shutil.which('raspi-config'):
            print_info("Enabling I2C...")
            run_command(['raspi-config', 'nonint', 'do_i2c', '0'], check=False)
            
            print_info("Enabling SPI...")
            run_command(['raspi-config', 'nonint', 'do_spi', '0'], check=False)
        else:
            print_warning("raspi-config not available, using config.txt method")
            if 'dtparam=i2c_arm=on' not in content:
                additions.append('dtparam=i2c_arm=on')
            if 'dtparam=spi=on' not in content:
                additions.append('dtparam=spi=on')
    
    # Check for PWM overlay (needed for both Fedora and Pi OS)
    if 'dtoverlay=pwm-2chan' not in content and 'dtoverlay=pwm' not in content:
        additions.append('dtoverlay=pwm-2chan')
        print_info("Will enable hardware PWM")
    else:
        print_info("PWM already configured")
    
    # Add configurations if needed
    if additions:
        print_info(f"Adding {len(additions)} configuration(s) to {config_file}...")
        with open(config_file, 'a') as f:
            f.write('\n# Pironman5 hardware interfaces\n')
            for item in additions:
                f.write(f'{item}\n')
                print_success(f"Added: {item}")
        print_success("Hardware interfaces configured (requires reboot)")
    else:
        print_info("All hardware interfaces already configured")
    
    # Load I2C module immediately (if not Fedora, as it may not have modprobe)
    if not is_fedora:
        print_info("Loading I2C kernel module...")
        run_command(['modprobe', 'i2c-dev'], check=False)
    
    print_success("Hardware interface configuration complete")

def build_binary():
    """Build the Rust binary"""
    print_step(4, 7, "Building Pironman5 binary")
    
    # Get script directory
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    print_info("Building release binary (this may take 5-10 minutes)...")
    print_info("Compiling optimized Rust code...")
    
    # Update PATH to include cargo
    cargo_bin = os.path.expanduser('~/.cargo/bin')
    os.environ['PATH'] = f"{cargo_bin}:{os.environ['PATH']}"
    
    result = run_command(['cargo', 'build', '--release'], 
                        check=False)
    
    if result.returncode != 0:
        print_error("Build failed!")
        print_error(result.stderr)
        sys.exit(1)
    
    binary_path = os.path.join(script_dir, 'target', 'release', 'pironman5')
    if not os.path.exists(binary_path):
        print_error(f"Binary not found at {binary_path}")
        sys.exit(1)
    
    # Get binary size
    size_mb = os.path.getsize(binary_path) / (1024 * 1024)
    print_success(f"Binary built successfully ({size_mb:.1f} MB)")
    
    return binary_path

def install_binary(binary_path):
    """Install the binary to system path"""
    print_step(5, 7, "Installing binary")
    
    install_dir = '/usr/local/bin'
    target_path = os.path.join(install_dir, 'pironman5')
    
    print_info(f"Installing to {target_path}...")
    shutil.copy2(binary_path, target_path)
    os.chmod(target_path, 0o755)
    
    # Verify installation
    result = run_command([target_path, '--version'])
    print_success(f"Installed: {result.stdout.strip()}")

def setup_config_directory():
    """Create configuration directory"""
    print_step(6, 7, "Setting up configuration")
    
    config_dir = '/opt/pironman5'
    log_dir = '/var/log/pironman5'
    
    print_info(f"Creating {config_dir}...")
    os.makedirs(config_dir, exist_ok=True)
    
    print_info(f"Creating {log_dir}...")
    os.makedirs(log_dir, exist_ok=True)
    os.chmod(log_dir, 0o755)
    
    # Create default config if it doesn't exist
    config_file = os.path.join(config_dir, 'config.json')
    if not os.path.exists(config_file):
        print_info("Creating default configuration...")
        import json
        default_config = {
            "system": {
                "data_interval": 1,
                "rgb_color": "#0a1aff",
                "rgb_brightness": 50,
                "rgb_style": "breathing",
                "rgb_speed": 50,
                "rgb_enable": True,
                "rgb_led_count": 4,
                "temperature_unit": "C",
                "oled_enable": True,
                "oled_rotation": 0,
                "oled_disk": "total",
                "oled_network_interface": "all",
                "gpio_fan_pin": 6,
                "gpio_fan_mode": 1,
                "debug_level": "INFO"
            }
        }
        with open(config_file, 'w') as f:
            json.dump(default_config, f, indent=4)
        print_success("Default configuration created")
    else:
        print_info("Configuration file already exists")
    
    print_success("Configuration setup complete")

def install_systemd_service():
    """Install and enable systemd service"""
    print_step(7, 7, "Installing systemd service")
    
    service_content = """[Unit]
Description=Pironman5 Hardware Control Service
After=network.target sys-devices-platform-soc
Wants=sys-devices-platform-soc

[Service]
Type=simple
User=root
ExecStartPre=/bin/sleep 3
ExecStart=/usr/local/bin/pironman5 start
ExecStop=/usr/local/bin/pironman5 stop
Restart=on-failure
RestartSec=5s

# Security settings
NoNewPrivileges=false
PrivateTmp=false

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=pironman5

[Install]
WantedBy=multi-user.target
"""
    
    service_path = '/etc/systemd/system/pironman5.service'
    
    print_info(f"Creating {service_path}...")
    with open(service_path, 'w') as f:
        f.write(service_content)
    
    print_info("Reloading systemd daemon...")
    run_command(['systemctl', 'daemon-reload'])
    
    print_info("Enabling pironman5 service...")
    run_command(['systemctl', 'enable', 'pironman5.service'])
    
    print_success("Systemd service installed and enabled")

def setup_user_permissions():
    """Add user to required groups"""
    print_info("\nSetting up user permissions...")
    
    # Get the actual user (not root)
    actual_user = os.environ.get('SUDO_USER')
    if not actual_user:
        print_warning("Could not determine actual user, skipping group setup")
        return
    
    groups = ['gpio', 'i2c', 'spi']
    for group in groups:
        # Check if group exists
        try:
            run_command(['getent', 'group', group], check=False)
            run_command(['usermod', '-a', '-G', group, actual_user], check=False)
            print_info(f"Added {actual_user} to {group} group")
        except:
            print_warning(f"Group {group} not found, skipping")

def print_post_install_info():
    """Print post-installation information"""
    print(f"\n{Color.BOLD}{Color.GREEN}{'=' * 60}{Color.END}")
    print(f"{Color.BOLD}{Color.GREEN}Installation Complete!{Color.END}")
    print(f"{Color.BOLD}{Color.GREEN}{'=' * 60}{Color.END}\n")
    
    print(f"{Color.BOLD}Pironman5 has been successfully installed!{Color.END}\n")
    
    print(f"{Color.CYAN}Commands:{Color.END}")
    print(f"  pironman5 --version           Show version")
    print(f"  pironman5 --config            Show configuration")
    print(f"  pironman5 --help              Show all options")
    print(f"  pironman5 start               Start service (manual)")
    print()
    
    print(f"{Color.CYAN}Service Management:{Color.END}")
    print(f"  sudo systemctl start pironman5    Start service")
    print(f"  sudo systemctl stop pironman5     Stop service")
    print(f"  sudo systemctl status pironman5   Check status")
    print(f"  sudo systemctl restart pironman5  Restart service")
    print(f"  sudo journalctl -u pironman5 -f   View logs")
    print()
    
    print(f"{Color.CYAN}Configuration:{Color.END}")
    print(f"  Config file: /opt/pironman5/config.json")
    print(f"  Log files:   /var/log/pironman5/")
    print()
    
    print(f"{Color.CYAN}Example Configuration:{Color.END}")
    print(f"  pironman5 --rgb-color ff00ff")
    print(f"  pironman5 --rgb-style rainbow")
    print(f"  pironman5 --oled-enable true")
    print(f"  pironman5 --temperature-unit F")
    print()
    
    print(f"{Color.YELLOW}Important:{Color.END}")
    print(f"  • Reboot required for hardware interfaces to work")
    print(f"  • Service will start automatically on boot")
    print(f"  • Run 'sudo reboot' to apply changes")
    print()
    
    print(f"{Color.CYAN}Next Steps:{Color.END}")
    print(f"  1. Reboot your system: sudo reboot")
    print(f"  2. Check service status: sudo systemctl status pironman5")
    print(f"  3. View logs: sudo journalctl -u pironman5 -f")
    print()
    
    print(f"{Color.BOLD}Documentation:{Color.END}")
    print(f"  README.md         - User guide")
    print(f"  QUICKSTART.md     - Quick start guide")
    print(f"  HARDWARE.md       - Hardware details")
    print(f"  FINAL_SUMMARY.md  - Complete overview")
    print()

def main():
    """Main installation function"""
    print(f"\n{Color.BOLD}{Color.CYAN}{'=' * 60}{Color.END}")
    print(f"{Color.BOLD}{Color.CYAN}Pironman5 Rust Installer{Color.END}")
    print(f"{Color.BOLD}{Color.CYAN}{'=' * 60}{Color.END}\n")
    
    # Pre-flight checks
    check_root()
    
    distro, pkg_manager = detect_os()
    print_info(f"Detected OS: {distro}")
    print_info(f"Package manager: {pkg_manager}")
    
    check_architecture()
    
    print(f"\n{Color.YELLOW}This will install:{Color.END}")
    print(f"  • System dependencies")
    print(f"  • Rust toolchain (if needed)")
    print(f"  • Pironman5 binary")
    print(f"  • Systemd service")
    print(f"  • Hardware configuration")
    print()
    
    response = input(f"{Color.BOLD}Continue with installation? (y/n): {Color.END}")
    if response.lower() != 'y':
        print_info("Installation cancelled")
        sys.exit(0)
    
    try:
        # Installation steps
        install_dependencies(pkg_manager)
        install_rust()
        enable_hardware_interfaces()
        binary_path = build_binary()
        install_binary(binary_path)
        setup_config_directory()
        install_systemd_service()
        setup_user_permissions()
        
        # Success!
        print_post_install_info()
        
    except KeyboardInterrupt:
        print(f"\n\n{Color.YELLOW}Installation interrupted by user{Color.END}")
        sys.exit(1)
    except Exception as e:
        print_error(f"Installation failed: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == '__main__':
    main()


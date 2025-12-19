#!/usr/bin/env python3
"""
Pironman5 Rust Uninstaller
"""

import os
import sys
import subprocess
import shutil

class Color:
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    CYAN = '\033[96m'
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

def run_command(cmd, check=True):
    """Run a command"""
    try:
        result = subprocess.run(cmd, check=check, capture_output=True, text=True)
        return result
    except subprocess.CalledProcessError as e:
        if check:
            print_error(f"Command failed: {' '.join(cmd)}")
            raise
        return e

def check_root():
    """Check if running as root"""
    if os.geteuid() != 0:
        print_error("This script must be run as root")
        print_info("Please run: sudo python3 uninstall.py")
        sys.exit(1)

def main():
    print(f"\n{Color.BOLD}{Color.RED}{'=' * 60}{Color.END}")
    print(f"{Color.BOLD}{Color.RED}Pironman5 Rust Uninstaller{Color.END}")
    print(f"{Color.BOLD}{Color.RED}{'=' * 60}{Color.END}\n")
    
    check_root()
    
    print(f"{Color.YELLOW}This will remove:{Color.END}")
    print("  • Pironman5 binary")
    print("  • Systemd service")
    print("  • Configuration files (optional)")
    print()
    
    response = input(f"{Color.BOLD}Continue with uninstallation? (y/n): {Color.END}")
    if response.lower() != 'y':
        print_info("Uninstallation cancelled")
        sys.exit(0)
    
    try:
        # Stop and disable service
        print_info("Stopping pironman5 service...")
        run_command(['systemctl', 'stop', 'pironman5.service'], check=False)
        
        print_info("Disabling pironman5 service...")
        run_command(['systemctl', 'disable', 'pironman5.service'], check=False)
        
        # Remove service file
        service_path = '/etc/systemd/system/pironman5.service'
        if os.path.exists(service_path):
            print_info("Removing systemd service...")
            os.remove(service_path)
            run_command(['systemctl', 'daemon-reload'])
        
        # Remove binary
        binary_path = '/usr/local/bin/pironman5'
        if os.path.exists(binary_path):
            print_info("Removing binary...")
            os.remove(binary_path)
        
        # Ask about configuration
        print()
        response = input(f"{Color.YELLOW}Remove configuration and logs? (y/n): {Color.END}")
        if response.lower() == 'y':
            if os.path.exists('/opt/pironman5'):
                print_info("Removing configuration...")
                shutil.rmtree('/opt/pironman5')
            
            if os.path.exists('/var/log/pironman5'):
                print_info("Removing logs...")
                shutil.rmtree('/var/log/pironman5')
        else:
            print_info("Keeping configuration and logs")
        
        print_success("\nPironman5 has been uninstalled!")
        print_info("Note: Rust toolchain was not removed")
        print_info("To remove Rust: rustup self uninstall")
        
    except Exception as e:
        print_error(f"Uninstallation failed: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()


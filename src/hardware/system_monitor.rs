/// System monitoring for CPU, memory, disk, and network
use anyhow::Result;
use std::fs;

use super::SystemStatus;

pub struct SystemMonitor {
    last_cpu_stats: Option<CpuStats>,
}

#[derive(Debug, Clone)]
struct CpuStats {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
}

impl SystemMonitor {
    pub fn new() -> Self {
        SystemMonitor {
            last_cpu_stats: None,
        }
    }

    /// Get current system status
    pub fn get_status(&mut self) -> Result<SystemStatus> {
        let cpu_usage = self.read_cpu_usage()?;
        let memory_usage = self.read_memory_usage()?;
        let cpu_temperature = self.read_temperature()?;
        let disk_usage = self.read_disk_usage()?;
        let (network_active, network_ip, network_interface) = self.get_network_info()?;

        Ok(SystemStatus {
            cpu_usage,
            memory_usage,
            cpu_temperature,
            gpu_temperature: None,
            disk_usage,
            network_active,
            network_ip,
            network_interface,
        })
    }

    fn read_cpu_usage(&mut self) -> Result<f32> {
        let stat_content = fs::read_to_string("/proc/stat")?;
        let cpu_line = stat_content
            .lines()
            .find(|line| line.starts_with("cpu "))
            .ok_or_else(|| anyhow::anyhow!("No CPU line in /proc/stat"))?;

        let values: Vec<u64> = cpu_line
            .split_whitespace()
            .skip(1)
            .filter_map(|s| s.parse().ok())
            .collect();

        if values.len() < 5 {
            return Ok(0.0);
        }

        let current_stats = CpuStats {
            user: values[0],
            nice: values[1],
            system: values[2],
            idle: values[3],
            iowait: values[4],
        };

        let usage = if let Some(last) = &self.last_cpu_stats {
            let total_delta = (current_stats.user + current_stats.nice + current_stats.system + current_stats.idle + current_stats.iowait)
                - (last.user + last.nice + last.system + last.idle + last.iowait);
            let idle_delta = current_stats.idle - last.idle;

            if total_delta > 0 {
                ((total_delta - idle_delta) as f32 / total_delta as f32) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        self.last_cpu_stats = Some(current_stats);
        Ok(usage.min(100.0))
    }

    fn read_memory_usage(&self) -> Result<f32> {
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        
        let mut mem_total = 0u64;
        let mut mem_available = 0u64;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                mem_total = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                mem_available = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }

        if mem_total > 0 {
            let used = mem_total - mem_available;
            Ok((used as f32 / mem_total as f32) * 100.0)
        } else {
            Ok(0.0)
        }
    }

    fn read_temperature(&self) -> Result<f32> {
        let temp_str = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")?;
        let temp_millidegrees: i32 = temp_str.trim().parse()?;
        Ok(temp_millidegrees as f32 / 1000.0)
    }

    fn read_disk_usage(&self) -> Result<f32> {
        // Read disk usage for root filesystem
        if let Ok(output) = std::process::Command::new("df")
            .args(&["-h", "/"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = output_str.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    if let Some(usage_str) = parts[4].strip_suffix('%') {
                        if let Ok(usage) = usage_str.parse::<f32>() {
                            return Ok(usage);
                        }
                    }
                }
            }
        }
        Ok(0.0)
    }

    fn check_network_activity(&self) -> Result<bool> {
        // Check if any network interface is up and has received packets
        let net_dev = fs::read_to_string("/proc/net/dev")?;
        
        for line in net_dev.lines().skip(2) {
            if let Some((interface, stats)) = line.split_once(':') {
                let interface = interface.trim();
                if interface.starts_with("eth") || interface.starts_with("wlan") {
                    let stats: Vec<&str> = stats.split_whitespace().collect();
                    if let Some(rx_bytes) = stats.first() {
                        if let Ok(bytes) = rx_bytes.parse::<u64>() {
                            if bytes > 0 {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }

    fn get_network_info(&self) -> Result<(bool, Option<String>, String)> {
        // Get network interface information and IP address
        let mut active = false;
        let mut ip_address: Option<String> = None;
        let mut interface_name = String::from("none");

        // Try to get IP address using 'ip addr' command
        if let Ok(output) = std::process::Command::new("ip")
            .args(&["addr", "show"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut current_interface = String::new();

            for line in output_str.lines() {
                // Parse interface name (e.g., "2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP>")
                if let Some(iface) = line.split(':').nth(1) {
                    let iface = iface.trim();
                    if iface.starts_with("eth") || iface.starts_with("wlan") || iface.starts_with("en") || iface.starts_with("wl") {
                        current_interface = iface.to_string();
                    }
                }

                // Parse IP address (e.g., "    inet 192.168.1.100/24")
                if line.contains("inet ") && !line.contains("inet6") && !current_interface.is_empty() {
                    if let Some(inet_part) = line.split_whitespace().nth(1) {
                        if let Some(ip) = inet_part.split('/').next() {
                            // Skip loopback
                            if !ip.starts_with("127.") {
                                ip_address = Some(ip.to_string());
                                interface_name = current_interface.clone();
                                active = true;
                                break; // Use first non-loopback interface
                            }
                        }
                    }
                }
            }
        }

        Ok((active, ip_address, interface_name))
    }
}


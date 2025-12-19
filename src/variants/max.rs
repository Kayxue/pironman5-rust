use super::Variant;
use serde_json::{json, Value};

pub struct Pironman5Max;

impl Variant for Pironman5Max {
    fn name(&self) -> &'static str {
        "Pironman 5 Max"
    }

    fn id(&self) -> &'static str {
        "pironman5"
    }

    fn product_version(&self) -> &'static str {
        ""
    }

    fn peripherals(&self) -> Vec<String> {
        vec![
            "storage",
            "cpu",
            "network",
            "memory",
            "history",
            "log",
            "ws2812",
            "cpu_temperature",
            "gpu_temperature",
            "temperature_unit",
            "oled",
            "clear_history",
            "delete_log_file",
            "pwm_fan_speed",
            "gpio_fan_state",
            "gpio_fan_mode",
            "gpio_fan_led",
            "vibration_switch",
            "oled_sleep",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    fn system_default_config(&self) -> Value {
        json!({
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
            "gpio_fan_led": "follow",
            "gpio_fan_led_pin": 5,
            "oled_sleep_timeout": 10,
            "vibration_switch_pin": 26,
            "vibration_switch_pull_up": false,
        })
    }

    fn dt_overlays(&self) -> Vec<String> {
        vec!["sunfounder-pironman5.dtbo".to_string()]
    }
}


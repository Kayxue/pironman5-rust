use super::Variant;
use serde_json::{json, Value};

pub struct Pironman5Mini;

impl Variant for Pironman5Mini {
    fn name(&self) -> &'static str {
        "Pironman 5 Mini"
    }

    fn id(&self) -> &'static str {
        "pironman5-mini"
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
            "clear_history",
            "delete_log_file",
            "pwm_fan_speed",
            "gpio_fan_state",
            "gpio_fan_mode",
            "gpio_fan_led",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    fn system_default_config(&self) -> Value {
        json!({
            "data_interval": 1,
            "temperature_unit": "C",
            "rgb_led_count": 4,
            "rgb_enable": true,
            "rgb_color": "#ff00ff",
            "rgb_brightness": 50,
            "rgb_style": "rainbow",
            "rgb_speed": 50,
            "gpio_fan_pin": 6,
            "gpio_fan_mode": 0,
            "gpio_fan_led": "follow",
            "gpio_fan_led_pin": 5,
        })
    }

    fn dt_overlays(&self) -> Vec<String> {
        vec!["sunfounder-pironman5mini.dtbo".to_string()]
    }
}


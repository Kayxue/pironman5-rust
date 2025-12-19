pub mod base;
pub mod mini;
pub mod max;

use serde_json::Value;
use std::fs;
use std::path::Path;

pub use base::Pironman5Base;
pub use mini::Pironman5Mini;
pub use max::Pironman5Max;

/// Trait defining a Pironman5 variant
pub trait Variant {
    fn name(&self) -> &'static str;
    #[allow(dead_code)]
    fn id(&self) -> &'static str;
    fn product_version(&self) -> &'static str;
    #[allow(dead_code)]
    fn peripherals(&self) -> Vec<String>;
    fn system_default_config(&self) -> Value;
    #[allow(dead_code)]
    fn dt_overlays(&self) -> Vec<String>;
}

/// Get device tree path
fn get_device_tree_path() -> Option<String> {
    let paths = ["/proc/device-tree", "/device-tree"];
    for path in &paths {
        if Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    None
}

/// Read device tree file as hex value
fn read_device_tree_file(file_path: &str) -> Option<u32> {
    if let Ok(content) = fs::read_to_string(file_path) {
        let trimmed = content.trim_end_matches('\0');
        if let Ok(value) = u32::from_str_radix(trimmed, 16) {
            return Some(value);
        }
    }
    None
}

/// Get part number from HAT device tree
fn get_part_number() -> Option<String> {
    let device_tree_path = get_device_tree_path()?;
    let entries = fs::read_dir(&device_tree_path).ok()?;
    
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if name.starts_with("hat") {
            let hat_path = format!("{}/{}", device_tree_path, name);
            let product_id_file = format!("{}/product_id", hat_path);
            let product_ver_file = format!("{}/product_ver", hat_path);
            
            if let (Some(product_id), Some(product_ver)) = 
                (read_device_tree_file(&product_id_file), read_device_tree_file(&product_ver_file)) {
                return Some(format!("{:04}V{:02}", product_id, product_ver));
            }
        }
    }
    None
}

/// Get variant ID and version
pub fn get_variant_id_and_version() -> (String, String) {
    // Check environment variable first
    if let Ok(part_number) = std::env::var("PIRONMAN5_PART_NUMBER") {
        let parts: Vec<&str> = part_number.split('V').collect();
        if parts.len() == 2 {
            return (parts[0].to_string(), parts[1].to_string());
        }
    }
    
    // Get from HAT info
    if let Some(part_number) = get_part_number() {
        let parts: Vec<&str> = part_number.split('V').collect();
        if parts.len() == 2 {
            return (parts[0].to_string(), parts[1].to_string());
        }
    }
    
    // Default
    ("0306".to_string(), "10".to_string())
}

/// Get variant based on ID and version
pub fn get_variant(variant_id: &str, version: &str) -> Box<dyn Variant> {
    match variant_id {
        "0306" => {
            if version == "10" {
                Box::new(Pironman5Base)
            } else {
                Box::new(Pironman5Max)
            }
        }
        "0308" => Box::new(Pironman5Mini),
        _ => Box::new(Pironman5Base), // Default fallback
    }
}

/// Get forced variant from file
pub fn get_force_variant() -> Option<Box<dyn Variant>> {
    let variant_file = "/opt/pironman5/variant";
    if let Ok(content) = fs::read_to_string(variant_file) {
        let short_name = content.trim();
        return match short_name {
            "base" => Some(Box::new(Pironman5Base)),
            "mini" => Some(Box::new(Pironman5Mini)),
            "max" => Some(Box::new(Pironman5Max)),
            _ => None,
        };
    }
    None
}

/// Get the active variant (forced or auto-detected)
pub fn get_active_variant() -> Box<dyn Variant> {
    if let Some(variant) = get_force_variant() {
        println!("Force variant: {}", variant.name());
        return variant;
    }
    
    let (variant_id, version) = get_variant_id_and_version();
    get_variant(&variant_id, &version)
}


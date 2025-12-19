# Python to Rust Migration Guide

This document maps the Python implementation to the Rust implementation of Pironman5.

## File Structure Mapping

| Python | Rust | Notes |
|--------|------|-------|
| `pironman5/version.py` | `src/version.rs` | Version constant |
| `pironman5/logger.py` | `src/logger.rs` | Logger implementation |
| `pironman5/utils.py` | `src/utils.rs` | Utility functions |
| `pironman5/pironman5.py` | `src/pironman5.rs` | Main application logic |
| `pironman5/__init__.py` | `src/cli.rs` | CLI interface |
| `pironman5/variants/__init__.py` | `src/variants/mod.rs` | Variant detection |
| `pironman5/variants/pironman5.py` | `src/variants/base.rs` | Base variant |
| `pironman5/variants/pironman5_mini.py` | `src/variants/mini.rs` | Mini variant |
| `pironman5/variants/pironman5_max.py` | `src/variants/max.rs` | Max variant |
| `bin/pironman5` (bash) | Binary output | Compiled Rust binary |

## Class/Struct Mapping

### Logger (Python) → Logger (Rust)

**Python:**
```python
class Logger(logging.Logger):
    def __init__(self, appname, name='logger', level=0, 
                 maxBytes=10*1024*1024, backupCount=10):
        # ...
```

**Rust:**
```rust
pub struct Logger {
    app_name: String,
    log_path: PathBuf,
    level: LevelFilter,
    file: Mutex<std::fs::File>,
}
```

### Pironman5 (Python) → Pironman5 (Rust)

**Python:**
```python
class Pironman5:
    def __init__(self, config_path=CONFIG_PATH):
        self.config = {...}
        # ...
```

**Rust:**
```rust
pub struct Pironman5 {
    config: Value,
    config_path: String,
    logger: Logger,
    variant: Box<dyn Variant>,
    running: Arc<AtomicBool>,
}
```

## Function Mapping

### Utils

| Python | Rust |
|--------|------|
| `merge_dict(dict1, dict2)` | `merge_dict(&dict1, &dict2)` |
| `is_included(li, target)` | `is_included(&list, target)` |
| `has_common_items(list1, list2)` | `has_common_items(&list1, &list2)` |

### Variant Detection

| Python | Rust |
|--------|------|
| `get_device_tree_path()` | `get_device_tree_path()` |
| `get_part_number()` | `get_part_number()` |
| `get_variant_id_and_version()` | `get_variant_id_and_version()` |
| `get_variant(id, version)` | `get_variant(&id, &version)` |
| `get_force_variant()` | `get_force_variant()` |

## Configuration Handling

### Python
```python
with open(config_path, 'r') as f:
    config = json.load(f)
```

### Rust
```rust
let mut file = File::open(&config_path)?;
let mut contents = String::new();
file.read_to_string(&mut contents)?;
let config: Value = serde_json::from_str(&contents)?;
```

## Error Handling

### Python
```python
def log_error(func):
    def wrapper(self, *args, **kwargs):
        try:
            return func(self, *args, **kwargs)
        except Exception as e:
            self.log.exception(str(e))
    return wrapper
```

### Rust
```rust
use anyhow::{Context, Result};

pub fn some_function() -> Result<()> {
    do_something().context("Failed to do something")?;
    Ok(())
}
```

## CLI Arguments

The Rust implementation uses the `clap` crate for argument parsing, which provides:
- Automatic help generation
- Type-safe argument parsing
- Better error messages
- Derives for struct-based CLI definition

### Python (argparse)
```python
parser = argparse.ArgumentParser()
parser.add_argument("-v", "--version", action="store_true")
args = parser.parse_args()
```

### Rust (clap)
```rust
#[derive(Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub version: bool,
}

let cli = Cli::parse();
```

## Concurrency

### Python
```python
import threading
while True:
    time.sleep(1)
```

### Rust
```rust
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

let running = Arc::new(AtomicBool::new(true));
while running.load(Ordering::SeqCst) {
    thread::sleep(Duration::from_secs(1));
}
```

## Signal Handling

### Python
```python
import signal
signal.signal(signal.SIGINT, self.signal_handler)
signal.signal(signal.SIGTERM, self.signal_handler)
```

### Rust
```rust
use ctrlc;

ctrlc::set_handler(move || {
    println!("Received interrupt signal");
    running.store(false, Ordering::SeqCst);
})?;
```

## Type Safety Differences

### Python (Dynamic)
```python
config = {
    'system': {
        'rgb_brightness': 50,  # Could be any type at runtime
    }
}
```

### Rust (Static)
```rust
// JSON Value is validated at parse time
let config: Value = json!({
    "system": {
        "rgb_brightness": 50u8,  // Type checked at compile time
    }
});
```

## Memory Management

### Python
- Garbage collected
- Reference counting
- Automatic memory management

### Rust
- Ownership system
- No garbage collection
- Zero-cost abstractions
- Memory safety at compile time

## Performance Comparison

| Aspect | Python | Rust |
|--------|--------|------|
| Startup Time | Slower (interpreter) | Fast (compiled) |
| Memory Usage | Higher (runtime overhead) | Lower (no runtime) |
| CPU Usage | Higher (interpreted) | Lower (native code) |
| Binary Size | N/A (requires Python) | ~5-10 MB standalone |

## Key Rust Advantages

1. **Compile-time guarantees**: Many bugs caught before runtime
2. **No runtime dependencies**: Single binary, no Python required
3. **Memory safety**: No null pointer dereferences, buffer overflows
4. **Concurrency safety**: Thread safety enforced by compiler
5. **Performance**: Native code execution
6. **Zero-cost abstractions**: High-level code with low-level performance

## Migration Checklist

- [x] Port version information
- [x] Port logger implementation
- [x] Port utility functions
- [x] Port variant detection logic
- [x] Port variant configurations
- [x] Port main application logic
- [x] Port CLI interface
- [x] Port configuration management
- [x] Add signal handling
- [ ] Integrate hardware control libraries (pm_auto)
- [ ] Integrate dashboard (pm_dashboard)
- [ ] Add systemd service file
- [ ] Add installation script

## Hardware Integration

The current Rust implementation provides the framework. To integrate actual hardware control:

### Option 1: Rewrite in Rust
Rewrite pm_auto and pm_dashboard libraries in pure Rust using crates like:
- `rppal` - Raspberry Pi GPIO
- `i2cdev` - I2C communication
- `spidev` - SPI communication

### Option 2: FFI (Foreign Function Interface)
Call existing Python/C libraries from Rust using:
- `pyo3` - Python bindings for Rust
- Direct C FFI for C libraries

### Example FFI with pyo3
```rust
use pyo3::prelude::*;
use pyo3::types::PyModule;

fn call_python_pm_auto() -> PyResult<()> {
    Python::with_gil(|py| {
        let pm_auto = PyModule::import(py, "pm_auto.pm_auto")?;
        let pm_auto_class = pm_auto.getattr("PMAuto")?;
        // Create instance and call methods
        Ok(())
    })
}
```

## Testing

### Python
```python
import unittest

class TestPironman5(unittest.TestCase):
    def test_merge_dict(self):
        result = merge_dict({'a': 1}, {'b': 2})
        self.assertEqual(result, {'a': 1, 'b': 2})
```

### Rust
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_dict() {
        let dict1 = json!({"a": 1});
        let dict2 = json!({"b": 2});
        let result = merge_dict(&dict1, &dict2);
        assert_eq!(result, json!({"a": 1, "b": 2}));
    }
}
```

## Building and Deployment

### Python
```bash
pip install -e .
```

### Rust
```bash
# Build
cargo build --release

# Install
sudo cp target/release/pironman5 /usr/local/bin/

# Or use cargo install
cargo install --path .
```

## Next Steps

1. Choose hardware integration strategy (pure Rust or FFI)
2. Implement or bind hardware control libraries
3. Test on actual Pironman5 hardware
4. Create systemd service file
5. Create Debian package (.deb) for easy installation
6. Performance testing and optimization
7. Documentation and examples


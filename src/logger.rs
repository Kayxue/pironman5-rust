use log::{LevelFilter, Metadata, Record};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// A custom logger that writes to both file and console
pub struct Logger {
    #[allow(dead_code)]
    app_name: String,
    #[allow(dead_code)]
    log_path: PathBuf,
    level: LevelFilter,
    file: Mutex<std::fs::File>,
}

impl Logger {
    pub fn new(app_name: &str, name: &str, level: LevelFilter) -> anyhow::Result<Self> {
        let log_folder = format!("/var/log/{}", app_name);
        let log_path = PathBuf::from(&log_folder).join(format!("{}.log", name.to_lowercase()));

        // Create log directory if it doesn't exist
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        Ok(Logger {
            app_name: app_name.to_string(),
            log_path,
            level,
            file: Mutex::new(file),
        })
    }

    pub fn set_level(&mut self, level: LevelFilter) {
        self.level = level;
    }

    #[allow(dead_code)]
    fn format_record(&self, record: &Record) -> String {
        let now = chrono::Local::now();
        format!(
            "{} [{}] {}",
            now.format("%y/%m/%d %H:%M:%S%.3f"),
            record.level(),
            record.args()
        )
    }

    #[allow(dead_code)]
    pub fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let formatted = self.format_record(record);
            
            // Write to console
            println!("{}", formatted);
            
            // Write to file
            if let Ok(mut file) = self.file.lock() {
                let _ = writeln!(file, "{}", formatted);
            }
        }
    }

    #[allow(dead_code)]
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }
}

/// Create a logger factory function
#[allow(dead_code)]
pub fn create_get_child_logger(app_name: &'static str) -> impl Fn(&str) -> anyhow::Result<Logger> {
    move |name: &str| Logger::new(app_name, name, LevelFilter::Info)
}


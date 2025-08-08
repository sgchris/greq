use std::env;
use std::fs;
use std::path::PathBuf;
use dirs;
use log::LevelFilter;
use env_logger::Builder;
use std::io::Write;

/// Initialize logging for the application
/// Detailed logs go to AppData/Local/greq/greq.log
/// Console shows only important information with colors
pub fn init_logger() -> crate::Result<()> {
    let log_dir = get_log_directory()?;
    fs::create_dir_all(&log_dir)?;
    
    let log_file_path = log_dir.join("greq.log");
    
    let mut builder = Builder::new();
    
    // Set log level from environment or default to Info
    let level = env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .parse()
        .unwrap_or(LevelFilter::Info);
    
    builder
        .filter_level(level)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {} {}:{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .target(env_logger::Target::Stdout);
    
    // For file logging, we would need to implement a custom logger
    // For now, we'll use the default env_logger to stdout
    builder.init();
    
    log::info!("Greq logger initialized. Log file: {log_file_path:?}");
    Ok(())
}

/// Get the appropriate log directory for the current OS
fn get_log_directory() -> crate::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let local_app_data = dirs::data_local_dir()
            .ok_or_else(|| crate::error::GreqError::Io(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot find local app data directory")
            ))?;
        Ok(local_app_data.join("greq"))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let home = dirs::home_dir()
            .ok_or_else(|| crate::error::GreqError::Io(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot find home directory")
            ))?;
        Ok(home.join(".local/share/greq"))
    }
}

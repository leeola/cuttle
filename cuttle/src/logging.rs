use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging(log_file: Option<&str>) {
    if let Some(file_path) = log_file {
        // File logging for production/Blender
        let file = std::fs::File::create(file_path)
            .unwrap_or_else(|e| panic!("Failed to create log file {file_path}: {e}"));

        tracing_subscriber::registry()
            .with(fmt::layer().with_writer(file))
            .init();
    } else {
        // Console logging for development
        fmt::init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::{debug, info};

    #[test]
    fn test_console_logging() {
        init_logging(None);
        info!("Test console logging");
        debug!("Debug message");
    }

    #[test]
    #[ignore] // Skip in regular test runs since tracing can only be initialized once
    fn test_file_logging() {
        let temp_file = "/tmp/cuttle_test.log";
        init_logging(Some(temp_file));
        info!("Test file logging");

        // Verify file was created
        assert!(std::path::Path::new(temp_file).exists());

        // Clean up
        let _ = std::fs::remove_file(temp_file);
    }
}

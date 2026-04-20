use std::fs::OpenOptions;
use std::path::Path;

pub fn rotate_log_file() {
    let log_path = "logs/pipeline.log";
    
    if !Path::new(log_path).exists() {
        return;
    }

    // Read existing log content
    let existing_content = match std::fs::read_to_string(log_path) {
        Ok(content) => content,
        Err(_) => return,
    };

    if existing_content.is_empty() {
        return;
    }

    // Create separator
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let separator = format!(
        "\n\n{}\n{}\n{}\n\n",
        "═".repeat(80),
        format!("  PREVIOUS SESSION (archived at {})", timestamp),
        "═".repeat(80)
    );
    
    // Save to archive with separator
    let archive_path = format!("logs/pipeline_archive_{}.log", 
        chrono::Local::now().format("%Y%m%d_%H%M%S"));
    let archive_content = format!("{}{}", separator, existing_content);
    std::fs::write(&archive_path, &archive_content).ok();
}

pub fn setup_tracing() {
    use std::sync::OnceLock;
    static INIT: OnceLock<()> = OnceLock::new();
    
    INIT.get_or_init(|| {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        use tracing_subscriber::{fmt, EnvFilter};
        use std::fs::OpenOptions;

        // Ensure logs directory exists before opening file
        std::fs::create_dir_all("logs").ok();

        let file_layer = fmt::layer()
            .with_writer(move || {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("logs/pipeline.log")
                    .expect("Failed to open log file")
            })
            .with_ansi(false);

        let stdout_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .with_ansi(true);

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));

        // try_init() returns Err if already set — we silently ignore that
        let _ = tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(stdout_layer)
            .try_init();
    });
}

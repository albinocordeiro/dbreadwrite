use color_eyre::eyre::Result;
use log::info;
use pretty_env_logger;
use std::env;
use std::{thread, time};
use clap::Clap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dbreadwrite::schema_management::load_event_types_from_file;
use dbreadwrite::random_events::execute_random_read_query;

#[derive(Clap)]
#[clap(
    version = "0.1.0",
    author = "Albino Cordeiro <albino@intuitionlogic.com",
    about = "Concurrent database reader experiment"
)]
struct Options {
    #[clap(short, long, default_value = "0.1")]
    seconds_between_reads: f64,
    #[clap(short, long, default_value = "./typemapping/type_mapping.json")]
    event_type_file: String,
}


fn main() -> Result<()> {
    let options = Options::parse();
    // If no log level env var defined, define log level == trace by default
    match env::var("RUST_LOG") {
        Ok(_) => pretty_env_logger::init(),
        _ => {
            env::set_var("AUTOMATIC_LOG_LEVEL", "trace");
            pretty_env_logger::init_custom_env("AUTOMATIC_LOG_LEVEL");
        }
    }
    info!("Starting writer. Use Ctrl+C to stop it at any point");

    // Capture user keyboard interrupt commands
    let terminate_command = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&terminate_command))?;

    let event_types = load_event_types_from_file(&options.event_type_file)?;

    while !terminate_command.load(Ordering::Relaxed) {
    
        execute_random_read_query(&event_types)?;

        thread::sleep(time::Duration::from_secs_f64(
            options.seconds_between_reads,
        ));
    }

    info!("Terminated by the user...");

    Ok(())
}

use color_eyre::eyre::Result;
use log::info;
use pretty_env_logger;
use std::env;
use std::{thread, time};
// use serde_json::{Map, Value};
use clap::Clap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dbreadwrite::schema_management::SchemaManager;
use dbreadwrite::random_events::commit_random_event;

#[derive(Clap)]
#[clap(
    version = "0.1.0",
    author = "Albino Cordeiro <albino@intuitionlogic.com",
    about = "Concurrent database writer experiment"
)]
struct Options {
    #[clap(short, long, default_value = "5")]
    seconds_between_writes: f64,
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

    // SchemaManager constructor will try to update schema and update local copy of the event_type map
    let mut schema_manager = SchemaManager::new(&options.event_type_file)?;

    while !terminate_command.load(Ordering::Relaxed) {
        schema_manager.check_update_schema()?;
        commit_random_event(schema_manager.get_event_types())?;

        thread::sleep(time::Duration::from_secs_f64(
            options.seconds_between_writes,
        ));
    }

    info!("Terminated by the user...");

    Ok(())
}

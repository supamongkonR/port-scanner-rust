use env_logger::{Builder, Target};
use log::{error, info};
use std::fs::OpenOptions;
use std::io::Write;

pub fn init_logger() {
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("scan.log")
        .unwrap();

    Builder::new()
        .target(Target::Pipe(Box::new(log_file))) // Write logs to "scan.log"
        .filter(None, log::LevelFilter::Info) // Set log level
        .init();
}

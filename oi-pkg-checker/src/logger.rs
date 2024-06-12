use std::process::exit;

use colored::Colorize;
use log::{Level, Log, Metadata, Record};

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let (level, message) = match record.level() {
                Level::Error => {
                    let msg = record.args().to_string();
                    if msg.starts_with("fatal ") {
                        println!(
                            "{} {}",
                            "[FATAL]".to_string().red().bold(),
                            msg.trim_start_matches("fatal ").red().bold()
                        );
                        exit(1);
                    } else {
                        ("[ERROR]".to_string().red(), msg.to_string().red())
                    }
                }
                Level::Warn => (
                    "[WARN]".to_string().bright_yellow(),
                    format!("{}", record.args()).bright_yellow(),
                ),
                Level::Info => (
                    "[INFO]".to_string().bright_green(),
                    format!("{}", record.args()).bright_green(),
                ),
                Level::Debug => (
                    "[DEBUG]".to_string().blue(),
                    format!("{}", record.args()).blue(),
                ),
                Level::Trace => unimplemented!(),
            };

            println!("{} {}", level, message);
        }
    }
    fn flush(&self) {
        unimplemented!()
    }
}

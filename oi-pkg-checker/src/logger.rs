use colored::{Colorize, CustomColor};
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
                    (format!("[ERROR]").red(), format!("{}", record.args()).red())
                }
                Level::Warn => {
                    (format!("[WARN]").custom_color(CustomColor::new(255,140,0)), format!("{}", record.args()).custom_color(CustomColor::new(255,140,0)))
                }
                Level::Info => {
                    (format!("[INFO]").bright_green(), format!("{}", record.args()).bright_green())
                }
                Level::Debug => {
                    (format!("[DEBUG]").blue(), format!("{}", record.args()).blue())
                }
                Level::Trace => unimplemented!()
            };

            println!("{} {}", level, message);
        }
    }
    fn flush(&self) { unimplemented!() }
}
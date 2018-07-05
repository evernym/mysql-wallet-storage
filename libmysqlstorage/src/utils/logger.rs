extern crate env_logger;
extern crate log;

use self::env_logger::LogBuilder;
use self::log::{LogRecord, LogLevelFilter};
use std::env;
use std::sync::{Once, ONCE_INIT};

static LOGGER_INIT: Once = ONCE_INIT;

pub fn init() {
    LOGGER_INIT.call_once(|| {
        let format = |record: &LogRecord| {
            format!("{:>5}|{:<30}|{:>35}:{:<4}| {}", record.level(), record.target(), record.location().file(), record.location().line(), record.args())
        };
        let mut builder = LogBuilder::new();
        builder.format(format).filter(None, LogLevelFilter::Off);

        if env::var("RUST_LOG").is_ok() {
            builder.parse(&env::var("RUST_LOG").unwrap());
        }

        builder.init().unwrap();
    });
}

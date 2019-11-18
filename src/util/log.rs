use std::env;
use std::io::Write;

use env_logger;

pub fn set_logger() {
    match env::var("RUST_LOG") {
        Ok(val) => {
            let log_level: &str = &val;
            match log_level {
                "error" => { /* noop */ }
                "warn" => { /* noop */ }
                "info" => { /* noop */ }
                "debug" => { /* noop */ }
                _ => env::set_var("RUST_LOG", "info"),
            }
        }
        Err(_e) => env::set_var("RUST_LOG", "info"),
    };
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let ts = buf.timestamp();
            writeln!(
                buf,
                "[{} {} {} {}:{}] {}",
                ts,
                record.level(),
                record.target(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args(),
            )
        })
        .init();
}

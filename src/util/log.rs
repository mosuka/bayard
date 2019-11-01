use std::env;

pub fn set_log_level() {
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
    }
}

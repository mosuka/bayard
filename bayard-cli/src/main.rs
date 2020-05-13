use bayard_cli::cli::command::root;
use bayard_common::log::set_logger;

fn main() -> Result<(), std::io::Error> {
    set_logger();

    root()
}

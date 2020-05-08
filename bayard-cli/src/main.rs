use bayard_cli::cli::root::run_root_cli;
use bayard_common::log::set_logger;

fn main() -> Result<(), std::io::Error> {
    set_logger();

    run_root_cli()
}

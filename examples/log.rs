extern crate sdl3;

use sdl3::log::*;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _sdl_context = sdl3::init()?;

    set_log_priority_prefix(Priority::Verbose, "VERBOSE: ");
    set_log_priority_prefix(Priority::Debug, "DEBUG: ");
    set_log_priority_prefix(Priority::Info, "INFO: ");
    set_log_priority_prefix(Priority::Critical, "CRITICAL: ");

    set_log_priority(Category::Application, Priority::Verbose);

    log_verbose(Category::Application, "This is verbose");
    log_debug(Category::Application, "This is verbose");
    log_info(Category::Application, "This is info");
    log_warn(Category::Application, "This is warn");
    log_error(Category::Application, "This is error");
    log_critical(Category::Application, "This is critical");

    Ok(())
}

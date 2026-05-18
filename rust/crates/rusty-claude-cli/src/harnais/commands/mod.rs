pub mod arch;
pub mod cb;
pub mod context;
pub mod ka;
pub mod top_level;

use super::cli::Command;

pub fn handle(command: Command) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Command::Init(args) => top_level::handle_init(args),
        Command::Upgrade => top_level::handle_upgrade(),
        Command::Status => top_level::handle_status(),
        Command::Test(args) => top_level::handle_test(args),
        Command::Why(args) => top_level::handle_why(args),
        Command::Skip(args) => top_level::handle_skip(args),
        Command::Reflect(args) => top_level::handle_reflect(args),
        Command::InstallHooks(args) => top_level::handle_install_hooks(args),
        Command::Cb(args) => cb::handle(args),
        Command::Ka(args) => ka::handle(args),
        Command::Arch(args) => arch::handle(args),
        Command::Context(args) => context::handle(args),
    }
}

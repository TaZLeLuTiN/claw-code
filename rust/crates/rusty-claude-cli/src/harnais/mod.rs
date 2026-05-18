pub mod cli;
pub mod commands;

use clap::Parser;
use cli::Cli;

const HARNAIS_COMMANDS: &[&str] = &[
    "init",
    "upgrade",
    "status",
    "test",
    "why",
    "skip",
    "reflect",
    "install-hooks",
    "cb",
    "ka",
    "arch",
    "context",
];

pub fn is_harnais_command(arg: &str) -> bool {
    HARNAIS_COMMANDS.contains(&arg)
}

pub fn dispatch(raw_args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse_from(raw_args);
    commands::handle(cli.command)
}

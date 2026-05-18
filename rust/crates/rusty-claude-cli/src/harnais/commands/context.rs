use crate::harnais::cli::{ContextArgs, ContextCommand};

#[allow(clippy::needless_pass_by_value)]
pub fn handle(args: ContextArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        ContextCommand::Init(_) => {
            Err(Box::from("context init: not yet implemented (PURE_RUST — Étape 3.2)"))
        }
        ContextCommand::Start(_) => {
            Err(Box::from("context start: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
    }
}

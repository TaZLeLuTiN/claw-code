use crate::harnais::cli::{KaArgs, KaCommand};

#[allow(clippy::needless_pass_by_value)]
pub fn handle(args: KaArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        KaCommand::Retrospective(_) => {
            Err(Box::from("ka retrospective: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
        KaCommand::Validate(_) => {
            Err(Box::from("ka validate: not yet implemented (PURE_RUST — Étape 3.2)"))
        }
        KaCommand::Status => {
            Err(Box::from("ka status: not yet implemented (PURE_RUST — Étape 3.2)"))
        }
        KaCommand::Search(_) => {
            Err(Box::from("ka search: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
        KaCommand::Deduplicate => {
            Err(Box::from("ka deduplicate: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
        KaCommand::ExportToSymphony(_) => {
            Err(Box::from("ka export-to-symphony: not yet implemented (PURE_RUST — Étape 3.2)"))
        }
        KaCommand::PhaseReport(_) => {
            Err(Box::from("ka phase-report: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
    }
}

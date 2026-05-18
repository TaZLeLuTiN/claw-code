use crate::harnais::cli::{CbArgs, CbCommand};

#[allow(clippy::needless_pass_by_value)]
pub fn handle(args: CbArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        CbCommand::Start => Err(Box::from("cb start: not yet implemented (PURE_RUST — Étape 3.2)")),
        CbCommand::Stop => Err(Box::from("cb stop: not yet implemented (PURE_RUST — Étape 3.2)")),
        CbCommand::Status => Err(Box::from("cb status: not yet implemented (PURE_RUST — Étape 3.2)")),
        CbCommand::Ingest(_) => Err(Box::from("cb ingest: not yet implemented (VIA_FFI — Étape 3.3)")),
        CbCommand::Query(_) => Err(Box::from("cb query: not yet implemented (VIA_FFI — Étape 3.3)")),
        CbCommand::Handoff(_) => Err(Box::from("cb handoff: not yet implemented (VIA_FFI — Étape 3.3)")),
        CbCommand::Purge(_) => Err(Box::from("cb purge: not yet implemented (PURE_RUST — Étape 3.2)")),
    }
}

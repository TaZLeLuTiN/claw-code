use crate::harnais::cli::{ArchArgs, ArchCommand};

#[allow(clippy::needless_pass_by_value)]
pub fn handle(args: ArchArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        ArchCommand::Check => {
            Err(Box::from("arch check: not yet implemented (PURE_RUST — Étape 3.2)"))
        }
        ArchCommand::Status => {
            Err(Box::from("arch status: not yet implemented (PURE_RUST — Étape 3.2)"))
        }
        ArchCommand::Ingest(_) => {
            Err(Box::from("arch ingest: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
        ArchCommand::IngestAll(_) => {
            Err(Box::from("arch ingest-all: not yet implemented (VIA_FFI — Étape 3.3)"))
        }
    }
}

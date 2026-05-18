use crate::harnais::cli::{
    InitArgs, InstallHooksArgs, ReflectArgs, SkipArgs, TestArgs, WhyArgs,
};

pub fn handle_init(_args: InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    Err(Box::from("harnais init: not yet implemented (PURE_RUST — Étape 3.2)"))
}

pub fn handle_upgrade() -> Result<(), Box<dyn std::error::Error>> {
    Err(Box::from("harnais upgrade: not yet implemented (PURE_RUST — Étape 3.2)"))
}

pub fn handle_status() -> Result<(), Box<dyn std::error::Error>> {
    Err(Box::from("harnais status: not yet implemented (PURE_RUST — Étape 3.2)"))
}

pub fn handle_test(_args: TestArgs) -> Result<(), Box<dyn std::error::Error>> {
    Err(Box::from("harnais test: not yet implemented (PURE_RUST — Étape 3.2)"))
}

pub fn handle_why(_args: WhyArgs) -> Result<(), Box<dyn std::error::Error>> {
    Err(Box::from("harnais why: not yet implemented (PURE_RUST — Étape 3.2)"))
}

pub fn handle_skip(_args: SkipArgs) -> Result<(), Box<dyn std::error::Error>> {
    Err(Box::from("harnais skip: not yet implemented (PURE_RUST — Étape 3.2)"))
}

pub fn handle_reflect(_args: ReflectArgs) -> Result<(), Box<dyn std::error::Error>> {
    Err(Box::from("harnais reflect: not yet implemented (VIA_FFI — Étape 3.3)"))
}

pub fn handle_install_hooks(_args: InstallHooksArgs) -> Result<(), Box<dyn std::error::Error>> {
    Err(Box::from("harnais install-hooks: not yet implemented (PURE_RUST — Étape 3.2)"))
}

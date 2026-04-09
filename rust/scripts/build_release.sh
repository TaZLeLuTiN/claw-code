#!/bin/bash

# Build optimisé
cargo build --release --workspace

# Création packages
cargo deb --package rusty-claude-cli
cargo rpm --package rusty-claude-cli

# Docker image
docker build -t claude-rust .

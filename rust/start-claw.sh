#!/bin/bash

cd /Users/mburini/Documents/GitHub/claude/claw-code/rust

echo "📊 Démarrage de CLAW Framework..."
echo "🔧 Backend: http://localhost:4000"
echo "🌐 Frontend: http://localhost:5175"

# Démarrer le backend
CLAW_PORT=4000 cargo run --bin gui


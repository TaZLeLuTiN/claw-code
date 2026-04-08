#!/bin/bash

PROMPT="$1"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
LOG_FILE=".claw_history.log"

# Créer le dossier de logs
mkdir -p .claw

# Logger la requête
echo "[$TIMESTAMP] Prompt: $PROMPT" >> "$LOG_FILE"

# Détecter automatiquement le modèle optimal
MODEL_OUTPUT=$(./smart_claw.sh "$PROMPT" 2>&1)
MODEL=$(echo "$MODEL_OUTPUT" | grep "Using" | cut -d' ' -f4- | tr -d '\n')

if [ -z "$MODEL" ]; then
    MODEL="codellama:7b"  # fallback par défaut
fi

echo "🦀 Claw Code Pro - $TIMESTAMP"
echo "🤖 Model: $MODEL"
echo "📝 Prompt: $PROMPT"
echo "=========================================="

# Exécuter et capturer la réponse
RESPONSE=$(./claw_ollama.sh "$MODEL" "$PROMPT" 2>/dev/null)
echo "$RESPONSE"
echo ""

# Logger la réponse
echo "Response: " >> "$LOG_FILE"
echo "$RESPONSE" >> "$LOG_FILE"
echo "---" >> "$LOG_FILE"

echo "💾 Logged to: $LOG_FILE"

#!/bin/bash

PROMPT="$1"

if [ -z "$PROMPT" ]; then
    echo "Usage: $0 <prompt>"
    exit 1
fi

# Détecter le type de tâche
if echo "$PROMPT" | grep -qi -E "(optimize|performance|speed|fast)"; then
    MODEL="deepseek-coder:6.7b"
    echo "🚀 Task detected: Optimization - Using $MODEL"
elif echo "$PROMPT" | grep -qi -E "(explain|teach|learn|what is|how does)"; then
    MODEL="deepseek-coder-v2:latest"
    echo "🧠 Task detected: Explanation - Using $MODEL"
elif echo "$PROMPT" | grep -qi -E "(fix|debug|error|problem|issue)"; then
    MODEL="codellama:7b"
    echo "🔧 Task detected: Debugging - Using $MODEL"
elif echo "$PROMPT" | grep -qi -E "(compare|difference|versus)"; then
    MODEL="mistral:7b"
    echo "📊 Task detected: Comparison - Using $MODEL"
else
    MODEL="codellama:7b"
    echo "📝 Default task: General coding - Using $MODEL"
fi

echo "Prompt: $PROMPT"
echo "=========================================="

./claw_ollama.sh "$MODEL" "$PROMPT"

#!/bin/bash

MODEL="$1"
PROMPT="$2"

if [ -z "$MODEL" ] || [ -z "$PROMPT" ]; then
    echo "Usage: $0 <model> <prompt>"
    echo "Available models:"
    echo "  - codellama:7b"
    echo "  - deepseek-coder-v2:latest"
    echo "  - deepseek-coder:6.7b"
    echo "  - starcoder2:3b"
    echo "  - mistral:7b"
    exit 1
fi

echo "Claw Code with Ollama - Model: $MODEL"
echo "=========================================="
echo "Prompt: $PROMPT"
echo ""

ollama run "$MODEL" "$PROMPT"

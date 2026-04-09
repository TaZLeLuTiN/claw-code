#!/bin/bash
echo "🧪 CLAW Framework - Test complet"

# Démarrer le serveur
echo "🚀 Démarrage du serveur..."
cd /Users/mburini/Documents/GitHub/claude/claw-code/rust
CLAW_PORT=4001 cargo run --bin gui > server.log 2>&1 &
SERVER_PID=$!
sleep 5

# Fonction de test
run_test() {
    echo -e "\n🔍 Test: $1"
    local response=$(curl -s -w "%{http_code}" "$2" ${@:3})
    local status=${response: -3}
    local body=${response%???}
    
    if [ "$status" -eq 200 ]; then
        echo "✅ SUCCÈS ($status)"
        echo "$body" | jq . 2>/dev/null || echo "$body"
    else
        echo "❌ ÉCHEC ($status)"
        echo "$body"
    fi
}

# Exécuter les tests
run_test "Santé API" "http://localhost:4001/api/health"
run_test "Modèles IA" "http://localhost:4001/api/ai-models"
run_test "Projets" "http://localhost:4001/api/projects"
run_test "Modèles Ollama" "http://localhost:4001/api/ollama/models"
run_test "Recommandations" "http://localhost:4001/api/recommendations"
run_test "Rôle Architect" "http://localhost:4001/api/models/role/architect"

# Test création
echo -e "\n🎯 Test création projet..."
create_response=$(curl -s -X POST http://localhost:4001/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name":"test-integration", "path":"./test-integration", "language":"rust"}')
echo "$create_response" | jq .

# Arrêt
echo -e "\n🛑 Arrêt du serveur..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null

echo -e "\n📊 Résumé des tests:"
grep -E "✅|❌" server.log || echo "Consultez server.log pour les détails"

echo -e "\n🎉 Test complet terminé!"

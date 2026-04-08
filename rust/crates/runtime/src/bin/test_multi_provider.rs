use runtime::providers::{create_provider, ProviderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Test du système multi-provider Claw Code !");
    
    // Test Ollama (local)
    println!("\n🦙 Test Ollama provider...");
    let config = ProviderConfig {
        provider_type: "ollama".to_string(),
        api_key: None,
        base_url: Some("http://localhost:11434".to_string()),
        model: "codellama".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.2),
    };
    
    match create_provider(config) {
        Ok(provider) => {
            println!("✅ Provider '{}' créé avec succès !", provider.provider_name());
            println!("   Modèle: {}", provider.model_name());
            println!("   Supporte tools: {}", provider.supports_tools());
            println!("   Max tokens: {}", provider.max_tokens());
            
            // Health check
            match provider.health_check().await {
                Ok(()) => println!("   🏥 Health check: OK"),
                Err(e) => println!("   ⚠️ Health check: {}", e),
            }
        }
        Err(e) => println!("❌ Erreur Ollama: {}", e),
    }
    
    // Test DeepSeek (si clé)
    if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
        println!("\n🤖 Test DeepSeek provider...");
        let config = ProviderConfig {
            provider_type: "deepseek".to_string(),
            api_key: Some(api_key),
            base_url: None,
            model: "deepseek-coder-v2".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.1),
        };
        
        match create_provider(config) {
            Ok(provider) => {
                println!("✅ Provider '{}' créé avec succès !", provider.provider_name());
                println!("   Modèle: {}", provider.model_name());
                println!("   Supporte tools: {}", provider.supports_tools());
            }
            Err(e) => println!("❌ Erreur DeepSeek: {}", e),
        }
    } else {
        println!("\n🤖 DeepSeek: pas de clé API (DEEPSEEK_API_KEY non définie)");
    }
    
    // Test Anthropic (si clé)
    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        println!("\n🧠 Test Anthropic provider...");
        let config = ProviderConfig {
            provider_type: "anthropic".to_string(),
            api_key: Some(api_key),
            base_url: None,
            model: "claude-3-haiku-20240307".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
        };
        
        match create_provider(config) {
            Ok(provider) => {
                println!("✅ Provider '{}' créé avec succès !", provider.provider_name());
                println!("   Modèle: {}", provider.model_name());
                println!("   Supporte tools: {}", provider.supports_tools());
            }
            Err(e) => println!("❌ Erreur Anthropic: {}", e),
        }
    } else {
        println!("\n🧠 Anthropic: pas de clé API (ANTHROPIC_API_KEY non définie)");
    }
    
    println!("\n🎉 Test terminé !");
    Ok(())
}

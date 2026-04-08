use runtime::bridge::MultiProviderClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌉 Test complet du bridge multi-provider...");
    
    let mut client = MultiProviderClient::new("codellama:7b")?;
    println!("✅ Client créé avec succès !");
    
    println!("📝 Test avec un prompt simple...");
    let messages = vec![
        runtime::providers::ChatMessage {
            role: "user".to_string(),
            content: "Write a simple 'hello world' function in Rust".to_string(),
            tool_calls: None,
        }
    ];
    
    match client.chat_completion(messages).await {
        Ok(response) => {
            println!("✅ Réponse reçue :");
            println!("{}", response);
        }
        Err(e) => {
            println!("❌ Erreur : {}", e);
        }
    }
    
    println!("🎉 Test terminé !");
    Ok(())
}

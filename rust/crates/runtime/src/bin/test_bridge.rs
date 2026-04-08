use runtime::bridge::MultiProviderClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌉 Test du bridge multi-provider...");
    
    let _client = MultiProviderClient::new("codellama:7b")?;
    println!("✅ Client créé avec succès !");
    
    Ok(())
}

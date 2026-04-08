//! Bridge pour intégrer les providers dans le CLI existant

use crate::providers::{create_provider, ProviderConfig, ChatRequest, ChatMessage};

/// Interface compatible avec l'ancien système mais utilisant nos providers
pub struct MultiProviderClient {
    provider: Box<dyn crate::providers::AIProvider>,
}

impl MultiProviderClient {
    pub fn new(model: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (provider_type, model_name) = parse_model_name(model);
        
        let config = ProviderConfig {
            provider_type: provider_type.clone(), // Cloner pour éviter le move
            api_key: get_api_key(&provider_type),
            base_url: get_base_url(&provider_type),
            model: model_name,
            max_tokens: Some(4096),
            temperature: Some(0.2),
        };
        
        let provider = create_provider(config)?;
        Ok(Self { provider })
    }
    
    pub async fn chat_completion(&mut self, messages: Vec<ChatMessage>) -> Result<String, Box<dyn std::error::Error>> {
        let request = ChatRequest {
            messages,
            model: None,
            max_tokens: None,
            temperature: None,
            tools: None,
        };
        
        let response = self.provider.chat_completion(request).await?;
        Ok(response.content)
    }
}

fn parse_model_name(model: &str) -> (String, String) {
    if model.contains("codellama") || model.contains("starcoder") || model.contains("deepseek") || model.contains("mistral") {
        ("ollama".to_string(), model.to_string())
    } else if model.contains("deepseek") {
        ("deepseek".to_string(), model.to_string())
    } else if model.contains("claude") {
        ("anthropic".to_string(), model.to_string())
    } else {
        ("ollama".to_string(), model.to_string()) // défaut
    }
}

fn get_api_key(provider_type: &str) -> Option<String> {
    match provider_type {
        "anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
        "deepseek" => std::env::var("DEEPSEEK_API_KEY").ok(),
        "ollama" => None,
        _ => None,
    }
}

fn get_base_url(provider_type: &str) -> Option<String> {
    match provider_type {
        "ollama" => Some("http://localhost:11434".to_string()),
        _ => None,
    }
}

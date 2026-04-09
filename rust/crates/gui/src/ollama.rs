use serde::{Deserialize, Serialize};
use anyhow::Result;
use reqwest::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
    pub done: bool,
}

pub struct OllamaService {
    client: Client,
    base_url: String,
}

impl OllamaService {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
        };

        println!("🤖 Request to Ollama: {} - {}", model, prompt);
        
        // D'abord récupérez le texte brut pour debug
        let response_text = self.client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?
            .text()
            .await?;
        
        println!("📥 Raw response: {}", response_text);
        
        // Essayez de parser le JSON
        let ollama_response: OllamaResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse Ollama response: {} - Raw: {}", e, response_text))?;
        
        Ok(ollama_response.response)
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        #[derive(Deserialize)]
        struct ModelResponse {
            models: Vec<ModelInfo>,
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }

        let response = self.client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await?
            .json::<ModelResponse>()
            .await?;

        Ok(response.models.into_iter().map(|m| m.name).collect())
    }
}

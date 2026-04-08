//! Ollama local provider implementation

use super::{AIProvider, ChatRequest, ChatResponse, Usage, ProviderError};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

pub struct OllamaProvider {
    client: Client,
    model: String,
    max_tokens: usize,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(config: super::ProviderConfig) -> Result<Self, ProviderError> {
        Ok(Self {
            client: Client::new(),
            model: config.model,
            max_tokens: config.max_tokens.unwrap_or(4096),
            base_url: config.base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
        })
    }
    
    fn convert_request(&self, request: ChatRequest) -> Value {
        let messages: Vec<Value> = request.messages.into_iter().map(|msg| {
            let mut msg_obj = json!({
                "role": msg.role,
                "content": msg.content
            });
            
            // Ollama has limited tool support - add images if present
            if let Some(tool_calls) = msg.tool_calls {
                // For now, we'll convert tool calls to text
                let tool_text = tool_calls.iter()
                    .map(|tc| format!("Tool call: {} with args {}", tc.name, tc.arguments))
                    .collect::<Vec<_>>()
                    .join("\n");
                    
                msg_obj["content"] = json!(format!("{}\n\n{}", msg.content, tool_text));
            }
            
            msg_obj
        }).collect();
        
        let mut request_body = json!({
            "model": self.model,
            "messages": messages,
            "stream": false,
        });
        
        if let Some(temperature) = request.temperature {
            request_body["options"] = json!({
                "temperature": temperature
            });
        }
        
        // Handle tools (limited support in Ollama)
        if let Some(_tools) = request.tools {
            // Ollama doesn't have native tool support yet
            // We could implement a wrapper, but for now we'll pass through
        }
        
        request_body
    }
    
    fn parse_response(&self, response: Value) -> Result<ChatResponse, ProviderError> {
        let message = response["message"].as_object()
            .ok_or(ProviderError::Api("Invalid response format".to_string()))?;
            
        let content = message["content"].as_str().unwrap_or("").to_string();
        
        // Ollama doesn't support tool calls natively
        let tool_calls = None;
        
        let usage = Usage {
            input_tokens: response["prompt_eval_count"].as_u64().unwrap_or(0) as usize,
            output_tokens: response["eval_count"].as_u64().unwrap_or(0) as usize,
        };
        
        Ok(ChatResponse {
            id: response["id"].as_str().unwrap_or("ollama").to_string(),
            content,
            tool_calls,
            usage,
            finish_reason: response["done"].as_bool().unwrap_or(true)
                .then_some("stop")
                .unwrap_or("stop")
                .to_string(),
        })
    }
    
    async fn check_model_availability(&self) -> Result<(), ProviderError> {
        let response = self.client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;
            
        if response.status().is_success() {
            let models: Value = response
                .json()
                .await
                .map_err(|e| ProviderError::Api(e.to_string()))?;
                
            let model_exists = models["models"].as_array()
                .map(|models| {
                    models.iter().any(|model| {
                        model["name"].as_str()
                            .unwrap_or("")
                            .contains(&self.model)
                    })
                })
                .unwrap_or(false);
                
            if !model_exists {
                return Err(ProviderError::InvalidModel(
                    format!("Model '{}' not found in Ollama. Run: ollama pull {}", self.model, self.model)
                ));
            }
            
            Ok(())
        } else {
            Err(ProviderError::Network("Failed to connect to Ollama".to_string()))
        }
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn chat_completion(&mut self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        // Check if model is available
        self.check_model_availability().await?;
        
        let request_body = self.convert_request(request);
        
        let response = self.client
            .post(&format!("{}/api/chat", self.base_url))
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;
            
        if response.status().is_success() {
            let response_json: Value = response
                .json()
                .await
                .map_err(|e| ProviderError::Api(e.to_string()))?;
                
            self.parse_response(response_json)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(match status.as_u16() {
                404 => ProviderError::InvalidModel(format!("Model '{}' not found", self.model)),
                _ => ProviderError::Api(format!("HTTP {}: {}", status, error_text)),
            })
        }
    }
    
    fn supports_tools(&self) -> bool {
        false // Ollama doesn't support tools natively yet
    }
    
    fn max_tokens(&self) -> usize {
        self.max_tokens
    }
    
    fn model_name(&self) -> &str {
        &self.model
    }
    
    fn provider_name(&self) -> &str {
        "ollama"
    }
    
    async fn health_check(&self) -> Result<(), ProviderError> {
        // Check if Ollama is running
        let response = self.client
            .get(&format!("{}/api/version", self.base_url))
            .send()
            .await;
            
        match response {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(_) => Err(ProviderError::Network("Ollama service not responding correctly".to_string())),
            Err(_) => Err(ProviderError::Network("Cannot connect to Ollama. Is it running?".to_string())),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::ChatMessage;
    use super::ChatMessage;

    #[tokio::test]
    async fn test_convert_request() {
        let config = super::super::ProviderConfig {
            provider_type: "ollama".to_string(),
            api_key: None,
            base_url: Some("http://localhost:11434".to_string()),
            model: "codellama".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.2),
        };
        
        let provider = OllamaProvider::new(config).unwrap();
        
        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Explain this Rust code".to_string(),
                tool_calls: None,
            }],
            model: None,
            max_tokens: Some(2000),
            temperature: Some(0.2),
            tools: None,
        };
        
        let converted = provider.convert_request(request);
        assert_eq!(converted["model"], "codellama");
        assert_eq!(converted["stream"], false);
    }
}

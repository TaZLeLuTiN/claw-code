//! DeepSeek Coder V2 provider implementation

use super::{AIProvider, ChatRequest, ChatResponse, ToolCall, Usage, ProviderError};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

pub struct DeepSeekProvider {
    client: Client,
    api_key: String,
    model: String,
    max_tokens: usize,
    base_url: String,
}

impl DeepSeekProvider {
    pub fn new(config: super::ProviderConfig) -> Result<Self, ProviderError> {
        let api_key = config.api_key.ok_or(ProviderError::Authentication)?;
        
        Ok(Self {
            client: Client::new(),
            api_key,
            model: config.model,
            max_tokens: config.max_tokens.unwrap_or(4096),
            base_url: config.base_url.unwrap_or_else(|| "https://api.deepseek.com".to_string()),
        })
    }
    
    fn convert_request(&self, request: ChatRequest) -> Value {
        let messages: Vec<Value> = request.messages.into_iter().map(|msg| {
            let mut msg_obj = json!({
                "role": msg.role,
                "content": msg.content
            });
            
            // Add tool calls if present (DeepSeek supports function calling)
            if let Some(tool_calls) = msg.tool_calls {
                msg_obj["tool_calls"] = json!(tool_calls.into_iter().map(|tc| {
                    json!({
                        "id": tc.id,
                        "type": "function",
                        "function": {
                            "name": tc.name,
                            "arguments": serde_json::to_string(&tc.arguments).unwrap_or_default()
                        }
                    })
                }).collect::<Vec<_>>());
            }
            
            msg_obj
        }).collect();
        
        let mut request_body = json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(self.max_tokens),
        });
        
        if let Some(temperature) = request.temperature {
            request_body["temperature"] = json!(temperature);
        }
        
        // Add tools if present (function calling)
        if let Some(tools) = request.tools {
            request_body["tools"] = json!(tools.into_iter().map(|tool| {
                json!({
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": tool.input_schema
                    }
                })
            }).collect::<Vec<_>>());
            
            // Enable function calling
            request_body["tool_choice"] = json!("auto");
        }
        
        request_body
    }
    
    fn parse_response(&self, response: Value) -> Result<ChatResponse, ProviderError> {
        let message = response["choices"].as_array()
            .and_then(|choices| choices.first())
            .and_then(|choice| choice["message"].as_object())
            .ok_or(ProviderError::Api("Invalid response format".to_string()))?;
            
        let content = message["content"].as_str().unwrap_or("").to_string();
        
        // Parse tool calls
        let tool_calls = message["tool_calls"].as_array().map(|calls| {
            calls.iter().map(|call| {
                ToolCall {
                    id: call["id"].as_str().unwrap_or("").to_string(),
                    name: call["function"]["name"].as_str().unwrap_or("").to_string(),
                    arguments: call["function"]["arguments"].as_str()
                        .and_then(|args| serde_json::from_str(args).ok())
                        .unwrap_or(json!({})),
                }
            }).collect()
        });
        
        // Get usage information directly without reference issues
        let input_tokens = response["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as usize;
        let output_tokens = response["usage"]["completion_tokens"].as_u64().unwrap_or(0) as usize;

        
        Ok(ChatResponse {
            id: response["id"].as_str().unwrap_or("").to_string(),
            content,
            tool_calls,
            usage: Usage {
                input_tokens,
                output_tokens,
            },
            finish_reason: response["choices"].as_array()
                .and_then(|choices| choices.first())
                .and_then(|choice| choice["finish_reason"].as_str())
                .unwrap_or("stop")
                .to_string(),
        })
    }
}

#[async_trait]
impl AIProvider for DeepSeekProvider {
    async fn chat_completion(&mut self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        let request_body = self.convert_request(request);
        
        let response = self.client
            .post(&format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
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
                401 => ProviderError::Authentication,
                429 => ProviderError::RateLimit,
                _ => ProviderError::Api(format!("HTTP {}: {}", status, error_text)),
            })
        }
    }
    
    fn supports_tools(&self) -> bool {
        true // DeepSeek Coder V2 supports function calling
    }
    
    fn max_tokens(&self) -> usize {
        self.max_tokens
    }
    
    fn model_name(&self) -> &str {
        &self.model
    }
    
    fn provider_name(&self) -> &str {
        "deepseek"
    }
    
    async fn health_check(&self) -> Result<(), ProviderError> {
        // Simple health check
        Ok(())
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
            provider_type: "deepseek".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: None,
            model: "deepseek-coder-v2".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.1),
        };
        
        let provider = DeepSeekProvider::new(config).unwrap();
        
        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Write a Python function to calculate fibonacci".to_string(),
                tool_calls: None,
            }],
            model: None,
            max_tokens: Some(2000),
            temperature: Some(0.1),
            tools: None,
        };
        
        let converted = provider.convert_request(request);
        assert_eq!(converted["model"], "deepseek-coder");
        assert_eq!(converted["max_tokens"], 2000);
        assert_eq!(converted["temperature"], 0.1);
    }
}

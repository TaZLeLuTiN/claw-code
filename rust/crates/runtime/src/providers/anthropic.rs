//! Anthropic Claude provider implementation

use super::{AIProvider, ChatRequest, ChatResponse, ToolCall, Usage, ProviderError};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
    max_tokens: usize,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(config: super::ProviderConfig) -> Result<Self, ProviderError> {
        let api_key = config.api_key.ok_or(ProviderError::Authentication)?;
        
        Ok(Self {
            client: Client::new(),
            api_key,
            model: config.model,
            max_tokens: config.max_tokens.unwrap_or(4096),
            base_url: config.base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string()),
        })
    }
    
    fn convert_request(&self, request: ChatRequest) -> Value {
        let messages: Vec<Value> = request.messages.into_iter().map(|msg| {
            let mut msg_obj = json!({
                "role": msg.role,
                "content": vec![json!({
                    "type": "text",
                    "text": msg.content
                })]
            });
            
            // Add tool calls if present
            if let Some(tool_calls) = msg.tool_calls {
                let mut content = vec![json!({
                    "type": "text", 
                    "text": msg.content
                })];
                
                for tool_call in tool_calls {
                    content.push(json!({
                        "type": "tool_use",
                        "id": tool_call.id,
                        "name": tool_call.name,
                        "input": tool_call.arguments
                    }));
                }
                
                msg_obj["content"] = json!(content);
            }
            
            msg_obj
        }).collect();
        
        let mut request_body = json!({
            "model": self.model,
            "max_tokens": request.max_tokens.unwrap_or(self.max_tokens),
            "messages": messages,
        });
        
        if let Some(temperature) = request.temperature {
            request_body["temperature"] = json!(temperature);
        }
        
        // Add tools if present
        if let Some(tools) = request.tools {
            request_body["tools"] = json!(tools.into_iter().map(|tool| {
                json!({
                    "name": tool.name,
                    "description": tool.description,
                    "input_schema": tool.input_schema
                })
            }).collect::<Vec<_>>());
        }
        
        request_body
    }
    
    fn parse_response(&self, response: Value) -> Result<ChatResponse, ProviderError> {
        let content = response["content"].as_array()
            .ok_or(ProviderError::Api("Invalid response format".to_string()))?;
            
        let mut text_content = String::new();
        let mut tool_calls = Vec::new();
        
        for item in content {
            if item["type"] == "text" {
                text_content.push_str(item["text"].as_str().unwrap_or(""));
            } else if item["type"] == "tool_use" {
                tool_calls.push(ToolCall {
                    id: item["id"].as_str().unwrap_or("").to_string(),
                    name: item["name"].as_str().unwrap_or("").to_string(),
                    arguments: item["input"].clone(),
                });
            }
        }
        
        Ok(ChatResponse {
            id: response["id"].as_str().unwrap_or("").to_string(),
            content: text_content,
            tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
            usage: Usage {
                input_tokens: response["usage"]["input_tokens"].as_u64().unwrap_or(0) as usize,
                output_tokens: response["usage"]["output_tokens"].as_u64().unwrap_or(0) as usize,
            },
            finish_reason: response["stop_reason"].as_str().unwrap_or("stop").to_string(),
        })
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    async fn chat_completion(&mut self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        let request_body = self.convert_request(request);
        
        let response = self.client
            .post(&format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
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
        true
    }
    
    fn max_tokens(&self) -> usize {
        self.max_tokens
    }
    
    fn model_name(&self) -> &str {
        &self.model
    }
    
    fn provider_name(&self) -> &str {
        "anthropic"
    }
    
    async fn health_check(&self) -> Result<(), ProviderError> {
        // Simple health check - we could ping their API
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
            provider_type: "anthropic".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: None,
            model: "claude-3-opus-20240229".to_string(),
            max_tokens: Some(4096),
            temperature: None,
        };
        
        let provider = AnthropicProvider::new(config).unwrap();
        
        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                tool_calls: None,
            }],
            model: None,
            max_tokens: Some(100),
            temperature: Some(0.7),
            tools: None,
        };
        
        let converted = provider.convert_request(request);
        assert_eq!(converted["model"], "claude-3-opus-20240229");
        assert_eq!(converted["max_tokens"], 100);
        assert_eq!(converted["temperature"], 0.7);
    }
}

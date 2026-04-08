//! Multi-provider AI abstraction layer for Claw Code

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Standard request format for all AI providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: Option<String>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub tools: Option<Vec<Tool>>,
}

/// Standard message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Standard response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub usage: Usage,
    pub finish_reason: String,
}

/// Token usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

/// Error type for providers
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Authentication error")]
    Authentication,
    #[error("Rate limit exceeded")]
    RateLimit,
    #[error("Invalid model: {0}")]
    InvalidModel(String),
}

/// Core trait for all AI providers
#[async_trait]
pub trait AIProvider {
    /// Send a chat completion request
    async fn chat_completion(&mut self, request: ChatRequest) -> Result<ChatResponse, ProviderError>;
    
    /// Check if provider supports tools
    fn supports_tools(&self) -> bool;
    
    /// Get maximum tokens for this provider/model
    fn max_tokens(&self) -> usize;
    
    /// Get model name
    fn model_name(&self) -> &str;
    
    /// Get provider name
    fn provider_name(&self) -> &str;
    
    /// Check if provider is available
    async fn health_check(&self) -> Result<(), ProviderError>;
}

/// Provider factory
pub fn create_provider(config: ProviderConfig) -> Result<Box<dyn AIProvider>, ProviderError> {
    match config.provider_type.as_str() {
        "anthropic" => Ok(Box::new(anthropic::AnthropicProvider::new(config)?)),
        "deepseek" => Ok(Box::new(deepseek::DeepSeekProvider::new(config)?)),
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(config)?)),
        _ => Err(ProviderError::InvalidModel(format!("Unsupported provider: {}", config.provider_type)))
    }
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
}

// Module declarations
pub mod anthropic;
pub mod deepseek;
pub mod ollama;


impl From<std::io::Error> for ProviderError {
    fn from(error: std::io::Error) -> Self {
        ProviderError::Io(error.to_string())
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(error: serde_json::Error) -> Self {
        ProviderError::Api(error.to_string())
    }
}


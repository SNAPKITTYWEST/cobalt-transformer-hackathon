// IBM watsonx.ai Client
// Complete implementation for Granite model inference, embeddings, and rerank

use crate::ibm::auth::IamAuthenticator;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct WatsonxConfig {
    pub project_id: String,
    pub model_id: String,
    pub region: WatsonxRegion,
}

#[derive(Debug, Clone)]
pub enum WatsonxRegion {
    UsSouth,
    EuDe,
    JpTok,
    AuSyd,
}

impl WatsonxRegion {
    pub fn endpoint(&self) -> &str {
        match self {
            WatsonxRegion::UsSouth => "https://us-south.ml.cloud.ibm.com",
            WatsonxRegion::EuDe => "https://eu-de.ml.cloud.ibm.com",
            WatsonxRegion::JpTok => "https://jp-tok.ml.cloud.ibm.com",
            WatsonxRegion::AuSyd => "https://au-syd.ml.cloud.ibm.com",
        }
    }
}

pub struct WatsonxClient {
    authenticator: IamAuthenticator,
    config: WatsonxConfig,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model_id: String,
    project_id: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<ChatParameters>,
}

#[derive(Debug, Serialize)]
struct ChatParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct ModelResponse {
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize)]
struct EmbeddingsRequest {
    model_id: String,
    project_id: String,
    inputs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingsResponse {
    pub results: Vec<EmbeddingResult>,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingResult {
    pub embedding: Vec<f32>,
}

#[derive(Debug, Serialize)]
struct RerankRequest {
    model_id: String,
    project_id: String,
    query: String,
    inputs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RerankResponse {
    pub results: Vec<RerankResult>,
}

#[derive(Debug, Deserialize)]
pub struct RerankResult {
    pub index: usize,
    pub score: f32,
}

impl WatsonxClient {
    pub fn new(authenticator: IamAuthenticator, config: WatsonxConfig) -> Self {
        Self {
            authenticator,
            config,
            client: reqwest::Client::new(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("IBM_API_KEY")
            .context("IBM_API_KEY not set")?;
        let project_id = std::env::var("IBM_PROJECT_ID")
            .context("IBM_PROJECT_ID not set")?;
        let model_id = std::env::var("IBM_MODEL_ID")
            .unwrap_or_else(|_| "ibm/granite-3-3-8b-instruct".to_string());
        
        let credentials = crate::ibm::auth::Credentials::new(
            api_key,
            "https://us-south.ml.cloud.ibm.com".to_string(),
        );
        
        let config = WatsonxConfig {
            project_id,
            model_id,
            region: WatsonxRegion::UsSouth,
        };
        
        Ok(Self::new(IamAuthenticator::new(credentials), config))
    }

    pub async fn chat(&mut self, messages: Vec<ChatMessage>) -> Result<ModelResponse> {
        let token = self.authenticator.get_token().await?;
        let endpoint = self.config.region.endpoint();
        
        let request = ChatRequest {
            model_id: self.config.model_id.clone(),
            project_id: self.config.project_id.clone(),
            messages,
            parameters: Some(ChatParameters {
                max_tokens: Some(2048),
                temperature: Some(0.7),
                top_p: Some(0.95),
            }),
        };

        let response = self.client
            .post(format!("{}/ml/v1/text/chat?version=2024-01-01", endpoint))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send chat request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Chat request failed: {} - {}", status, body);
        }

        let model_response: ModelResponse = response
            .json()
            .await
            .context("Failed to parse chat response")?;

        Ok(model_response)
    }

    pub async fn explain_cobol(&mut self, cobol_code: &str) -> Result<String> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an expert COBOL programmer. Explain COBOL code clearly and concisely, focusing on business logic and data flow.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Explain this COBOL code:\n\n{}", cobol_code),
            },
        ];

        let response = self.chat(messages).await?;
        Ok(response.choices[0].message.content.clone())
    }

    pub async fn translate_to_java(&mut self, cobol_code: &str) -> Result<String> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an expert in COBOL and Java. Translate COBOL code to equivalent Java code, preserving business logic and data structures.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Translate this COBOL code to Java:\n\n{}", cobol_code),
            },
        ];

        let response = self.chat(messages).await?;
        Ok(response.choices[0].message.content.clone())
    }

    pub async fn generate_tests(&mut self, cobol_code: &str) -> Result<String> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an expert in COBOL testing. Generate comprehensive test cases for COBOL code, including edge cases and boundary conditions.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Generate test cases for this COBOL code:\n\n{}", cobol_code),
            },
        ];

        let response = self.chat(messages).await?;
        Ok(response.choices[0].message.content.clone())
    }

    pub async fn document_business_rules(&mut self, cobol_code: &str) -> Result<String> {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a business analyst. Extract and document business rules from COBOL code in clear, non-technical language.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Document the business rules in this COBOL code:\n\n{}", cobol_code),
            },
        ];

        let response = self.chat(messages).await?;
        Ok(response.choices[0].message.content.clone())
    }

    pub async fn embed_documents(&mut self, documents: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let token = self.authenticator.get_token().await?;
        let endpoint = self.config.region.endpoint();
        
        let request = EmbeddingsRequest {
            model_id: "ibm/slate-125m-english-rtrvr".to_string(),
            project_id: self.config.project_id.clone(),
            inputs: documents,
        };

        let response = self.client
            .post(format!("{}/ml/v1/text/embeddings?version=2024-01-01", endpoint))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send embeddings request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Embeddings request failed: {} - {}", status, body);
        }

        let embeddings_response: EmbeddingsResponse = response
            .json()
            .await
            .context("Failed to parse embeddings response")?;

        Ok(embeddings_response.results.into_iter().map(|r| r.embedding).collect())
    }

    pub async fn rerank(&mut self, query: String, documents: Vec<String>) -> Result<Vec<RerankResult>> {
        let token = self.authenticator.get_token().await?;
        let endpoint = self.config.region.endpoint();
        
        let request = RerankRequest {
            model_id: "ibm/slate-125m-english-rtrvr".to_string(),
            project_id: self.config.project_id.clone(),
            query,
            inputs: documents,
        };

        let response = self.client
            .post(format!("{}/ml/v1/text/rerank?version=2024-01-01", endpoint))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send rerank request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Rerank request failed: {} - {}", status, body);
        }

        let rerank_response: RerankResponse = response
            .json()
            .await
            .context("Failed to parse rerank response")?;

        Ok(rerank_response.results)
    }

    pub async fn find_similar_programs(&mut self, query_program: &str, program_library: Vec<String>) -> Result<Vec<(usize, f32)>> {
        let rerank_results = self.rerank(query_program.to_string(), program_library).await?;
        Ok(rerank_results.into_iter().map(|r| (r.index, r.score)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watsonx_config() {
        let config = WatsonxConfig {
            project_id: "test-project".to_string(),
            model_id: "ibm/granite-3-3-8b-instruct".to_string(),
            region: WatsonxRegion::UsSouth,
        };
        assert_eq!(config.region.endpoint(), "https://us-south.ml.cloud.ibm.com");
    }

    #[test]
    fn test_chat_message() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Test message".to_string(),
        };
        assert_eq!(msg.role, "user");
    }
}

// Made with Bob

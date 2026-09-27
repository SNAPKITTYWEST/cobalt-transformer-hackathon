// IBM Cloud IAM Authentication
// Handles API key exchange for bearer tokens

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct Credentials {
    pub api_key: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct IamAuthenticator {
    credentials: Credentials,
    cached_token: Option<CachedToken>,
}

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: SystemTime,
}

#[derive(Debug, Deserialize)]
struct IamTokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Serialize)]
struct IamTokenRequest {
    grant_type: String,
    apikey: String,
}

impl Credentials {
    pub fn new(api_key: String, url: String) -> Self {
        Self { api_key, url }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("IBM_API_KEY")
            .context("IBM_API_KEY environment variable not set")?;
        let url = std::env::var("IBM_WATSONX_URL")
            .unwrap_or_else(|_| "https://us-south.ml.cloud.ibm.com".to_string());
        
        Ok(Self { api_key, url })
    }
}

impl IamAuthenticator {
    pub fn new(credentials: Credentials) -> Self {
        Self {
            credentials,
            cached_token: None,
        }
    }

    pub async fn get_token(&mut self) -> Result<String> {
        // Check if cached token is still valid
        if let Some(cached) = &self.cached_token {
            if SystemTime::now() < cached.expires_at {
                return Ok(cached.access_token.clone());
            }
        }

        // Request new token
        let token = self.request_new_token().await?;
        Ok(token)
    }

    async fn request_new_token(&mut self) -> Result<String> {
        let client = reqwest::Client::new();
        
        let request = IamTokenRequest {
            grant_type: "urn:ibm:params:oauth:grant-type:apikey".to_string(),
            apikey: self.credentials.api_key.clone(),
        };

        let response = client
            .post("https://iam.cloud.ibm.com/identity/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&request)
            .send()
            .await
            .context("Failed to request IAM token")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("IAM token request failed: {} - {}", status, body);
        }

        let token_response: IamTokenResponse = response
            .json()
            .await
            .context("Failed to parse IAM token response")?;

        // Cache the token (subtract 5 minutes for safety margin)
        let expires_at = SystemTime::now() + Duration::from_secs(token_response.expires_in - 300);
        self.cached_token = Some(CachedToken {
            access_token: token_response.access_token.clone(),
            expires_at,
        });

        Ok(token_response.access_token)
    }

    pub fn get_url(&self) -> &str {
        &self.credentials.url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_creation() {
        let creds = Credentials::new(
            "test-api-key".to_string(),
            "https://us-south.ml.cloud.ibm.com".to_string(),
        );
        assert_eq!(creds.api_key, "test-api-key");
    }

    #[test]
    fn test_authenticator_creation() {
        let creds = Credentials::new(
            "test-key".to_string(),
            "https://test.ibm.com".to_string(),
        );
        let auth = IamAuthenticator::new(creds);
        assert!(auth.cached_token.is_none());
    }
}

// Made with Bob

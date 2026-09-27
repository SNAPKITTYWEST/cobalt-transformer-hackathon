// IBM Cloud Service Clients
// Object Storage (S3-compatible) and Code Engine deployment helpers

use crate::ibm::auth::IamAuthenticator;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ObjectStorageConfig {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
}

pub struct ObjectStorageClient {
    authenticator: IamAuthenticator,
    config: ObjectStorageConfig,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct CodeEngineConfig {
    pub project_id: String,
    pub region: String,
}

pub struct CodeEngineClient {
    authenticator: IamAuthenticator,
    config: CodeEngineConfig,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct CreateJobRequest {
    name: String,
    image_reference: String,
    run_env_variables: Vec<EnvVar>,
    run_arguments: Vec<String>,
}

#[derive(Debug, Serialize)]
struct EnvVar {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
pub struct JobRun {
    pub name: String,
    pub status: String,
    pub created_at: String,
}

impl ObjectStorageClient {
    pub fn new(authenticator: IamAuthenticator, config: ObjectStorageConfig) -> Self {
        Self {
            authenticator,
            config,
            client: reqwest::Client::new(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("IBM_API_KEY")
            .context("IBM_API_KEY not set")?;
        let endpoint = std::env::var("IBM_COS_ENDPOINT")
            .unwrap_or_else(|_| "https://s3.us-south.cloud-object-storage.appdomain.cloud".to_string());
        let bucket = std::env::var("IBM_COS_BUCKET")
            .context("IBM_COS_BUCKET not set")?;
        let region = std::env::var("IBM_COS_REGION")
            .unwrap_or_else(|_| "us-south".to_string());
        
        let credentials = crate::ibm::auth::Credentials::new(
            api_key,
            endpoint.clone(),
        );
        
        let config = ObjectStorageConfig {
            endpoint,
            bucket,
            region,
        };
        
        Ok(Self::new(IamAuthenticator::new(credentials), config))
    }

    pub async fn upload_file(&mut self, key: &str, data: Vec<u8>) -> Result<()> {
        let token = self.authenticator.get_token().await?;
        
        let url = format!("{}/{}/{}", self.config.endpoint, self.config.bucket, key);
        
        let response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/octet-stream")
            .body(data)
            .send()
            .await
            .context("Failed to upload file")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Upload failed: {} - {}", status, body);
        }

        Ok(())
    }

    pub async fn download_file(&mut self, key: &str) -> Result<Vec<u8>> {
        let token = self.authenticator.get_token().await?;
        
        let url = format!("{}/{}/{}", self.config.endpoint, self.config.bucket, key);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to download file")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Download failed: {} - {}", status, body);
        }

        let data = response.bytes().await?.to_vec();
        Ok(data)
    }

    pub async fn list_objects(&mut self, prefix: Option<&str>) -> Result<Vec<String>> {
        let token = self.authenticator.get_token().await?;
        
        let mut url = format!("{}/{}?list-type=2", self.config.endpoint, self.config.bucket);
        if let Some(p) = prefix {
            url.push_str(&format!("&prefix={}", p));
        }
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to list objects")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("List failed: {} - {}", status, body);
        }

        let body = response.text().await?;
        
        // Parse XML response (simplified)
        let keys: Vec<String> = body
            .split("<Key>")
            .skip(1)
            .filter_map(|s| s.split("</Key>").next())
            .map(|s| s.to_string())
            .collect();

        Ok(keys)
    }

    pub async fn delete_object(&mut self, key: &str) -> Result<()> {
        let token = self.authenticator.get_token().await?;
        
        let url = format!("{}/{}/{}", self.config.endpoint, self.config.bucket, key);
        
        let response = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to delete object")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Delete failed: {} - {}", status, body);
        }

        Ok(())
    }

    pub async fn upload_cobol_source(&mut self, program_name: &str, source: &str) -> Result<String> {
        let key = format!("cobol/source/{}.cob", program_name);
        self.upload_file(&key, source.as_bytes().to_vec()).await?;
        Ok(key)
    }

    pub async fn upload_jcl(&mut self, job_name: &str, jcl: &str) -> Result<String> {
        let key = format!("jcl/{}.jcl", job_name);
        self.upload_file(&key, jcl.as_bytes().to_vec()).await?;
        Ok(key)
    }

    pub async fn upload_vector_chunk(&mut self, chunk_id: &str, chunk_data: Vec<u8>) -> Result<String> {
        let key = format!("chunks/{}.bin", chunk_id);
        self.upload_file(&key, chunk_data).await?;
        Ok(key)
    }
}

impl CodeEngineClient {
    pub fn new(authenticator: IamAuthenticator, config: CodeEngineConfig) -> Self {
        Self {
            authenticator,
            config,
            client: reqwest::Client::new(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("IBM_API_KEY")
            .context("IBM_API_KEY not set")?;
        let project_id = std::env::var("IBM_CE_PROJECT_ID")
            .context("IBM_CE_PROJECT_ID not set")?;
        let region = std::env::var("IBM_CE_REGION")
            .unwrap_or_else(|_| "us-south".to_string());
        
        let credentials = crate::ibm::auth::Credentials::new(
            api_key,
            format!("https://api.{}.codeengine.cloud.ibm.com", region),
        );
        
        let config = CodeEngineConfig {
            project_id,
            region,
        };
        
        Ok(Self::new(IamAuthenticator::new(credentials), config))
    }

    pub async fn create_job(&mut self, name: &str, image: &str, env_vars: Vec<(String, String)>) -> Result<String> {
        let token = self.authenticator.get_token().await?;
        
        let endpoint = format!("https://api.{}.codeengine.cloud.ibm.com", self.config.region);
        let url = format!("{}/v2/projects/{}/jobs", endpoint, self.config.project_id);
        
        let request = CreateJobRequest {
            name: name.to_string(),
            image_reference: image.to_string(),
            run_env_variables: env_vars.into_iter().map(|(k, v)| EnvVar { name: k, value: v }).collect(),
            run_arguments: vec![],
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to create job")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Job creation failed: {} - {}", status, body);
        }

        Ok(name.to_string())
    }

    pub async fn run_job(&mut self, job_name: &str) -> Result<JobRun> {
        let token = self.authenticator.get_token().await?;
        
        let endpoint = format!("https://api.{}.codeengine.cloud.ibm.com", self.config.region);
        let url = format!("{}/v2/projects/{}/job_runs", endpoint, self.config.project_id);
        
        let request = serde_json::json!({
            "job_name": job_name,
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to run job")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Job run failed: {} - {}", status, body);
        }

        let job_run: JobRun = response.json().await?;
        Ok(job_run)
    }

    pub async fn get_job_status(&mut self, run_name: &str) -> Result<String> {
        let token = self.authenticator.get_token().await?;
        
        let endpoint = format!("https://api.{}.codeengine.cloud.ibm.com", self.config.region);
        let url = format!("{}/v2/projects/{}/job_runs/{}", endpoint, self.config.project_id, run_name);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to get job status")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Status check failed: {} - {}", status, body);
        }

        let job_run: JobRun = response.json().await?;
        Ok(job_run.status)
    }

    pub async fn deploy_triton_job(&mut self, job_name: &str, cobol_source_key: &str) -> Result<String> {
        let env_vars = vec![
            ("COBOL_SOURCE_KEY".to_string(), cobol_source_key.to_string()),
            ("VECTOR_WIDTH".to_string(), "8".to_string()),
            ("ENABLE_VECTORIZATION".to_string(), "true".to_string()),
        ];

        self.create_job(
            job_name,
            "icr.io/triton/cobol-vector-engine:latest",
            env_vars,
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_storage_config() {
        let config = ObjectStorageConfig {
            endpoint: "https://s3.us-south.cloud-object-storage.appdomain.cloud".to_string(),
            bucket: "test-bucket".to_string(),
            region: "us-south".to_string(),
        };
        assert_eq!(config.region, "us-south");
    }

    #[test]
    fn test_code_engine_config() {
        let config = CodeEngineConfig {
            project_id: "test-project".to_string(),
            region: "us-south".to_string(),
        };
        assert_eq!(config.region, "us-south");
    }
}

// Made with Bob

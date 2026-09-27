// IBM Cloud Integration Module
// Authentication, watsonx.ai, Object Storage, and Code Engine

pub mod auth;
pub mod watsonx;
pub mod client;

pub use auth::{Credentials, IamAuthenticator};
pub use watsonx::{WatsonxClient, WatsonxConfig, WatsonxRegion, ChatMessage};
pub use client::{ObjectStorageClient, ObjectStorageConfig, CodeEngineClient, CodeEngineConfig};

use anyhow::Result;

/// Complete IBM Cloud integration for TRITON
pub struct TritonIbmIntegration {
    pub watsonx: WatsonxClient,
    pub storage: ObjectStorageClient,
    pub code_engine: CodeEngineClient,
}

impl TritonIbmIntegration {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            watsonx: WatsonxClient::from_env()?,
            storage: ObjectStorageClient::from_env()?,
            code_engine: CodeEngineClient::from_env()?,
        })
    }

    /// Complete workflow: Parse COBOL, explain with watsonx, store results, deploy job
    pub async fn process_cobol_program(
        &mut self,
        program_name: &str,
        cobol_source: &str,
    ) -> Result<ProcessingResult> {
        // 1. Upload source to Object Storage
        let source_key = self.storage.upload_cobol_source(program_name, cobol_source).await?;

        // 2. Get AI explanation
        let explanation = self.watsonx.explain_cobol(cobol_source).await?;

        // 3. Generate business rules documentation
        let business_rules = self.watsonx.document_business_rules(cobol_source).await?;

        // 4. Generate test cases
        let test_cases = self.watsonx.generate_tests(cobol_source).await?;

        // 5. Store documentation
        let docs_key = format!("docs/{}_explanation.md", program_name);
        self.storage.upload_file(&docs_key, explanation.as_bytes().to_vec()).await?;

        let rules_key = format!("docs/{}_business_rules.md", program_name);
        self.storage.upload_file(&rules_key, business_rules.as_bytes().to_vec()).await?;

        let tests_key = format!("tests/{}_tests.md", program_name);
        self.storage.upload_file(&tests_key, test_cases.as_bytes().to_vec()).await?;

        // 6. Deploy Code Engine job for vectorized execution
        let job_name = format!("triton-{}", program_name);
        let job_id = self.code_engine.deploy_triton_job(&job_name, &source_key).await?;

        Ok(ProcessingResult {
            program_name: program_name.to_string(),
            source_key,
            docs_key,
            rules_key,
            tests_key,
            job_id,
            explanation,
            business_rules,
            test_cases,
        })
    }

    /// Batch process multiple COBOL programs
    pub async fn process_batch(
        &mut self,
        programs: Vec<(String, String)>,
    ) -> Result<Vec<ProcessingResult>> {
        let mut results = Vec::new();
        
        for (name, source) in programs {
            match self.process_cobol_program(&name, &source).await {
                Ok(result) => results.push(result),
                Err(e) => eprintln!("Failed to process {}: {}", name, e),
            }
        }

        Ok(results)
    }

    /// Semantic search across COBOL programs using embeddings
    pub async fn search_programs(
        &mut self,
        query: &str,
        program_sources: Vec<String>,
    ) -> Result<Vec<(usize, f32)>> {
        self.watsonx.find_similar_programs(query, program_sources).await
    }
}

#[derive(Debug)]
pub struct ProcessingResult {
    pub program_name: String,
    pub source_key: String,
    pub docs_key: String,
    pub rules_key: String,
    pub tests_key: String,
    pub job_id: String,
    pub explanation: String,
    pub business_rules: String,
    pub test_cases: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_result() {
        let result = ProcessingResult {
            program_name: "TEST".to_string(),
            source_key: "cobol/source/TEST.cob".to_string(),
            docs_key: "docs/TEST_explanation.md".to_string(),
            rules_key: "docs/TEST_business_rules.md".to_string(),
            tests_key: "tests/TEST_tests.md".to_string(),
            job_id: "triton-TEST".to_string(),
            explanation: "Test explanation".to_string(),
            business_rules: "Test rules".to_string(),
            test_cases: "Test cases".to_string(),
        };
        assert_eq!(result.program_name, "TEST");
    }
}

// Made with Bob

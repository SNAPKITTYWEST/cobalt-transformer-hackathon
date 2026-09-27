// IBM Cloud Integration Tests
// Tests for watsonx.ai, Object Storage, and Code Engine clients

use cobol_transformer::ibm::{
    WatsonxClient, WatsonxConfig, WatsonxRegion, ChatMessage,
    ObjectStorageClient, ObjectStorageConfig,
    CodeEngineClient, CodeEngineConfig,
    TritonIbmIntegration,
};
use cobol_transformer::ibm::auth::{Credentials, IamAuthenticator};

#[test]
fn test_watsonx_config() {
    let config = WatsonxConfig {
        project_id: "test-project-123".to_string(),
        model_id: "ibm/granite-3-3-8b-instruct".to_string(),
        region: WatsonxRegion::UsSouth,
    };

    assert_eq!(config.project_id, "test-project-123");
    assert_eq!(config.model_id, "ibm/granite-3-3-8b-instruct");
    assert_eq!(config.region.endpoint(), "https://us-south.ml.cloud.ibm.com");
}

#[test]
fn test_watsonx_regions() {
    assert_eq!(WatsonxRegion::UsSouth.endpoint(), "https://us-south.ml.cloud.ibm.com");
    assert_eq!(WatsonxRegion::EuDe.endpoint(), "https://eu-de.ml.cloud.ibm.com");
    assert_eq!(WatsonxRegion::JpTok.endpoint(), "https://jp-tok.ml.cloud.ibm.com");
    assert_eq!(WatsonxRegion::AuSyd.endpoint(), "https://au-syd.ml.cloud.ibm.com");
}

#[test]
fn test_chat_message_creation() {
    let msg = ChatMessage {
        role: "user".to_string(),
        content: "Explain COBOL".to_string(),
    };

    assert_eq!(msg.role, "user");
    assert_eq!(msg.content, "Explain COBOL");
}

#[test]
fn test_object_storage_config() {
    let config = ObjectStorageConfig {
        endpoint: "https://s3.us-south.cloud-object-storage.appdomain.cloud".to_string(),
        bucket: "triton-cobol-bucket".to_string(),
        region: "us-south".to_string(),
    };

    assert_eq!(config.bucket, "triton-cobol-bucket");
    assert_eq!(config.region, "us-south");
    assert!(config.endpoint.contains("s3.us-south"));
}

#[test]
fn test_code_engine_config() {
    let config = CodeEngineConfig {
        project_id: "ce-project-456".to_string(),
        region: "us-south".to_string(),
    };

    assert_eq!(config.project_id, "ce-project-456");
    assert_eq!(config.region, "us-south");
}

#[test]
fn test_credentials_creation() {
    let creds = Credentials::new(
        "test-api-key-123".to_string(),
        "https://iam.cloud.ibm.com".to_string(),
    );

    assert_eq!(creds.api_key, "test-api-key-123");
    assert_eq!(creds.url, "https://iam.cloud.ibm.com");
}

#[test]
fn test_authenticator_creation() {
    let creds = Credentials::new(
        "test-key".to_string(),
        "https://iam.cloud.ibm.com".to_string(),
    );
    let auth = IamAuthenticator::new(creds);

    // Authenticator should be created successfully
    assert!(std::mem::size_of_val(&auth) > 0);
}

#[test]
fn test_watsonx_client_creation() {
    let creds = Credentials::new(
        "test-key".to_string(),
        "https://us-south.ml.cloud.ibm.com".to_string(),
    );
    let auth = IamAuthenticator::new(creds);
    let config = WatsonxConfig {
        project_id: "test-project".to_string(),
        model_id: "ibm/granite-3-3-8b-instruct".to_string(),
        region: WatsonxRegion::UsSouth,
    };

    let client = WatsonxClient::new(auth, config);
    assert!(std::mem::size_of_val(&client) > 0);
}

#[test]
fn test_object_storage_client_creation() {
    let creds = Credentials::new(
        "test-key".to_string(),
        "https://s3.us-south.cloud-object-storage.appdomain.cloud".to_string(),
    );
    let auth = IamAuthenticator::new(creds);
    let config = ObjectStorageConfig {
        endpoint: "https://s3.us-south.cloud-object-storage.appdomain.cloud".to_string(),
        bucket: "test-bucket".to_string(),
        region: "us-south".to_string(),
    };

    let client = ObjectStorageClient::new(auth, config);
    assert!(std::mem::size_of_val(&client) > 0);
}

#[test]
fn test_code_engine_client_creation() {
    let creds = Credentials::new(
        "test-key".to_string(),
        "https://api.us-south.codeengine.cloud.ibm.com".to_string(),
    );
    let auth = IamAuthenticator::new(creds);
    let config = CodeEngineConfig {
        project_id: "test-project".to_string(),
        region: "us-south".to_string(),
    };

    let client = CodeEngineClient::new(auth, config);
    assert!(std::mem::size_of_val(&client) > 0);
}

// Integration tests that require actual credentials
// These are skipped by default and run only when credentials are available

#[tokio::test]
#[ignore] // Run with: cargo test --ignored
async fn test_watsonx_chat_integration() {
    if let Ok(mut client) = WatsonxClient::from_env() {
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "What is 2+2?".to_string(),
            },
        ];

        let result = client.chat(messages).await;
        assert!(result.is_ok(), "Chat should succeed with valid credentials");

        let response = result.unwrap();
        assert!(!response.choices.is_empty(), "Should have at least one choice");
        assert!(!response.choices[0].message.content.is_empty(), "Response should not be empty");
    }
}

#[tokio::test]
#[ignore]
async fn test_cobol_explanation_integration() {
    if let Ok(mut client) = WatsonxClient::from_env() {
        let cobol = r#"
            IDENTIFICATION DIVISION.
            PROGRAM-ID. HELLO.
            PROCEDURE DIVISION.
                DISPLAY "HELLO WORLD".
                STOP RUN.
        "#;

        let result = client.explain_cobol(cobol).await;
        assert!(result.is_ok(), "Explanation should succeed");

        let explanation = result.unwrap();
        assert!(!explanation.is_empty(), "Explanation should not be empty");
        assert!(explanation.to_lowercase().contains("display") || 
                explanation.to_lowercase().contains("hello"),
                "Explanation should mention key COBOL elements");
    }
}

#[tokio::test]
#[ignore]
async fn test_embeddings_integration() {
    if let Ok(mut client) = WatsonxClient::from_env() {
        let documents = vec![
            "COBOL program for customer processing".to_string(),
            "Inventory management system".to_string(),
        ];

        let result = client.embed_documents(documents).await;
        assert!(result.is_ok(), "Embeddings should succeed");

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 2, "Should have 2 embeddings");
        assert!(!embeddings[0].is_empty(), "Embedding should not be empty");
    }
}

#[tokio::test]
#[ignore]
async fn test_object_storage_integration() {
    if let Ok(mut client) = ObjectStorageClient::from_env() {
        let test_data = b"Test COBOL source code".to_vec();
        let key = "test/integration_test.cob";

        // Upload
        let upload_result = client.upload_file(key, test_data.clone()).await;
        assert!(upload_result.is_ok(), "Upload should succeed");

        // Download
        let download_result = client.download_file(key).await;
        assert!(download_result.is_ok(), "Download should succeed");
        assert_eq!(download_result.unwrap(), test_data, "Downloaded data should match");

        // Cleanup
        let _ = client.delete_object(key).await;
    }
}

#[tokio::test]
#[ignore]
async fn test_complete_workflow_integration() {
    if let Ok(mut integration) = TritonIbmIntegration::from_env() {
        let cobol = r#"
            IDENTIFICATION DIVISION.
            PROGRAM-ID. TEST-PROG.
            DATA DIVISION.
            WORKING-STORAGE SECTION.
            01  COUNTER PIC 9(4) VALUE ZERO.
            PROCEDURE DIVISION.
                PERFORM VARYING COUNTER FROM 1 BY 1 UNTIL COUNTER > 10
                    DISPLAY COUNTER
                END-PERFORM.
                STOP RUN.
        "#;

        let result = integration.process_cobol_program("TEST-PROG", cobol).await;
        assert!(result.is_ok(), "Complete workflow should succeed");

        let processing_result = result.unwrap();
        assert_eq!(processing_result.program_name, "TEST-PROG");
        assert!(!processing_result.explanation.is_empty());
        assert!(!processing_result.business_rules.is_empty());
        assert!(!processing_result.test_cases.is_empty());
    }
}

#[test]
fn test_processing_result_structure() {
    use cobol_transformer::ibm::ProcessingResult;

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
    assert!(result.source_key.ends_with(".cob"));
    assert!(result.docs_key.contains("explanation"));
    assert!(result.rules_key.contains("business_rules"));
    assert!(result.tests_key.contains("tests"));
    assert!(result.job_id.starts_with("triton-"));
}

// Made with Bob

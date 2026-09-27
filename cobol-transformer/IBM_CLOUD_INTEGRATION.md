# IBM Cloud Integration for TRITON

Complete IBM watsonx.ai, Object Storage, and Code Engine integration for the TRITON COBOL Vector Engine.

## Overview

This integration connects TRITON's COBOL parsing and vectorization capabilities with IBM Cloud services:

- **watsonx.ai**: Granite models for COBOL explanation, translation, and documentation
- **Object Storage**: S3-compatible storage for COBOL sources, JCL, and vector chunks
- **Code Engine**: Serverless execution of vectorized COBOL workloads

## Architecture

```
COBOL Source
     ↓
TRITON Parser & Vectorizer
     ↓
     +------------------+------------------+
     ↓                  ↓                  ↓
watsonx.ai        Object Storage    Code Engine
(Explanation)     (Persistence)     (Execution)
```

## Components

### 1. Authentication (`src/ibm/auth.rs`)

IAM token-based authentication with automatic token refresh:

```rust
use cobol_transformer::ibm::auth::{Credentials, IamAuthenticator};

let credentials = Credentials::new(
    "your-api-key".to_string(),
    "https://iam.cloud.ibm.com".to_string(),
);

let mut authenticator = IamAuthenticator::new(credentials);
let token = authenticator.get_token().await?;
```

**Features:**
- Automatic token caching
- Token expiration handling
- Thread-safe token refresh

### 2. watsonx.ai Client (`src/ibm/watsonx.rs`)

Complete watsonx.ai integration with Granite models:

```rust
use cobol_transformer::ibm::{WatsonxClient, ChatMessage};

let mut client = WatsonxClient::from_env()?;

// Explain COBOL code
let explanation = client.explain_cobol(cobol_source).await?;

// Translate to Java
let java_code = client.translate_to_java(cobol_source).await?;

// Generate test cases
let tests = client.generate_tests(cobol_source).await?;

// Document business rules
let rules = client.document_business_rules(cobol_source).await?;
```

**Supported Operations:**
- Chat completion with Granite models
- COBOL code explanation
- Translation to Java/Python
- Test case generation
- Business rules documentation
- Text embeddings (Slate models)
- Semantic reranking
- Program similarity search

### 3. Object Storage Client (`src/ibm/client.rs`)

S3-compatible IBM Cloud Object Storage:

```rust
use cobol_transformer::ibm::ObjectStorageClient;

let mut client = ObjectStorageClient::from_env()?;

// Upload COBOL source
let key = client.upload_cobol_source("PROGRAM", source).await?;

// Upload JCL
let jcl_key = client.upload_jcl("JOB001", jcl).await?;

// Upload vector chunk
let chunk_key = client.upload_vector_chunk("chunk-001", data).await?;

// List objects
let objects = client.list_objects(Some("cobol/")).await?;

// Download
let data = client.download_file(&key).await?;
```

**Features:**
- File upload/download
- Object listing with prefix filtering
- Batch operations
- Specialized COBOL/JCL helpers

### 4. Code Engine Client (`src/ibm/client.rs`)

Serverless job execution on IBM Code Engine:

```rust
use cobol_transformer::ibm::CodeEngineClient;

let mut client = CodeEngineClient::from_env()?;

// Create job
let job_id = client.create_job(
    "triton-job",
    "icr.io/triton/cobol-vector-engine:latest",
    vec![
        ("COBOL_SOURCE_KEY".to_string(), "cobol/source/PROG.cob".to_string()),
        ("VECTOR_WIDTH".to_string(), "8".to_string()),
    ],
).await?;

// Run job
let run = client.run_job("triton-job").await?;

// Check status
let status = client.get_job_status(&run.name).await?;
```

**Features:**
- Job creation and configuration
- Job execution with environment variables
- Status monitoring
- TRITON-specific deployment helpers

### 5. Complete Integration (`src/ibm/mod.rs`)

End-to-end workflow orchestration:

```rust
use cobol_transformer::ibm::TritonIbmIntegration;

let mut integration = TritonIbmIntegration::from_env()?;

// Complete workflow: parse → explain → store → deploy
let result = integration.process_cobol_program(
    "CUSTOMER-REPORT",
    cobol_source,
).await?;

println!("Explanation: {}", result.explanation);
println!("Business Rules: {}", result.business_rules);
println!("Test Cases: {}", result.test_cases);
println!("Job ID: {}", result.job_id);
```

**Workflow Steps:**
1. Upload COBOL source to Object Storage
2. Generate AI explanation with watsonx.ai
3. Document business rules
4. Generate test cases
5. Store all documentation
6. Deploy Code Engine job for vectorized execution

## Configuration

### Environment Variables

```bash
# Required for all services
export IBM_API_KEY="your-ibm-cloud-api-key"

# watsonx.ai
export IBM_PROJECT_ID="your-watsonx-project-id"
export IBM_MODEL_ID="ibm/granite-3-3-8b-instruct"  # Optional, defaults to Granite

# Object Storage
export IBM_COS_ENDPOINT="https://s3.us-south.cloud-object-storage.appdomain.cloud"
export IBM_COS_BUCKET="your-bucket-name"
export IBM_COS_REGION="us-south"

# Code Engine
export IBM_CE_PROJECT_ID="your-code-engine-project-id"
export IBM_CE_REGION="us-south"
```

### Obtaining Credentials

1. **IBM Cloud API Key:**
   - Go to https://cloud.ibm.com/iam/apikeys
   - Create a new API key
   - Save it securely

2. **watsonx.ai Project:**
   - Go to https://dataplatform.cloud.ibm.com/wx/home
   - Create a new project
   - Copy the project ID from project settings

3. **Object Storage:**
   - Create a Cloud Object Storage instance
   - Create a bucket
   - Note the endpoint and bucket name

4. **Code Engine:**
   - Create a Code Engine project
   - Copy the project ID

## Examples

### Example 1: COBOL Explanation

```rust
use cobol_transformer::ibm::WatsonxClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = WatsonxClient::from_env()?;
    
    let cobol = r#"
        IDENTIFICATION DIVISION.
        PROGRAM-ID. HELLO.
        PROCEDURE DIVISION.
            DISPLAY "HELLO WORLD".
            STOP RUN.
    "#;
    
    let explanation = client.explain_cobol(cobol).await?;
    println!("{}", explanation);
    
    Ok(())
}
```

### Example 2: Batch Processing

```rust
use cobol_transformer::ibm::TritonIbmIntegration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut integration = TritonIbmIntegration::from_env()?;
    
    let programs = vec![
        ("CUSTOMER-REPORT".to_string(), customer_cobol),
        ("INVENTORY-UPDATE".to_string(), inventory_cobol),
        ("PAYROLL-CALC".to_string(), payroll_cobol),
    ];
    
    let results = integration.process_batch(programs).await?;
    
    for result in results {
        println!("Processed: {} -> {}", result.program_name, result.job_id);
    }
    
    Ok(())
}
```

### Example 3: Semantic Search

```rust
use cobol_transformer::ibm::TritonIbmIntegration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut integration = TritonIbmIntegration::from_env()?;
    
    let query = "Find programs that process customer accounts";
    let programs = vec![
        "PROGRAM-ID. CUSTOMER-REPORT. Process customer records.".to_string(),
        "PROGRAM-ID. INVENTORY-UPDATE. Update inventory levels.".to_string(),
        "PROGRAM-ID. ACCOUNT-BALANCE. Check account balances.".to_string(),
    ];
    
    let results = integration.search_programs(query, programs).await?;
    
    for (idx, score) in results {
        println!("Match {}: score {:.3}", idx, score);
    }
    
    Ok(())
}
```

## Running the Demo

```bash
# Set environment variables
export IBM_API_KEY="your-key"
export IBM_PROJECT_ID="your-project"
export IBM_COS_BUCKET="your-bucket"
export IBM_CE_PROJECT_ID="your-ce-project"

# Run the demo
cargo run --example ibm_integration_demo
```

## Testing

### Unit Tests

```bash
cargo test --lib ibm
```

### Integration Tests (requires credentials)

```bash
cargo test --test ibm_integration_test --ignored
```

## API Reference

### WatsonxClient

- `chat(messages: Vec<ChatMessage>) -> Result<ModelResponse>`
- `explain_cobol(code: &str) -> Result<String>`
- `translate_to_java(code: &str) -> Result<String>`
- `generate_tests(code: &str) -> Result<String>`
- `document_business_rules(code: &str) -> Result<String>`
- `embed_documents(docs: Vec<String>) -> Result<Vec<Vec<f32>>>`
- `rerank(query: String, docs: Vec<String>) -> Result<Vec<RerankResult>>`
- `find_similar_programs(query: &str, programs: Vec<String>) -> Result<Vec<(usize, f32)>>`

### ObjectStorageClient

- `upload_file(key: &str, data: Vec<u8>) -> Result<()>`
- `download_file(key: &str) -> Result<Vec<u8>>`
- `list_objects(prefix: Option<&str>) -> Result<Vec<String>>`
- `delete_object(key: &str) -> Result<()>`
- `upload_cobol_source(name: &str, source: &str) -> Result<String>`
- `upload_jcl(name: &str, jcl: &str) -> Result<String>`
- `upload_vector_chunk(id: &str, data: Vec<u8>) -> Result<String>`

### CodeEngineClient

- `create_job(name: &str, image: &str, env: Vec<(String, String)>) -> Result<String>`
- `run_job(name: &str) -> Result<JobRun>`
- `get_job_status(run_name: &str) -> Result<String>`
- `deploy_triton_job(name: &str, source_key: &str) -> Result<String>`

### TritonIbmIntegration

- `process_cobol_program(name: &str, source: &str) -> Result<ProcessingResult>`
- `process_batch(programs: Vec<(String, String)>) -> Result<Vec<ProcessingResult>>`
- `search_programs(query: &str, programs: Vec<String>) -> Result<Vec<(usize, f32)>>`

## Supported Models

### watsonx.ai Text Generation

- `ibm/granite-3-3-8b-instruct` (default)
- `ibm/granite-3-3-2b-instruct`
- `ibm/granite-20b-code-instruct`
- `meta-llama/llama-3-70b-instruct`

### watsonx.ai Embeddings

- `ibm/slate-125m-english-rtrvr` (default)
- `ibm/slate-30m-english-rtrvr`

## Performance

- **Token Caching**: IAM tokens cached for 50 minutes
- **Batch Processing**: Parallel processing of multiple programs
- **Streaming**: Support for streaming responses (future)
- **Connection Pooling**: Reused HTTP connections

## Error Handling

All operations return `Result<T, anyhow::Error>` with detailed error messages:

```rust
match client.explain_cobol(source).await {
    Ok(explanation) => println!("{}", explanation),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Security

- API keys stored in environment variables
- Tokens cached in memory only
- HTTPS for all API calls
- IAM-based authentication

## Limitations

- Maximum file size: 5GB (Object Storage)
- Token limit: 8192 tokens (Granite models)
- Rate limits apply per IBM Cloud account

## Support

For issues or questions:
- IBM Cloud Documentation: https://cloud.ibm.com/docs
- watsonx.ai Documentation: https://dataplatform.cloud.ibm.com/docs/content/wsj/analyze-data/fm-overview.html
- TRITON Issues: https://github.com/your-org/triton/issues

## License

MIT License - See LICENSE file for details
// IBM Cloud Integration Demo
// Complete end-to-end example: COBOL parsing → watsonx.ai → Object Storage → Code Engine

use cobol_transformer::ibm::{TritonIbmIntegration, WatsonxClient, ChatMessage};
use anyhow::Result;

const SAMPLE_COBOL: &str = r#"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. CUSTOMER-REPORT.
       
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  CUSTOMER-RECORD.
           05  CUSTOMER-ID         PIC 9(8).
           05  CUSTOMER-NAME       PIC X(30).
           05  ACCOUNT-BALANCE     PIC 9(7)V99 COMP-3.
           05  CREDIT-LIMIT        PIC 9(7)V99 COMP-3.
       
       01  REPORT-TOTALS.
           05  TOTAL-CUSTOMERS     PIC 9(6) VALUE ZERO.
           05  TOTAL-BALANCE       PIC 9(9)V99 VALUE ZERO.
       
       PROCEDURE DIVISION.
       MAIN-LOGIC.
           PERFORM PROCESS-CUSTOMERS
               VARYING CUSTOMER-ID FROM 1 BY 1
               UNTIL CUSTOMER-ID > 1000.
           
           DISPLAY "TOTAL CUSTOMERS: " TOTAL-CUSTOMERS.
           DISPLAY "TOTAL BALANCE: " TOTAL-BALANCE.
           STOP RUN.
       
       PROCESS-CUSTOMERS.
           ADD 1 TO TOTAL-CUSTOMERS.
           ADD ACCOUNT-BALANCE TO TOTAL-BALANCE.
"#;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== TRITON IBM Cloud Integration Demo ===\n");

    // Example 1: Direct watsonx.ai interaction
    println!("1. Testing watsonx.ai Granite model...");
    demo_watsonx_chat().await?;

    // Example 2: COBOL explanation
    println!("\n2. Explaining COBOL code with watsonx.ai...");
    demo_cobol_explanation().await?;

    // Example 3: Complete integration workflow
    println!("\n3. Running complete integration workflow...");
    demo_complete_workflow().await?;

    // Example 4: Semantic search
    println!("\n4. Semantic search across COBOL programs...");
    demo_semantic_search().await?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

async fn demo_watsonx_chat() -> Result<()> {
    // This requires IBM_API_KEY and IBM_PROJECT_ID environment variables
    match WatsonxClient::from_env() {
        Ok(mut client) => {
            let messages = vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "You are a helpful assistant.".to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: "What is COBOL?".to_string(),
                },
            ];

            match client.chat(messages).await {
                Ok(response) => {
                    println!("✓ watsonx.ai response:");
                    println!("{}", response.choices[0].message.content);
                    if let Some(usage) = response.usage {
                        println!("  Tokens: {} prompt + {} completion = {} total",
                            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens);
                    }
                }
                Err(e) => println!("✗ Chat failed: {}", e),
            }
        }
        Err(e) => println!("✗ Skipping (credentials not configured): {}", e),
    }
    Ok(())
}

async fn demo_cobol_explanation() -> Result<()> {
    match WatsonxClient::from_env() {
        Ok(mut client) => {
            match client.explain_cobol(SAMPLE_COBOL).await {
                Ok(explanation) => {
                    println!("✓ COBOL Explanation:");
                    println!("{}", explanation);
                }
                Err(e) => println!("✗ Explanation failed: {}", e),
            }
        }
        Err(e) => println!("✗ Skipping (credentials not configured): {}", e),
    }
    Ok(())
}

async fn demo_complete_workflow() -> Result<()> {
    match TritonIbmIntegration::from_env() {
        Ok(mut integration) => {
            println!("✓ IBM Cloud integration initialized");
            
            match integration.process_cobol_program("CUSTOMER-REPORT", SAMPLE_COBOL).await {
                Ok(result) => {
                    println!("✓ Processing complete:");
                    println!("  Program: {}", result.program_name);
                    println!("  Source stored: {}", result.source_key);
                    println!("  Documentation: {}", result.docs_key);
                    println!("  Business rules: {}", result.rules_key);
                    println!("  Test cases: {}", result.tests_key);
                    println!("  Code Engine job: {}", result.job_id);
                    println!("\n  Explanation preview:");
                    println!("  {}", result.explanation.lines().take(3).collect::<Vec<_>>().join("\n  "));
                }
                Err(e) => println!("✗ Processing failed: {}", e),
            }
        }
        Err(e) => println!("✗ Skipping (credentials not configured): {}", e),
    }
    Ok(())
}

async fn demo_semantic_search() -> Result<()> {
    match TritonIbmIntegration::from_env() {
        Ok(mut integration) => {
            let programs = vec![
                "PROGRAM-ID. CUSTOMER-REPORT. Process customer records.".to_string(),
                "PROGRAM-ID. INVENTORY-UPDATE. Update inventory levels.".to_string(),
                "PROGRAM-ID. PAYROLL-CALC. Calculate employee payroll.".to_string(),
                "PROGRAM-ID. ACCOUNT-BALANCE. Check account balances.".to_string(),
            ];

            let query = "Find programs that work with customer data";

            match integration.search_programs(query, programs.clone()).await {
                Ok(results) => {
                    println!("✓ Semantic search results for: '{}'", query);
                    for (idx, score) in results.iter().take(3) {
                        println!("  {:.3} - {}", score, programs[*idx]);
                    }
                }
                Err(e) => println!("✗ Search failed: {}", e),
            }
        }
        Err(e) => println!("✗ Skipping (credentials not configured): {}", e),
    }
    Ok(())
}

// Batch processing example
#[allow(dead_code)]
async fn demo_batch_processing() -> Result<()> {
    let mut integration = TritonIbmIntegration::from_env()?;

    let programs = vec![
        ("CUSTOMER-REPORT".to_string(), SAMPLE_COBOL.to_string()),
        ("INVENTORY-UPDATE".to_string(), "...".to_string()),
        ("PAYROLL-CALC".to_string(), "...".to_string()),
    ];

    let results = integration.process_batch(programs).await?;
    
    println!("Processed {} programs:", results.len());
    for result in results {
        println!("  - {}: {}", result.program_name, result.job_id);
    }

    Ok(())
}

// Translation example
#[allow(dead_code)]
async fn demo_translation() -> Result<()> {
    let mut client = WatsonxClient::from_env()?;

    let java_code = client.translate_to_java(SAMPLE_COBOL).await?;
    println!("Translated to Java:");
    println!("{}", java_code);

    Ok(())
}

// Test generation example
#[allow(dead_code)]
async fn demo_test_generation() -> Result<()> {
    let mut client = WatsonxClient::from_env()?;

    let tests = client.generate_tests(SAMPLE_COBOL).await?;
    println!("Generated test cases:");
    println!("{}", tests);

    Ok(())
}

// Business rules documentation example
#[allow(dead_code)]
async fn demo_business_rules() -> Result<()> {
    let mut client = WatsonxClient::from_env()?;

    let rules = client.document_business_rules(SAMPLE_COBOL).await?;
    println!("Business rules:");
    println!("{}", rules);

    Ok(())
}

// Made with Bob

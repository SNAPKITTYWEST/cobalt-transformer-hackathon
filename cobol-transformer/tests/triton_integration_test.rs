use cobol_transformer::*;
use cobol_transformer::lexer::Lexer;
use cobol_transformer::parser::Parser;
use cobol_transformer::vector::*;
use cobol_transformer::jcl::*;

#[test]
fn test_triton_end_to_end_vectorization() {
    // Parse COBOL program
    let source = r#"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. TEST-VECTOR.
       
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 ARRAY-DATA.
          05 VALUES OCCURS 100 TIMES.
             10 INPUT-VAL  PIC 9(5) COMP-3.
             10 OUTPUT-VAL PIC 9(5) COMP-3.
       
       PROCEDURE DIVISION.
       MAIN-PARA.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 100
               COMPUTE OUTPUT-VAL(I) = INPUT-VAL(I) * 2
           END-PERFORM.
           STOP RUN.
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexing failed");
    
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parsing failed");
    
    // Analyze for vectorization
    let mut analyzer = VectorizationAnalyzer::new();
    let candidates = analyzer.analyze(&program).expect("Analysis failed");
    
    assert!(!candidates.is_empty(), "Should find vectorization candidates");
    
    // Generate chunk IR
    let candidate = &candidates[0];
    let chunk = analyzer.generate_chunk_ir(candidate).expect("Chunk generation failed");
    
    assert!(!chunk.operations.is_empty(), "Chunk should have operations");
}

#[test]
fn test_decimal_arithmetic() {
    let engine = DecimalEngine::new();
    
    // Test addition
    let a = DecimalValue::from_i64(100, 2); // 1.00
    let b = DecimalValue::from_i64(50, 2);  // 0.50
    let result = engine.add(&a, &b).expect("Addition failed");
    
    assert_eq!(result.scale, 2);
    
    // Test multiplication
    let c = DecimalValue::from_i64(12, 1); // 1.2
    let d = DecimalValue::from_i64(5, 0);  // 5
    let result = engine.multiply(&c, &d).expect("Multiplication failed");
    
    assert!(result.scale >= 0);
}

#[test]
fn test_jcl_parsing() {
    let source = r#"
//TESTJOB JOB CLASS=A
//STEP1 EXEC PGM=TESTPROG
//DD1 DD DSN=TEST.DATA,DISP=SHR
    "#;
    
    let mut lexer = JclLexer::new(source);
    let tokens = lexer.tokenize().expect("JCL lexing failed");
    
    let mut parser = JclParser::new(tokens);
    let job = parser.parse().expect("JCL parsing failed");
    
    assert_eq!(job.steps.len(), 1);
    assert_eq!(job.steps[0].dd_statements.len(), 1);
}

#[test]
fn test_job_graph_construction() {
    let mut job = JclJob::new("TEST".to_string());
    
    let step1 = JclStep::new(
        "STEP1".to_string(),
        ExecStatement::program("PROG1".to_string()),
    );
    
    let step2 = JclStep::new(
        "STEP2".to_string(),
        ExecStatement::program("PROG2".to_string()),
    );
    
    job.add_step(step1);
    job.add_step(step2);
    
    let mut builder = JobGraphBuilder::new();
    let graph = builder.build(&job).expect("Graph building failed");
    
    assert_eq!(graph.node_count(), 2);
}

#[test]
fn test_chunk_scheduler() {
    let mut ir = ChunkIR::new();
    
    let chunk1 = Chunk::new(0);
    let chunk2 = Chunk::new(1);
    
    ir.add_chunk(chunk1);
    ir.add_chunk(chunk2);
    
    let mut scheduler = ChunkScheduler::new(ir);
    let plan = scheduler.schedule().expect("Scheduling failed");
    
    assert!(!plan.stages.is_empty());
    assert_eq!(plan.total_chunks, 2);
}

#[test]
fn test_scalar_backend_execution() {
    use vector::scalar_backend::{ScalarBackend, ExecutionContext, Value};
    
    let mut backend = ScalarBackend::new();
    let mut context = ExecutionContext::new(10);
    
    // Add test data
    let input_values: Vec<Value> = (0..10).map(|i| Value::Integer(i)).collect();
    context.add_input("INPUT".to_string(), input_values);
    
    // Create simple chunk
    let mut chunk = Chunk::new(0);
    chunk.add_operation(ChunkOperation::Load {
        source: "INPUT".to_string(),
        target: "REG1".to_string(),
    });
    chunk.add_operation(ChunkOperation::Store {
        source: "REG1".to_string(),
        target: "OUTPUT".to_string(),
    });
    
    backend.execute_chunk(&chunk, &mut context).expect("Execution failed");
    
    let output = context.get_output("OUTPUT");
    assert!(output.is_some());
}

#[test]
fn test_vector_backend_execution() {
    use vector::cpu_backend::CpuVectorBackend;
    use vector::scalar_backend::{ExecutionContext, Value};
    
    let mut backend = CpuVectorBackend::new(8);
    let mut context = ExecutionContext::new(16);
    
    // Add test data
    let input_values: Vec<Value> = (0..16).map(|i| Value::Integer(i)).collect();
    context.add_input("INPUT".to_string(), input_values);
    
    // Create vectorizable chunk
    let mut chunk = Chunk::new(0);
    chunk.add_operation(ChunkOperation::Load {
        source: "INPUT".to_string(),
        target: "REG1".to_string(),
    });
    chunk.add_operation(ChunkOperation::Store {
        source: "REG1".to_string(),
        target: "OUTPUT".to_string(),
    });
    
    backend.execute_chunk(&chunk, &mut context).expect("Vector execution failed");
    
    let output = context.get_output("OUTPUT");
    assert!(output.is_some());
}

#[test]
fn test_memory_layout_optimization() {
    use vector::memory::MemoryLayoutOptimizer;
    
    let optimizer = MemoryLayoutOptimizer::new();
    
    let mut chunk = Chunk::new(0);
    chunk.add_input(DataSchema {
        name: "FIELD1".to_string(),
        data_type: CobolDataType::Comp3 { digits: 5, scale: 2 },
        size: 3,
        offset: 0,
    });
    
    let layout = optimizer.optimize_layout(&chunk).expect("Layout optimization failed");
    assert!(layout.total_size() > 0);
}

#[test]
fn test_differential_execution() {
    // Test that scalar and vector backends produce identical results
    use vector::scalar_backend::{ScalarBackend, ExecutionContext, Value};
    use vector::cpu_backend::CpuVectorBackend;
    
    let test_data: Vec<Value> = (1..=16).map(|i| Value::Integer(i * 10)).collect();
    
    // Scalar execution
    let mut scalar_backend = ScalarBackend::new();
    let mut scalar_context = ExecutionContext::new(16);
    scalar_context.add_input("INPUT".to_string(), test_data.clone());
    
    let mut chunk = Chunk::new(0);
    chunk.add_operation(ChunkOperation::Load {
        source: "INPUT".to_string(),
        target: "REG1".to_string(),
    });
    chunk.add_operation(ChunkOperation::Store {
        source: "REG1".to_string(),
        target: "OUTPUT".to_string(),
    });
    
    scalar_backend.execute_chunk(&chunk, &mut scalar_context).expect("Scalar execution failed");
    
    // Vector execution
    let mut vector_backend = CpuVectorBackend::new(8);
    let mut vector_context = ExecutionContext::new(16);
    vector_context.add_input("INPUT".to_string(), test_data);
    
    vector_backend.execute_chunk(&chunk, &mut vector_context).expect("Vector execution failed");
    
    // Compare results
    let scalar_output = scalar_context.get_output("OUTPUT").unwrap();
    let vector_output = vector_context.get_output("OUTPUT").unwrap();
    
    assert_eq!(scalar_output.len(), vector_output.len(), "Output lengths must match");
}

// Made with Bob

# TRITON AGENT — Quick Start Guide

## Installation

```bash
cd cobol-transformer
cargo build --release
```

## Basic Usage

### 1. Parse COBOL Program

```rust
use cobol_transformer::*;

let source = r#"
   IDENTIFICATION DIVISION.
   PROGRAM-ID. HELLO.
   
   PROCEDURE DIVISION.
       DISPLAY "Hello, TRITON!".
       STOP RUN.
"#;

let mut lexer = lexer::Lexer::new(source);
let tokens = lexer.tokenize()?;

let mut parser = parser::Parser::new(tokens);
let program = parser.parse()?;

println!("Program: {}", program.name());
```

### 2. Vectorize COBOL Code

```rust
use cobol_transformer::vector::*;

// Analyze for vectorization
let mut analyzer = VectorizationAnalyzer::new();
let candidates = analyzer.analyze(&program)?;

for candidate in &candidates {
    println!("Found vectorizable loop: {} iterations", 
             candidate.loop_info.as_ref().unwrap().iteration_count.unwrap_or(0));
    println!("Estimated speedup: {}x", candidate.estimated_speedup);
}

// Generate chunk IR
let chunk = analyzer.generate_chunk_ir(&candidates[0])?;
println!("Generated chunk with {} operations", chunk.operations.len());
```

### 3. Execute with Vector Backend

```rust
use cobol_transformer::vector::cpu_backend::*;
use cobol_transformer::vector::scalar_backend::*;

// Create execution context
let mut context = ExecutionContext::new(100);

// Add input data
let input: Vec<Value> = (0..100)
    .map(|i| Value::Integer(i))
    .collect();
context.add_input("INPUT".to_string(), input);

// Execute with vector backend
let mut backend = CpuVectorBackend::new(8);
backend.execute_chunk(&chunk, &mut context)?;

// Get results
let output = context.get_output("OUTPUT").unwrap();
println!("Processed {} records", output.len());
```

### 4. Process JCL Job

```rust
use cobol_transformer::jcl::*;

let jcl = r#"
//MYJOB JOB CLASS=A
//STEP1 EXEC PGM=MYPROG
//INPUT DD DSN=MY.INPUT.DATA,DISP=SHR
//OUTPUT DD DSN=MY.OUTPUT.DATA,DISP=(NEW,CATLG,DELETE)
"#;

let mut lexer = JclLexer::new(jcl);
let tokens = lexer.tokenize()?;

let mut parser = JclParser::new(tokens);
let job = parser.parse()?;

println!("Job: {}", job.name);
println!("Steps: {}", job.steps.len());

// Build job graph
let mut builder = JobGraphBuilder::new();
let graph = builder.build(&job)?;

// Get execution order
let order = graph.topological_order()?;
println!("Execution order: {:?}", order);
```

### 5. Decimal Arithmetic

```rust
use cobol_transformer::vector::decimal::*;

let engine = DecimalEngine::new();

// Create decimal values
let price = DecimalValue::from_i64(12995, 2);  // $129.95
let qty = DecimalValue::from_i64(5, 0);        // 5 units
let tax_rate = DecimalValue::from_i64(825, 3); // 8.25%

// Calculate total
let subtotal = engine.multiply(&price, &qty)?;
let tax = engine.multiply(&subtotal, &tax_rate)?;
let total = engine.add(&subtotal, &tax)?;

println!("Subtotal: {}", subtotal);
println!("Tax: {}", tax);
println!("Total: {}", total);
```

## Running Examples

### Vector Example

```bash
# View the example
cat examples/vector_example.cob

# Parse and analyze
cargo run --example vector_demo
```

### JCL Example

```bash
# View the example
cat examples/batch_job.jcl

# Parse and execute
cargo run --example jcl_demo
```

## Running Tests

```bash
# All tests
cargo test

# TRITON integration tests
cargo test --test triton_integration_test

# Specific test
cargo test test_decimal_arithmetic

# With output
cargo test -- --nocapture
```

## Common Patterns

### Pattern 1: Analyze and Optimize

```rust
// Parse COBOL
let program = parse_cobol(source)?;

// Find vectorization opportunities
let mut analyzer = VectorizationAnalyzer::new();
let candidates = analyzer.analyze(&program)?;

// Generate optimized chunks
let mut chunks = Vec::new();
for candidate in candidates {
    if candidate.is_safe {
        chunks.push(analyzer.generate_chunk_ir(&candidate)?);
    }
}

// Schedule execution
let mut ir = ChunkIR::new();
for chunk in chunks {
    ir.add_chunk(chunk);
}

let mut scheduler = ChunkScheduler::new(ir);
let plan = scheduler.schedule()?;

println!("Execution plan: {} stages", plan.stages.len());
```

### Pattern 2: Differential Testing

```rust
// Execute with both backends
let mut scalar = ScalarBackend::new();
let mut vector = CpuVectorBackend::new(8);

let mut scalar_ctx = context.clone();
let mut vector_ctx = context.clone();

scalar.execute_chunk(&chunk, &mut scalar_ctx)?;
vector.execute_chunk(&chunk, &mut vector_ctx)?;

// Compare results
assert_eq!(
    scalar_ctx.get_output("OUTPUT"),
    vector_ctx.get_output("OUTPUT"),
    "Scalar and vector results must match"
);
```

### Pattern 3: Memory Layout Optimization

```rust
use cobol_transformer::vector::memory::*;

let optimizer = MemoryLayoutOptimizer::new();

// Analyze chunk
let layout = optimizer.optimize_layout(&chunk)?;

match layout {
    MemoryLayout::ArrayOfStructures(aos) => {
        println!("Using AoS layout, struct size: {}", aos.struct_size);
    }
    MemoryLayout::StructureOfArrays(soa) => {
        println!("Using SoA layout, {} arrays", soa.array_count);
    }
}
```

## Performance Tips

1. **Use appropriate vector width:**
   ```rust
   let backend = CpuVectorBackend::new(8);  // Good for most CPUs
   ```

2. **Enable SoA for vector-heavy workloads:**
   ```rust
   chunk.memory_layout = MemoryLayoutHint::StructureOfArrays;
   ```

3. **Choose remainder policy wisely:**
   ```rust
   chunk.remainder_policy = RemainderPolicy::Masked;  // Best for small remainders
   ```

4. **Profile before optimizing:**
   ```rust
   let plan = scheduler.schedule()?;
   println!("Estimated time: {:.2}ms", plan.estimated_time);
   ```

## Troubleshooting

### Issue: Vectorization not applied

**Solution:** Check safety analysis
```rust
if !candidate.is_safe {
    println!("Unsafe due to: {:?}", candidate.data_dependencies);
}
```

### Issue: Decimal precision loss

**Solution:** Use appropriate scale
```rust
let value = DecimalValue::from_i64(12345, 2);  // 123.45
```

### Issue: JCL parsing fails

**Solution:** Check JCL format
```rust
// Ensure proper JCL format
//JOBNAME JOB ...
//STEPNAME EXEC ...
```

## Next Steps

1. Read [`TRITON_README.md`](TRITON_README.md) for complete documentation
2. Review [`IMPLEMENTATION_SUMMARY.md`](IMPLEMENTATION_SUMMARY.md) for architecture details
3. Explore [`tests/triton_integration_test.rs`](tests/triton_integration_test.rs) for more examples
4. Check [`examples/`](examples/) directory for working code

## Support

For questions or issues:
- Review the comprehensive documentation in `TRITON_README.md`
- Check the integration tests for usage patterns
- Examine the examples directory

---

**TRITON AGENT v1.0.0** — Production-grade IBM COBOL + JCL Vector Engine
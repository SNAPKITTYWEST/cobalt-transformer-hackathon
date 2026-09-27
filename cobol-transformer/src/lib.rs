pub mod lexer;
pub mod parser;
pub mod ast;
pub mod preprocessor;
pub mod symbol_table;
pub mod type_system;
pub mod cfg;
pub mod dataflow;
pub mod transform;
pub mod codegen;
pub mod diagnostics;
pub mod source_map;
pub mod format;

// TRITON Vector Engine
pub mod jcl;
pub mod vector;

// IBM Cloud Integration
pub mod ibm;

// Re-exports for convenience
pub use jcl::{JclLexer, JclParser, JclJob, JobGraph};
pub use vector::{
    ChunkIR, Chunk, VectorizationAnalyzer, ChunkScheduler,
    ScalarBackend, CpuVectorBackend, DecimalEngine, DecimalValue
};

// Made with Bob - TRITON AGENT v1.0.0

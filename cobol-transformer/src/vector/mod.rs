// Vector Engine Module
// Implements vectorization analysis and execution for COBOL operations

pub mod chunk_ir;
pub mod analysis;
pub mod scheduler;
pub mod scalar_backend;
pub mod cpu_backend;
pub mod decimal;
pub mod memory;

pub use chunk_ir::{ChunkIR, Chunk, ChunkOperation, ChunkDependency};
pub use analysis::{VectorizationAnalyzer, VectorizationCandidate};
pub use scheduler::{ChunkScheduler, ExecutionPlan};
pub use scalar_backend::ScalarBackend;
pub use cpu_backend::CpuVectorBackend;
pub use decimal::{DecimalEngine, DecimalValue, DecimalOperation};
pub use memory::{MemoryLayout, ArrayLayout};

// Made with Bob

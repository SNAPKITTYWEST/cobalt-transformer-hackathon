// Vector Chunk Intermediate Representation
// Represents vectorizable operations as independent execution chunks

use crate::ast::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ChunkIR {
    pub chunks: Vec<Chunk>,
    pub dependencies: HashMap<usize, Vec<usize>>,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: usize,
    pub input_schema: Vec<DataSchema>,
    pub output_schema: Vec<DataSchema>,
    pub operations: Vec<ChunkOperation>,
    pub data_types: Vec<CobolDataType>,
    pub control_flow: ChunkControlFlow,
    pub side_effects: Vec<SideEffect>,
    pub memory_layout: MemoryLayoutHint,
    pub vector_width: usize,
    pub remainder_policy: RemainderPolicy,
    pub error_policy: ErrorPolicy,
    pub source_location: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DataSchema {
    pub name: String,
    pub data_type: CobolDataType,
    pub size: usize,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub enum ChunkOperation {
    Load { source: String, target: String },
    Store { source: String, target: String },
    Add { left: String, right: String, result: String },
    Subtract { left: String, right: String, result: String },
    Multiply { left: String, right: String, result: String },
    Divide { left: String, right: String, result: String },
    Move { source: String, target: String },
    Compare { left: String, right: String, operator: CompareOp },
    DecimalConvert { source: String, target: String, from_type: CobolDataType, to_type: CobolDataType },
    Validate { source: String, rules: Vec<ValidationRule> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareOp {
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
}

#[derive(Debug, Clone)]
pub enum ValidationRule {
    Numeric,
    Range { min: i64, max: i64 },
    Pattern { regex: String },
}

#[derive(Debug, Clone)]
pub enum CobolDataType {
    Display { size: usize, scale: i32 },
    PackedDecimal { digits: usize, scale: i32 },
    Binary { bytes: usize, signed: bool },
    Comp { bytes: usize },
    Comp1,
    Comp2,
    Comp3 { digits: usize, scale: i32 },
    Comp4 { bytes: usize },
    Comp5 { bytes: usize },
    Alphanumeric { size: usize },
}

#[derive(Debug, Clone)]
pub enum ChunkControlFlow {
    Linear,
    Conditional { condition: String },
    Loop { iterations: Option<usize> },
}

#[derive(Debug, Clone)]
pub enum SideEffect {
    FileIO { operation: String, file: String },
    Display { message: String },
    ExternalCall { program: String },
}

#[derive(Debug, Clone)]
pub enum MemoryLayoutHint {
    ArrayOfStructures,
    StructureOfArrays,
    PackedDecimalArray,
    BinaryArray,
    DisplayArray,
}

#[derive(Debug, Clone)]
pub enum RemainderPolicy {
    Scalar,
    Masked,
    Padded,
}

#[derive(Debug, Clone)]
pub enum ErrorPolicy {
    Abort,
    Continue,
    Fallback,
}

#[derive(Debug, Clone)]
pub struct ChunkDependency {
    pub from_chunk: usize,
    pub to_chunk: usize,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone)]
pub enum DependencyType {
    DataFlow { variable: String },
    ControlFlow,
    SideEffect,
}

impl ChunkIR {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            dependencies: HashMap::new(),
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) -> usize {
        let id = chunk.id;
        self.chunks.push(chunk);
        id
    }

    pub fn add_dependency(&mut self, from: usize, to: usize) {
        self.dependencies.entry(to).or_insert_with(Vec::new).push(from);
    }

    pub fn get_chunk(&self, id: usize) -> Option<&Chunk> {
        self.chunks.iter().find(|c| c.id == id)
    }

    pub fn get_dependencies(&self, chunk_id: usize) -> Vec<usize> {
        self.dependencies.get(&chunk_id).cloned().unwrap_or_default()
    }

    pub fn topological_order(&self) -> Vec<usize> {
        let mut visited = HashSet::new();
        let mut order = Vec::new();
        
        for chunk in &self.chunks {
            if !visited.contains(&chunk.id) {
                self.dfs_visit(chunk.id, &mut visited, &mut order);
            }
        }
        
        order.reverse();
        order
    }

    fn dfs_visit(&self, chunk_id: usize, visited: &mut HashSet<usize>, order: &mut Vec<usize>) {
        visited.insert(chunk_id);
        
        if let Some(deps) = self.dependencies.get(&chunk_id) {
            for &dep in deps {
                if !visited.contains(&dep) {
                    self.dfs_visit(dep, visited, order);
                }
            }
        }
        
        order.push(chunk_id);
    }

    pub fn is_vectorizable(&self, chunk_id: usize) -> bool {
        if let Some(chunk) = self.get_chunk(chunk_id) {
            // Check if chunk has no side effects that prevent vectorization
            for effect in &chunk.side_effects {
                match effect {
                    SideEffect::FileIO { .. } => return false,
                    SideEffect::Display { .. } => return false,
                    SideEffect::ExternalCall { .. } => return false,
                }
            }
            
            // Check if control flow is suitable for vectorization
            matches!(chunk.control_flow, ChunkControlFlow::Linear | ChunkControlFlow::Loop { .. })
        } else {
            false
        }
    }
}

impl Chunk {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            input_schema: Vec::new(),
            output_schema: Vec::new(),
            operations: Vec::new(),
            data_types: Vec::new(),
            control_flow: ChunkControlFlow::Linear,
            side_effects: Vec::new(),
            memory_layout: MemoryLayoutHint::ArrayOfStructures,
            vector_width: 8,
            remainder_policy: RemainderPolicy::Scalar,
            error_policy: ErrorPolicy::Fallback,
            source_location: None,
        }
    }

    pub fn add_operation(&mut self, op: ChunkOperation) {
        self.operations.push(op);
    }

    pub fn add_input(&mut self, schema: DataSchema) {
        self.input_schema.push(schema);
    }

    pub fn add_output(&mut self, schema: DataSchema) {
        self.output_schema.push(schema);
    }

    pub fn has_side_effects(&self) -> bool {
        !self.side_effects.is_empty()
    }

    pub fn is_pure(&self) -> bool {
        !self.has_side_effects()
    }
}

impl CobolDataType {
    pub fn size_in_bytes(&self) -> usize {
        match self {
            CobolDataType::Display { size, .. } => *size,
            CobolDataType::PackedDecimal { digits, .. } => (digits + 1) / 2 + 1,
            CobolDataType::Binary { bytes, .. } => *bytes,
            CobolDataType::Comp { bytes } => *bytes,
            CobolDataType::Comp1 => 4,
            CobolDataType::Comp2 => 8,
            CobolDataType::Comp3 { digits, .. } => (digits + 1) / 2 + 1,
            CobolDataType::Comp4 { bytes } => *bytes,
            CobolDataType::Comp5 { bytes } => *bytes,
            CobolDataType::Alphanumeric { size } => *size,
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            CobolDataType::Display { .. }
                | CobolDataType::PackedDecimal { .. }
                | CobolDataType::Binary { .. }
                | CobolDataType::Comp { .. }
                | CobolDataType::Comp1
                | CobolDataType::Comp2
                | CobolDataType::Comp3 { .. }
                | CobolDataType::Comp4 { .. }
                | CobolDataType::Comp5 { .. }
        )
    }

    pub fn requires_decimal_semantics(&self) -> bool {
        matches!(
            self,
            CobolDataType::Display { .. }
                | CobolDataType::PackedDecimal { .. }
                | CobolDataType::Comp3 { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_creation() {
        let mut chunk = Chunk::new(0);
        chunk.add_operation(ChunkOperation::Load {
            source: "INPUT".to_string(),
            target: "REG1".to_string(),
        });
        
        assert_eq!(chunk.operations.len(), 1);
        assert!(chunk.is_pure());
    }

    #[test]
    fn test_chunk_ir_dependencies() {
        let mut ir = ChunkIR::new();
        
        let chunk1 = Chunk::new(0);
        let chunk2 = Chunk::new(1);
        
        ir.add_chunk(chunk1);
        ir.add_chunk(chunk2);
        ir.add_dependency(0, 1);
        
        let deps = ir.get_dependencies(1);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], 0);
    }

    #[test]
    fn test_topological_order() {
        let mut ir = ChunkIR::new();
        
        ir.add_chunk(Chunk::new(0));
        ir.add_chunk(Chunk::new(1));
        ir.add_chunk(Chunk::new(2));
        
        ir.add_dependency(0, 1);
        ir.add_dependency(1, 2);
        
        let order = ir.topological_order();
        assert_eq!(order, vec![0, 1, 2]);
    }
}

// Made with Bob

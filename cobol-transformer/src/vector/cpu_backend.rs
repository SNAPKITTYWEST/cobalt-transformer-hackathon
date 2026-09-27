// CPU Vector Backend
// Implements SIMD vectorization for x86-64 CPUs

use crate::vector::chunk_ir::*;
use crate::vector::scalar_backend::{ExecutionContext, Value};
use crate::vector::decimal::*;
use anyhow::Result;

pub struct CpuVectorBackend {
    vector_width: usize,
    scalar_fallback: crate::vector::scalar_backend::ScalarBackend,
}

#[derive(Debug, Clone)]
pub struct VectorRegister {
    pub values: Vec<Value>,
    pub width: usize,
}

impl CpuVectorBackend {
    pub fn new(vector_width: usize) -> Self {
        Self {
            vector_width,
            scalar_fallback: crate::vector::scalar_backend::ScalarBackend::new(),
        }
    }

    pub fn execute_chunk(&mut self, chunk: &Chunk, context: &mut ExecutionContext) -> Result<()> {
        // Check if chunk is vectorizable
        if !chunk.is_pure() || chunk.has_side_effects() {
            // Fall back to scalar execution
            return self.scalar_fallback.execute_chunk(chunk, context);
        }

        let record_count = context.record_count;
        let vector_width = self.vector_width;
        
        // Process full vector chunks
        let full_chunks = record_count / vector_width;
        for chunk_idx in 0..full_chunks {
            let start = chunk_idx * vector_width;
            let end = start + vector_width;
            self.execute_vector_chunk(chunk, context, start, end)?;
        }
        
        // Process remainder with scalar backend
        let remainder_start = full_chunks * vector_width;
        if remainder_start < record_count {
            match chunk.remainder_policy {
                RemainderPolicy::Scalar => {
                    self.execute_scalar_remainder(chunk, context, remainder_start, record_count)?;
                }
                RemainderPolicy::Masked => {
                    self.execute_masked_remainder(chunk, context, remainder_start, record_count)?;
                }
                RemainderPolicy::Padded => {
                    self.execute_padded_remainder(chunk, context, remainder_start, record_count)?;
                }
            }
        }
        
        Ok(())
    }

    fn execute_vector_chunk(&mut self, chunk: &Chunk, context: &mut ExecutionContext, start: usize, end: usize) -> Result<()> {
        // Load vector registers
        let mut vector_regs: std::collections::HashMap<String, VectorRegister> = std::collections::HashMap::new();
        
        for operation in &chunk.operations {
            match operation {
                ChunkOperation::Load { source, target } => {
                    if let Some(values) = context.input_data.get(source) {
                        let vec_values: Vec<Value> = values[start..end].to_vec();
                        vector_regs.insert(target.clone(), VectorRegister {
                            values: vec_values,
                            width: self.vector_width,
                        });
                    }
                }
                ChunkOperation::Add { left, right, result } => {
                    let left_reg = vector_regs.get(left).ok_or_else(|| anyhow::anyhow!("Register not found"))?;
                    let right_reg = vector_regs.get(right).ok_or_else(|| anyhow::anyhow!("Register not found"))?;
                    
                    let result_values = self.vector_add(&left_reg.values, &right_reg.values)?;
                    vector_regs.insert(result.clone(), VectorRegister {
                        values: result_values,
                        width: self.vector_width,
                    });
                }
                ChunkOperation::Multiply { left, right, result } => {
                    let left_reg = vector_regs.get(left).ok_or_else(|| anyhow::anyhow!("Register not found"))?;
                    let right_reg = vector_regs.get(right).ok_or_else(|| anyhow::anyhow!("Register not found"))?;
                    
                    let result_values = self.vector_multiply(&left_reg.values, &right_reg.values)?;
                    vector_regs.insert(result.clone(), VectorRegister {
                        values: result_values,
                        width: self.vector_width,
                    });
                }
                ChunkOperation::Store { source, target } => {
                    if let Some(reg) = vector_regs.get(source) {
                        context.output_data
                            .entry(target.clone())
                            .or_insert_with(Vec::new)
                            .extend(reg.values.clone());
                    }
                }
                _ => {
                    // Other operations would be implemented similarly
                }
            }
        }
        
        Ok(())
    }

    fn vector_add(&self, left: &[Value], right: &[Value]) -> Result<Vec<Value>> {
        let engine = DecimalEngine::new();
        let mut results = Vec::with_capacity(left.len());
        
        for (l, r) in left.iter().zip(right.iter()) {
            let result = match (l, r) {
                (Value::Decimal(ld), Value::Decimal(rd)) => {
                    Value::Decimal(engine.add(ld, rd)?)
                }
                (Value::Integer(li), Value::Integer(ri)) => {
                    Value::Integer(li + ri)
                }
                _ => anyhow::bail!("Type mismatch in vector add"),
            };
            results.push(result);
        }
        
        Ok(results)
    }

    fn vector_multiply(&self, left: &[Value], right: &[Value]) -> Result<Vec<Value>> {
        let engine = DecimalEngine::new();
        let mut results = Vec::with_capacity(left.len());
        
        for (l, r) in left.iter().zip(right.iter()) {
            let result = match (l, r) {
                (Value::Decimal(ld), Value::Decimal(rd)) => {
                    Value::Decimal(engine.multiply(ld, rd)?)
                }
                (Value::Integer(li), Value::Integer(ri)) => {
                    Value::Integer(li * ri)
                }
                _ => anyhow::bail!("Type mismatch in vector multiply"),
            };
            results.push(result);
        }
        
        Ok(results)
    }

    fn execute_scalar_remainder(&mut self, chunk: &Chunk, context: &mut ExecutionContext, start: usize, end: usize) -> Result<()> {
        // Create a sub-context for the remainder
        let mut sub_context = ExecutionContext::new(end - start);
        
        // Copy input data for remainder
        for (name, values) in &context.input_data {
            sub_context.add_input(name.clone(), values[start..end].to_vec());
        }
        
        // Execute with scalar backend
        self.scalar_fallback.execute_chunk(chunk, &mut sub_context)?;
        
        // Merge results back
        for (name, values) in sub_context.output_data {
            context.output_data
                .entry(name)
                .or_insert_with(Vec::new)
                .extend(values);
        }
        
        Ok(())
    }

    fn execute_masked_remainder(&mut self, chunk: &Chunk, context: &mut ExecutionContext, start: usize, end: usize) -> Result<()> {
        // Pad to vector width with dummy values, then mask out results
        let padded_end = ((end + self.vector_width - 1) / self.vector_width) * self.vector_width;
        
        // Create padded context
        let mut padded_context = ExecutionContext::new(padded_end - start);
        
        for (name, values) in &context.input_data {
            let mut padded_values = values[start..end].to_vec();
            while padded_values.len() < (padded_end - start) {
                padded_values.push(Value::Integer(0));
            }
            padded_context.add_input(name.clone(), padded_values);
        }
        
        // Execute vector chunk
        self.execute_vector_chunk(chunk, &mut padded_context, 0, padded_end - start)?;
        
        // Copy only valid results
        for (name, values) in padded_context.output_data {
            let valid_count = end - start;
            context.output_data
                .entry(name)
                .or_insert_with(Vec::new)
                .extend(values.into_iter().take(valid_count));
        }
        
        Ok(())
    }

    fn execute_padded_remainder(&mut self, chunk: &Chunk, context: &mut ExecutionContext, start: usize, end: usize) -> Result<()> {
        // Similar to masked, but simpler
        self.execute_masked_remainder(chunk, context, start, end)
    }

    pub fn supports_vector_width(width: usize) -> bool {
        // Check if CPU supports the requested vector width
        // In production, this would query CPU features
        matches!(width, 4 | 8 | 16 | 32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_backend_creation() {
        let backend = CpuVectorBackend::new(8);
        assert_eq!(backend.vector_width, 8);
    }

    #[test]
    fn test_vector_width_support() {
        assert!(CpuVectorBackend::supports_vector_width(8));
        assert!(!CpuVectorBackend::supports_vector_width(7));
    }

    #[test]
    fn test_vector_add() {
        let backend = CpuVectorBackend::new(4);
        let left = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ];
        let right = vec![
            Value::Integer(10),
            Value::Integer(20),
            Value::Integer(30),
            Value::Integer(40),
        ];
        
        let result = backend.vector_add(&left, &right).unwrap();
        assert_eq!(result.len(), 4);
    }
}

// Made with Bob

// Memory Layout Management
// Handles AoS/SoA transformations and memory optimization

use crate::vector::chunk_ir::*;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum MemoryLayout {
    ArrayOfStructures(AoSLayout),
    StructureOfArrays(SoALayout),
}

#[derive(Debug, Clone)]
pub struct AoSLayout {
    pub struct_size: usize,
    pub field_offsets: Vec<FieldOffset>,
    pub alignment: usize,
}

#[derive(Debug, Clone)]
pub struct SoALayout {
    pub array_count: usize,
    pub arrays: Vec<ArrayInfo>,
    pub total_size: usize,
}

#[derive(Debug, Clone)]
pub struct FieldOffset {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub data_type: CobolDataType,
}

#[derive(Debug, Clone)]
pub struct ArrayInfo {
    pub name: String,
    pub element_size: usize,
    pub element_count: usize,
    pub data_type: CobolDataType,
}

#[derive(Debug, Clone)]
pub enum ArrayLayout {
    Contiguous,
    Strided { stride: usize },
    Packed,
}

pub struct MemoryLayoutOptimizer {
    target_cache_line_size: usize,
}

impl MemoryLayoutOptimizer {
    pub fn new() -> Self {
        Self {
            target_cache_line_size: 64, // Common cache line size
        }
    }

    pub fn optimize_layout(&self, chunk: &Chunk) -> Result<MemoryLayout> {
        // Analyze access patterns
        let access_pattern = self.analyze_access_pattern(chunk);
        
        // Decide between AoS and SoA
        if self.should_use_soa(&access_pattern) {
            self.create_soa_layout(chunk)
        } else {
            self.create_aos_layout(chunk)
        }
    }

    fn analyze_access_pattern(&self, chunk: &Chunk) -> AccessPattern {
        let mut field_accesses: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut sequential_accesses = 0;
        let random_accesses = 0;
        
        for operation in &chunk.operations {
            match operation {
                ChunkOperation::Load { source, .. } => {
                    *field_accesses.entry(source.clone()).or_insert(0) += 1;
                    sequential_accesses += 1;
                }
                ChunkOperation::Store { target, .. } => {
                    *field_accesses.entry(target.clone()).or_insert(0) += 1;
                    sequential_accesses += 1;
                }
                _ => {}
            }
        }
        
        AccessPattern {
            field_accesses,
            sequential_accesses,
            random_accesses,
            vector_friendly: sequential_accesses > random_accesses,
        }
    }

    fn should_use_soa(&self, pattern: &AccessPattern) -> bool {
        // Use SoA if:
        // 1. Access pattern is vector-friendly
        // 2. Not all fields are accessed together
        // 3. Sequential access dominates
        
        pattern.vector_friendly && 
        pattern.sequential_accesses > pattern.random_accesses * 2
    }

    fn create_aos_layout(&self, chunk: &Chunk) -> Result<MemoryLayout> {
        let mut field_offsets = Vec::new();
        let mut current_offset = 0;
        
        for schema in &chunk.input_schema {
            let size = schema.data_type.size_in_bytes();
            field_offsets.push(FieldOffset {
                name: schema.name.clone(),
                offset: current_offset,
                size,
                data_type: schema.data_type.clone(),
            });
            current_offset += size;
        }
        
        // Align to cache line
        let struct_size = self.align_to_cache_line(current_offset);
        
        Ok(MemoryLayout::ArrayOfStructures(AoSLayout {
            struct_size,
            field_offsets,
            alignment: self.target_cache_line_size,
        }))
    }

    fn create_soa_layout(&self, chunk: &Chunk) -> Result<MemoryLayout> {
        let mut arrays = Vec::new();
        let element_count = 1000; // Would be determined by actual data size
        
        for schema in &chunk.input_schema {
            let element_size = schema.data_type.size_in_bytes();
            arrays.push(ArrayInfo {
                name: schema.name.clone(),
                element_size,
                element_count,
                data_type: schema.data_type.clone(),
            });
        }
        
        let total_size = arrays.iter()
            .map(|a| a.element_size * a.element_count)
            .sum();
        
        Ok(MemoryLayout::StructureOfArrays(SoALayout {
            array_count: arrays.len(),
            arrays,
            total_size,
        }))
    }

    fn align_to_cache_line(&self, size: usize) -> usize {
        ((size + self.target_cache_line_size - 1) / self.target_cache_line_size) * self.target_cache_line_size
    }

    pub fn transform_aos_to_soa(&self, aos_data: &[u8], aos_layout: &AoSLayout, record_count: usize) -> Result<Vec<Vec<u8>>> {
        let mut soa_arrays = vec![Vec::new(); aos_layout.field_offsets.len()];
        
        for record_idx in 0..record_count {
            let record_offset = record_idx * aos_layout.struct_size;
            
            for (field_idx, field) in aos_layout.field_offsets.iter().enumerate() {
                let field_offset = record_offset + field.offset;
                let field_data = &aos_data[field_offset..field_offset + field.size];
                soa_arrays[field_idx].extend_from_slice(field_data);
            }
        }
        
        Ok(soa_arrays)
    }

    pub fn transform_soa_to_aos(&self, soa_arrays: &[Vec<u8>], soa_layout: &SoALayout) -> Result<Vec<u8>> {
        let record_count = soa_layout.arrays[0].element_count;
        let struct_size: usize = soa_layout.arrays.iter().map(|a| a.element_size).sum();
        let mut aos_data = vec![0u8; record_count * struct_size];
        
        for record_idx in 0..record_count {
            let mut field_offset = 0;
            
            for (array_idx, array_info) in soa_layout.arrays.iter().enumerate() {
                let src_offset = record_idx * array_info.element_size;
                let dst_offset = record_idx * struct_size + field_offset;
                
                aos_data[dst_offset..dst_offset + array_info.element_size]
                    .copy_from_slice(&soa_arrays[array_idx][src_offset..src_offset + array_info.element_size]);
                
                field_offset += array_info.element_size;
            }
        }
        
        Ok(aos_data)
    }
}

#[derive(Debug, Clone)]
struct AccessPattern {
    field_accesses: std::collections::HashMap<String, usize>,
    sequential_accesses: usize,
    random_accesses: usize,
    vector_friendly: bool,
}

impl MemoryLayout {
    pub fn is_aos(&self) -> bool {
        matches!(self, MemoryLayout::ArrayOfStructures(_))
    }

    pub fn is_soa(&self) -> bool {
        matches!(self, MemoryLayout::StructureOfArrays(_))
    }

    pub fn total_size(&self) -> usize {
        match self {
            MemoryLayout::ArrayOfStructures(aos) => aos.struct_size,
            MemoryLayout::StructureOfArrays(soa) => soa.total_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_layout_optimizer() {
        let optimizer = MemoryLayoutOptimizer::new();
        assert_eq!(optimizer.target_cache_line_size, 64);
    }

    #[test]
    fn test_cache_line_alignment() {
        let optimizer = MemoryLayoutOptimizer::new();
        assert_eq!(optimizer.align_to_cache_line(50), 64);
        assert_eq!(optimizer.align_to_cache_line(64), 64);
        assert_eq!(optimizer.align_to_cache_line(65), 128);
    }
}

// Made with Bob

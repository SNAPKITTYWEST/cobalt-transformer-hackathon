// Chunk Scheduler
// Manages deterministic execution of vector chunks

use crate::vector::chunk_ir::*;
use anyhow::Result;
use std::collections::{HashMap, VecDeque};

pub struct ChunkScheduler {
    chunk_ir: ChunkIR,
    execution_plan: Option<ExecutionPlan>,
}

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub stages: Vec<ExecutionStage>,
    pub total_chunks: usize,
    pub estimated_time: f64,
}

#[derive(Debug, Clone)]
pub struct ExecutionStage {
    pub stage_id: usize,
    pub chunks: Vec<usize>,
    pub can_parallelize: bool,
    pub dependencies_satisfied: bool,
}

#[derive(Debug, Clone)]
pub struct ChunkPartition {
    pub chunk_id: usize,
    pub start_offset: usize,
    pub end_offset: usize,
    pub record_count: usize,
}

impl ChunkScheduler {
    pub fn new(chunk_ir: ChunkIR) -> Self {
        Self {
            chunk_ir,
            execution_plan: None,
        }
    }

    pub fn schedule(&mut self) -> Result<ExecutionPlan> {
        let order = self.chunk_ir.topological_order();
        let mut stages = Vec::new();
        let mut current_stage = Vec::new();
        let mut scheduled = std::collections::HashSet::new();
        
        for &chunk_id in &order {
            let deps = self.chunk_ir.get_dependencies(chunk_id);
            let deps_satisfied = deps.iter().all(|d| scheduled.contains(d));
            
            if deps_satisfied {
                current_stage.push(chunk_id);
                scheduled.insert(chunk_id);
            } else {
                // Start new stage
                if !current_stage.is_empty() {
                    stages.push(ExecutionStage {
                        stage_id: stages.len(),
                        chunks: current_stage.clone(),
                        can_parallelize: current_stage.len() > 1,
                        dependencies_satisfied: true,
                    });
                    current_stage.clear();
                }
                current_stage.push(chunk_id);
                scheduled.insert(chunk_id);
            }
        }
        
        // Add final stage
        if !current_stage.is_empty() {
            stages.push(ExecutionStage {
                stage_id: stages.len(),
                chunks: current_stage,
                can_parallelize: false,
                dependencies_satisfied: true,
            });
        }
        
        let plan = ExecutionPlan {
            stages,
            total_chunks: order.len(),
            estimated_time: self.estimate_execution_time(&order),
        };
        
        self.execution_plan = Some(plan.clone());
        Ok(plan)
    }

    pub fn partition_dataset(&self, _chunk_id: usize, dataset_size: usize, chunk_size: usize) -> Vec<ChunkPartition> {
        let mut partitions = Vec::new();
        let mut offset = 0;
        let mut partition_id = 0;
        
        while offset < dataset_size {
            let end = (offset + chunk_size).min(dataset_size);
            partitions.push(ChunkPartition {
                chunk_id: partition_id,
                start_offset: offset,
                end_offset: end,
                record_count: end - offset,
            });
            offset = end;
            partition_id += 1;
        }
        
        partitions
    }

    pub fn get_execution_order(&self) -> Vec<usize> {
        if let Some(plan) = &self.execution_plan {
            plan.stages.iter()
                .flat_map(|stage| stage.chunks.iter())
                .copied()
                .collect()
        } else {
            Vec::new()
        }
    }

    fn estimate_execution_time(&self, order: &[usize]) -> f64 {
        let mut total_time = 0.0;
        
        for &chunk_id in order {
            if let Some(chunk) = self.chunk_ir.get_chunk(chunk_id) {
                // Simple cost model
                let operation_cost = chunk.operations.len() as f64 * 0.1;
                let memory_cost = (chunk.input_schema.len() + chunk.output_schema.len()) as f64 * 0.05;
                total_time += operation_cost + memory_cost;
            }
        }
        
        total_time
    }
}

impl ExecutionPlan {
    pub fn can_execute_in_parallel(&self, stage_id: usize) -> bool {
        self.stages.get(stage_id)
            .map(|s| s.can_parallelize)
            .unwrap_or(false)
    }

    pub fn get_stage(&self, stage_id: usize) -> Option<&ExecutionStage> {
        self.stages.get(stage_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let ir = ChunkIR::new();
        let scheduler = ChunkScheduler::new(ir);
        assert!(scheduler.execution_plan.is_none());
    }

    #[test]
    fn test_partition_dataset() {
        let ir = ChunkIR::new();
        let scheduler = ChunkScheduler::new(ir);
        
        let partitions = scheduler.partition_dataset(0, 1000, 100);
        assert_eq!(partitions.len(), 10);
        assert_eq!(partitions[0].record_count, 100);
    }
}

// Made with Bob

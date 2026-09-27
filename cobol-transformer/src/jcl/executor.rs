use crate::jcl::ast::*;
use crate::jcl::job_graph::{JobGraph, JobGraphBuilder, topological_order};
use anyhow::Result;
use std::collections::HashMap;

pub struct JclExecutor {
    job: JclJob,
    graph: JobGraph,
    step_results: HashMap<String, StepResult>,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_name: String,
    pub return_code: i32,
    pub output: Vec<String>,
    pub datasets_created: Vec<String>,
}

impl JclExecutor {
    pub fn new(job: JclJob) -> Result<Self> {
        let mut builder = JobGraphBuilder::new();
        let graph = builder.build(&job)?;
        
        Ok(Self {
            job,
            graph,
            step_results: HashMap::new(),
        })
    }

    pub fn execute(&mut self) -> Result<ExecutionResult> {
        let order = topological_order(&self.graph)?;
        
        for node_idx in order {
            let node = &self.graph[node_idx];
            let step = &node.step;
            
            // Check step condition
            if let Some(condition) = &step.condition {
                if !self.evaluate_condition(condition)? {
                    continue;
                }
            }
            
            // Execute step
            let result = self.execute_step(step)?;
            self.step_results.insert(step.name.clone(), result);
        }
        
        Ok(ExecutionResult {
            job_name: self.job.name.clone(),
            steps_executed: self.step_results.len(),
            overall_rc: self.compute_overall_rc(),
        })
    }

    fn execute_step(&self, step: &JclStep) -> Result<StepResult> {
        // This is a stub - actual execution would invoke COBOL programs
        // or other system utilities
        
        Ok(StepResult {
            step_name: step.name.clone(),
            return_code: 0,
            output: Vec::new(),
            datasets_created: Vec::new(),
        })
    }

    fn evaluate_condition(&self, _condition: &StepCondition) -> Result<bool> {
        // Stub for condition evaluation
        Ok(true)
    }

    fn compute_overall_rc(&self) -> i32 {
        self.step_results.values()
            .map(|r| r.return_code)
            .max()
            .unwrap_or(0)
    }

    pub fn get_step_result(&self, step_name: &str) -> Option<&StepResult> {
        self.step_results.get(step_name)
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub job_name: String,
    pub steps_executed: usize,
    pub overall_rc: i32,
}

impl ExecutionResult {
    pub fn is_success(&self) -> bool {
        self.overall_rc == 0
    }
}

// Made with Bob

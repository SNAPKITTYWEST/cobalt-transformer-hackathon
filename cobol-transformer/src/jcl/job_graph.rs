use crate::jcl::ast::*;
use anyhow::Result;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

pub struct JobGraphWrapper(DiGraph<JobNode, JobEdge>);

pub type JobGraph = DiGraph<JobNode, JobEdge>;

#[derive(Debug, Clone)]
pub struct JobNode {
    pub id: usize,
    pub step: JclStep,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum JobEdge {
    Sequential,
    Conditional(String),
    DataDependency(String),
}

pub struct JobGraphBuilder {
    next_id: usize,
}

impl JobGraphBuilder {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    pub fn build(&mut self, job: &JclJob) -> Result<JobGraph> {
        let mut graph = DiGraph::new();
        let mut step_nodes: HashMap<String, NodeIndex> = HashMap::new();
        
        // Create nodes for each step
        for step in &job.steps {
            let node = JobNode {
                id: self.next_node_id(),
                step: step.clone(),
                dependencies: self.extract_dependencies(step),
            };
            
            let node_idx = graph.add_node(node);
            step_nodes.insert(step.name.clone(), node_idx);
        }
        
        // Create edges based on dependencies
        for (_step_name, node_idx) in &step_nodes {
            let dependencies = graph[*node_idx].dependencies.clone();
            
            for dep in &dependencies {
                if let Some(dep_idx) = step_nodes.get(dep) {
                    graph.add_edge(*dep_idx, *node_idx, JobEdge::DataDependency(dep.clone()));
                }
            }
        }
        
        // Add sequential edges
        let step_indices: Vec<_> = job.steps.iter()
            .filter_map(|s| step_nodes.get(&s.name))
            .copied()
            .collect();
        
        for i in 0..step_indices.len().saturating_sub(1) {
            graph.add_edge(step_indices[i], step_indices[i + 1], JobEdge::Sequential);
        }
        
        Ok(graph)
    }

    fn extract_dependencies(&self, step: &JclStep) -> Vec<String> {
        let mut deps = Vec::new();
        
        // Extract dependencies from DD statements
        for dd in &step.dd_statements {
            if let Some(dsn) = &dd.dsn {
                // Check if DSN references another step's output
                if dsn.contains("&&") {
                    // Temporary dataset - extract step name
                    if let Some(step_ref) = self.extract_step_reference(dsn) {
                        deps.push(step_ref);
                    }
                }
            }
            
            // Check DISP for PASS
            if let Some(disp) = &dd.disp {
                if disp.normal_termination == DispositionAction::Pass {
                    // This creates a dependency for the next step
                }
            }
        }
        
        // Extract dependencies from COND parameter
        if let Some(cond) = &step.exec_statement.cond {
            if let Some(step_ref) = self.extract_step_reference(cond) {
                deps.push(step_ref);
            }
        }
        
        deps
    }

    fn extract_step_reference(&self, text: &str) -> Option<String> {
        // Simple extraction - in production this would be more sophisticated
        if text.contains('.') {
            text.split('.').next().map(|s| s.to_string())
        } else {
            None
        }
    }

    fn next_node_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

pub fn topological_order(graph: &JobGraph) -> Result<Vec<NodeIndex>> {
    use petgraph::algo::toposort;
    
    match toposort(graph, None) {
        Ok(order) => Ok(order),
        Err(_) => anyhow::bail!("Job graph contains cycles"),
    }
}

impl JobGraphWrapper {
    pub fn topological_order(&self) -> Result<Vec<NodeIndex>> {
        topological_order(&self.0)
    }

    pub fn get_step_dependencies(&self, node: NodeIndex) -> Vec<NodeIndex> {
        use petgraph::Direction;
        
        self.0.neighbors_directed(node, Direction::Incoming)
            .collect()
    }

    pub fn get_step_dependents(&self, node: NodeIndex) -> Vec<NodeIndex> {
        use petgraph::Direction;
        
        self.0.neighbors_directed(node, Direction::Outgoing)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_simple_graph() {
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
        let graph = builder.build(&job).unwrap();
        
        assert_eq!(graph.node_count(), 2);
    }
}

// Made with Bob

use crate::ast::*;
use anyhow::Result;
use petgraph::graph::{DiGraph, NodeIndex};

pub type ControlFlowGraph = DiGraph<BasicBlock, EdgeType>;

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum EdgeType {
    Fallthrough,
    Branch,
    Call,
    Return,
}

pub struct ControlFlowGraphBuilder {
    next_id: usize,
}

impl ControlFlowGraphBuilder {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    pub fn build(&mut self, _program: &CobolProgram) -> Result<ControlFlowGraph> {
        let mut graph = DiGraph::new();
        
        // Create entry block
        let _entry = graph.add_node(BasicBlock {
            id: self.next_block_id(),
            statements: Vec::new(),
        });

        Ok(graph)
    }

    fn next_block_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

// Made with Bob

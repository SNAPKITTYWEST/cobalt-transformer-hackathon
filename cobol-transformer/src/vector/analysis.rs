// Vectorization Analysis Engine
// Analyzes COBOL code to identify vectorizable operations

use crate::ast::*;
use crate::cfg::ControlFlowGraph;
use crate::vector::chunk_ir::*;
use anyhow::Result;
use std::collections::{HashMap, HashSet};

pub struct VectorizationAnalyzer {
    candidates: Vec<VectorizationCandidate>,
    safety_checks: SafetyChecker,
}

#[derive(Debug, Clone)]
pub struct VectorizationCandidate {
    pub id: usize,
    pub loop_info: Option<LoopInfo>,
    pub operations: Vec<Statement>,
    pub data_dependencies: Vec<DataDependency>,
    pub is_safe: bool,
    pub estimated_speedup: f64,
    pub vector_width: usize,
}

#[derive(Debug, Clone)]
pub struct LoopInfo {
    pub loop_variable: String,
    pub start_value: i64,
    pub end_value: i64,
    pub step: i64,
    pub iteration_count: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct DataDependency {
    pub from_statement: usize,
    pub to_statement: usize,
    pub variable: String,
    pub dependency_type: DependencyKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyKind {
    ReadAfterWrite,
    WriteAfterRead,
    WriteAfterWrite,
    AntiDependence,
}

struct SafetyChecker {
    aliasing_analysis: AliasingAnalysis,
}

struct AliasingAnalysis {
    may_alias: HashMap<String, HashSet<String>>,
}

impl VectorizationAnalyzer {
    pub fn new() -> Self {
        Self {
            candidates: Vec::new(),
            safety_checks: SafetyChecker::new(),
        }
    }

    pub fn analyze(&mut self, program: &CobolProgram) -> Result<Vec<VectorizationCandidate>> {
        self.candidates.clear();
        
        if let Some(procedure) = &program.procedure {
            for paragraph in &procedure.paragraphs {
                self.analyze_paragraph(paragraph)?;
            }
        }
        
        Ok(self.candidates.clone())
    }

    fn analyze_paragraph(&mut self, paragraph: &Paragraph) -> Result<()> {
        // Look for PERFORM VARYING loops
        for (idx, statement) in paragraph.statements.iter().enumerate() {
            if let Statement::Perform(perform) = statement {
                if let Some(varying) = &perform.varying {
                    let candidate = self.analyze_perform_varying(perform, idx)?;
                    if candidate.is_safe {
                        self.candidates.push(candidate);
                    }
                }
            }
        }
        
        // Look for sequential record processing patterns
        self.analyze_sequential_operations(&paragraph.statements)?;
        
        Ok(())
    }

    fn analyze_perform_varying(&self, perform: &PerformStatement, idx: usize) -> Result<VectorizationCandidate> {
        let varying = perform.varying.as_ref().unwrap();
        
        // Extract end value from condition (simplified - production would parse the condition)
        let end_value = 100; // Placeholder - would extract from varying.until condition
        
        let loop_info = LoopInfo {
            loop_variable: varying.identifier.name.clone(),
            start_value: self.extract_numeric_value(&varying.from)?,
            end_value,
            step: self.extract_numeric_value(&varying.by).unwrap_or(1),
            iteration_count: None,
        };
        
        // Extract operations from the loop body
        let operations = match &perform.target {
            PerformTarget::Inline(stmts) => stmts.clone(),
            _ => Vec::new(),
        };
        
        // Analyze data dependencies
        let dependencies = self.analyze_dependencies(&operations)?;
        
        // Check if vectorization is safe
        let is_safe = self.is_vectorization_safe(&operations, &dependencies);
        
        // Estimate speedup
        let estimated_speedup = if is_safe {
            self.estimate_speedup(&operations, &loop_info)
        } else {
            1.0
        };
        
        Ok(VectorizationCandidate {
            id: idx,
            loop_info: Some(loop_info),
            operations,
            data_dependencies: dependencies,
            is_safe,
            estimated_speedup,
            vector_width: 8,
        })
    }

    fn analyze_sequential_operations(&mut self, statements: &[Statement]) -> Result<()> {
        // Look for patterns like:
        // READ file INTO record
        // COMPUTE result = field1 * field2
        // WRITE output FROM result
        
        let mut current_sequence = Vec::new();
        
        for (idx, stmt) in statements.iter().enumerate() {
            match stmt {
                Statement::Read(_) => {
                    current_sequence.clear();
                    current_sequence.push(stmt.clone());
                }
                Statement::Compute(_) | Statement::Move(_) | Statement::Add(_) => {
                    if !current_sequence.is_empty() {
                        current_sequence.push(stmt.clone());
                    }
                }
                Statement::Write(_) => {
                    if !current_sequence.is_empty() {
                        current_sequence.push(stmt.clone());
                        
                        // Analyze this sequence as a candidate
                        let dependencies = self.analyze_dependencies(&current_sequence)?;
                        let is_safe = self.is_vectorization_safe(&current_sequence, &dependencies);
                        
                        if is_safe {
                            self.candidates.push(VectorizationCandidate {
                                id: idx,
                                loop_info: None,
                                operations: current_sequence.clone(),
                                data_dependencies: dependencies,
                                is_safe,
                                estimated_speedup: 4.0,
                                vector_width: 8,
                            });
                        }
                        
                        current_sequence.clear();
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    fn analyze_dependencies(&self, operations: &[Statement]) -> Result<Vec<DataDependency>> {
        let mut dependencies = Vec::new();
        let mut reads: HashMap<String, Vec<usize>> = HashMap::new();
        let mut writes: HashMap<String, Vec<usize>> = HashMap::new();
        
        for (idx, stmt) in operations.iter().enumerate() {
            let (read_vars, write_vars) = self.extract_variables(stmt);
            
            for var in read_vars {
                reads.entry(var.clone()).or_insert_with(Vec::new).push(idx);
                
                // Check for RAW (Read After Write)
                if let Some(write_indices) = writes.get(&var) {
                    for &write_idx in write_indices {
                        if write_idx < idx {
                            dependencies.push(DataDependency {
                                from_statement: write_idx,
                                to_statement: idx,
                                variable: var.clone(),
                                dependency_type: DependencyKind::ReadAfterWrite,
                            });
                        }
                    }
                }
            }
            
            for var in write_vars {
                writes.entry(var.clone()).or_insert_with(Vec::new).push(idx);
                
                // Check for WAR (Write After Read)
                if let Some(read_indices) = reads.get(&var) {
                    for &read_idx in read_indices {
                        if read_idx < idx {
                            dependencies.push(DataDependency {
                                from_statement: read_idx,
                                to_statement: idx,
                                variable: var.clone(),
                                dependency_type: DependencyKind::WriteAfterRead,
                            });
                        }
                    }
                }
                
                // Check for WAW (Write After Write)
                if let Some(write_indices) = writes.get(&var) {
                    for &write_idx in write_indices {
                        if write_idx < idx {
                            dependencies.push(DataDependency {
                                from_statement: write_idx,
                                to_statement: idx,
                                variable: var.clone(),
                                dependency_type: DependencyKind::WriteAfterWrite,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(dependencies)
    }

    fn extract_variables(&self, stmt: &Statement) -> (Vec<String>, Vec<String>) {
        let mut reads = Vec::new();
        let mut writes = Vec::new();
        
        match stmt {
            Statement::Move(mv) => {
                if let Expression::Identifier(id) = &mv.source {
                    reads.push(id.name.clone());
                }
                for target in &mv.targets {
                    writes.push(target.name.clone());
                }
            }
            Statement::Compute(comp) => {
                // Extract reads from expression
                self.extract_expression_vars(&comp.expression, &mut reads);
                writes.push(comp.target.name.clone());
            }
            Statement::Add(add) => {
                for operand in &add.operands {
                    if let Expression::Identifier(id) = operand {
                        reads.push(id.name.clone());
                    }
                }
                if let Some(targets) = &add.to {
                    for target in targets {
                        writes.push(target.name.clone());
                    }
                }
            }
            _ => {}
        }
        
        (reads, writes)
    }

    fn extract_expression_vars(&self, expr: &Expression, vars: &mut Vec<String>) {
        match expr {
            Expression::Identifier(id) => {
                vars.push(id.name.clone());
            }
            Expression::Binary(bin) => {
                self.extract_expression_vars(&bin.left, vars);
                self.extract_expression_vars(&bin.right, vars);
            }
            Expression::Unary(un) => {
                self.extract_expression_vars(&un.operand, vars);
            }
            _ => {}
        }
    }

    fn is_vectorization_safe(&self, operations: &[Statement], dependencies: &[DataDependency]) -> bool {
        // Check for loop-carried dependencies
        for dep in dependencies {
            match dep.dependency_type {
                DependencyKind::ReadAfterWrite => {
                    // RAW dependencies prevent vectorization if they cross iterations
                    if self.is_loop_carried(dep) {
                        return false;
                    }
                }
                DependencyKind::WriteAfterWrite => {
                    // WAW dependencies prevent vectorization
                    return false;
                }
                _ => {}
            }
        }
        
        // Check for side effects
        for stmt in operations {
            if self.has_unsafe_side_effects(stmt) {
                return false;
            }
        }
        
        true
    }

    fn is_loop_carried(&self, _dep: &DataDependency) -> bool {
        // Simplified check - production would analyze array subscripts
        false
    }

    fn has_unsafe_side_effects(&self, stmt: &Statement) -> bool {
        matches!(
            stmt,
            Statement::Display(_) | Statement::Accept(_) | Statement::Call(_)
        )
    }

    fn estimate_speedup(&self, _operations: &[Statement], loop_info: &LoopInfo) -> f64 {
        let iteration_count = (loop_info.end_value - loop_info.start_value) / loop_info.step;
        let vector_width = 8.0;
        
        // Simple model: speedup = min(vector_width, iteration_count) * efficiency
        let efficiency = 0.8; // Account for overhead
        let max_speedup = vector_width * efficiency;
        
        if iteration_count as f64 >= vector_width {
            max_speedup
        } else {
            iteration_count as f64 * efficiency
        }
    }

    fn extract_numeric_value(&self, expr: &Expression) -> Result<i64> {
        match expr {
            Expression::Literal(Literal::Numeric(s)) => {
                Ok(s.parse()?)
            }
            _ => Ok(0),
        }
    }

    pub fn generate_chunk_ir(&self, candidate: &VectorizationCandidate) -> Result<Chunk> {
        let mut chunk = Chunk::new(candidate.id);
        
        // Set vector width
        chunk.vector_width = candidate.vector_width;
        
        // Convert operations to chunk operations
        for stmt in &candidate.operations {
            match stmt {
                Statement::Compute(comp) => {
                    chunk.add_operation(ChunkOperation::Load {
                        source: "INPUT".to_string(),
                        target: "REG1".to_string(),
                    });
                    // Add compute operation
                    chunk.add_operation(ChunkOperation::Store {
                        source: "REG1".to_string(),
                        target: comp.target.name.clone(),
                    });
                }
                Statement::Move(mv) => {
                    if let Expression::Identifier(id) = &mv.source {
                        for target in &mv.targets {
                            chunk.add_operation(ChunkOperation::Move {
                                source: id.name.clone(),
                                target: target.name.clone(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        
        Ok(chunk)
    }
}

impl SafetyChecker {
    fn new() -> Self {
        Self {
            aliasing_analysis: AliasingAnalysis::new(),
        }
    }
}

impl AliasingAnalysis {
    fn new() -> Self {
        Self {
            may_alias: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_analysis() {
        let analyzer = VectorizationAnalyzer::new();
        
        // Create simple statements
        let statements = vec![];
        
        let deps = analyzer.analyze_dependencies(&statements).unwrap();
        assert_eq!(deps.len(), 0);
    }
}

// Made with Bob

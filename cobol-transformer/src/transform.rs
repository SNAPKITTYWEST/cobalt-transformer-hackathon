use crate::ast::*;
use crate::symbol_table::SymbolTable;
use anyhow::Result;

pub struct TransformEngine {
}

impl TransformEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn apply_pass(&mut self, pass_name: &str, program: &mut CobolProgram, _symbols: &SymbolTable) -> Result<()> {
        match pass_name {
            "normalize" => self.normalize(program),
            "modernize" => self.modernize(program),
            _ => Ok(()),
        }
    }

    fn normalize(&self, _program: &mut CobolProgram) -> Result<()> {
        // Normalization pass stub
        Ok(())
    }

    fn modernize(&self, _program: &mut CobolProgram) -> Result<()> {
        // Modernization pass stub
        Ok(())
    }
}

// Made with Bob

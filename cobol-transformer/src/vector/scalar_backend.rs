// Scalar Execution Backend
// Provides fallback scalar execution for all chunks

use crate::vector::chunk_ir::*;
use crate::vector::decimal::*;
use anyhow::Result;
use std::collections::HashMap;

pub struct ScalarBackend {
    decimal_engine: DecimalEngine,
    registers: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Decimal(DecimalValue),
    Integer(i64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub input_data: HashMap<String, Vec<Value>>,
    pub output_data: HashMap<String, Vec<Value>>,
    pub record_count: usize,
}

impl ScalarBackend {
    pub fn new() -> Self {
        Self {
            decimal_engine: DecimalEngine::new(),
            registers: HashMap::new(),
        }
    }

    pub fn execute_chunk(&mut self, chunk: &Chunk, context: &mut ExecutionContext) -> Result<()> {
        // Execute chunk operations for each record
        for record_idx in 0..context.record_count {
            self.execute_chunk_for_record(chunk, context, record_idx)?;
        }
        
        Ok(())
    }

    fn execute_chunk_for_record(&mut self, chunk: &Chunk, context: &mut ExecutionContext, record_idx: usize) -> Result<()> {
        for operation in &chunk.operations {
            self.execute_operation(operation, context, record_idx)?;
        }
        
        Ok(())
    }

    fn execute_operation(&mut self, operation: &ChunkOperation, context: &mut ExecutionContext, record_idx: usize) -> Result<()> {
        match operation {
            ChunkOperation::Load { source, target } => {
                if let Some(values) = context.input_data.get(source) {
                    if let Some(value) = values.get(record_idx) {
                        self.registers.insert(target.clone(), value.clone());
                    }
                }
            }
            ChunkOperation::Store { source, target } => {
                if let Some(value) = self.registers.get(source) {
                    context.output_data
                        .entry(target.clone())
                        .or_insert_with(Vec::new)
                        .push(value.clone());
                }
            }
            ChunkOperation::Add { left, right, result } => {
                let left_val = self.get_decimal_value(left)?;
                let right_val = self.get_decimal_value(right)?;
                let result_val = self.decimal_engine.add(&left_val, &right_val)?;
                self.registers.insert(result.clone(), Value::Decimal(result_val));
            }
            ChunkOperation::Subtract { left, right, result } => {
                let left_val = self.get_decimal_value(left)?;
                let right_val = self.get_decimal_value(right)?;
                let result_val = self.decimal_engine.subtract(&left_val, &right_val)?;
                self.registers.insert(result.clone(), Value::Decimal(result_val));
            }
            ChunkOperation::Multiply { left, right, result } => {
                let left_val = self.get_decimal_value(left)?;
                let right_val = self.get_decimal_value(right)?;
                let result_val = self.decimal_engine.multiply(&left_val, &right_val)?;
                self.registers.insert(result.clone(), Value::Decimal(result_val));
            }
            ChunkOperation::Divide { left, right, result } => {
                let left_val = self.get_decimal_value(left)?;
                let right_val = self.get_decimal_value(right)?;
                let result_val = self.decimal_engine.divide(&left_val, &right_val)?;
                self.registers.insert(result.clone(), Value::Decimal(result_val));
            }
            ChunkOperation::Move { source, target } => {
                if let Some(value) = self.registers.get(source) {
                    self.registers.insert(target.clone(), value.clone());
                }
            }
            ChunkOperation::Compare { left, right, operator } => {
                let left_val = self.get_decimal_value(left)?;
                let right_val = self.get_decimal_value(right)?;
                let result = self.compare_values(&left_val, &right_val, operator);
                self.registers.insert("COMPARE_RESULT".to_string(), Value::Boolean(result));
            }
            ChunkOperation::DecimalConvert { source, target, from_type, to_type } => {
                if let Some(value) = self.registers.get(source) {
                    let converted = self.convert_decimal(value, from_type, to_type)?;
                    self.registers.insert(target.clone(), converted);
                }
            }
            ChunkOperation::Validate { source, rules } => {
                if let Some(value) = self.registers.get(source) {
                    for rule in rules {
                        self.validate_value(value, rule)?;
                    }
                }
            }
        }
        
        Ok(())
    }

    fn get_decimal_value(&self, name: &str) -> Result<DecimalValue> {
        if let Some(value) = self.registers.get(name) {
            match value {
                Value::Decimal(d) => Ok(d.clone()),
                Value::Integer(i) => Ok(DecimalValue::from_i64(*i, 0)),
                _ => anyhow::bail!("Cannot convert to decimal"),
            }
        } else {
            Ok(DecimalValue::zero())
        }
    }

    fn compare_values(&self, left: &DecimalValue, right: &DecimalValue, operator: &CompareOp) -> bool {
        let left_str = left.to_string();
        let right_str = right.to_string();
        
        match operator {
            CompareOp::Equal => left_str == right_str,
            CompareOp::NotEqual => left_str != right_str,
            CompareOp::Greater => left_str > right_str,
            CompareOp::Less => left_str < right_str,
            CompareOp::GreaterOrEqual => left_str >= right_str,
            CompareOp::LessOrEqual => left_str <= right_str,
        }
    }

    fn convert_decimal(&self, value: &Value, _from_type: &CobolDataType, _to_type: &CobolDataType) -> Result<Value> {
        // Simplified conversion - production would handle all COBOL types
        Ok(value.clone())
    }

    fn validate_value(&self, value: &Value, rule: &ValidationRule) -> Result<()> {
        match rule {
            ValidationRule::Numeric => {
                if !matches!(value, Value::Decimal(_) | Value::Integer(_)) {
                    anyhow::bail!("Value is not numeric");
                }
            }
            ValidationRule::Range { min, max } => {
                if let Value::Integer(i) = value {
                    if *i < *min || *i > *max {
                        anyhow::bail!("Value out of range");
                    }
                }
            }
            ValidationRule::Pattern { .. } => {
                // Pattern validation would go here
            }
        }
        Ok(())
    }
}

impl ExecutionContext {
    pub fn new(record_count: usize) -> Self {
        Self {
            input_data: HashMap::new(),
            output_data: HashMap::new(),
            record_count,
        }
    }

    pub fn add_input(&mut self, name: String, values: Vec<Value>) {
        self.input_data.insert(name, values);
    }

    pub fn get_output(&self, name: &str) -> Option<&Vec<Value>> {
        self.output_data.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_backend_creation() {
        let backend = ScalarBackend::new();
        assert_eq!(backend.registers.len(), 0);
    }

    #[test]
    fn test_execution_context() {
        let mut context = ExecutionContext::new(10);
        context.add_input("TEST".to_string(), vec![Value::Integer(42)]);
        assert_eq!(context.input_data.len(), 1);
    }
}

// Made with Bob

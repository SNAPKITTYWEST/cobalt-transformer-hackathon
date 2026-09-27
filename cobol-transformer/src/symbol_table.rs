use crate::ast::*;
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub level: Option<u8>,
}

#[derive(Debug, Clone)]
pub enum SymbolType {
    Program,
    Section,
    Paragraph,
    Variable,
    File,
    Condition88,
}

#[derive(Debug)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) {
        self.symbols.insert(name, symbol);
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }
}

pub struct SymbolTableBuilder {
    table: SymbolTable,
}

impl SymbolTableBuilder {
    pub fn new() -> Self {
        Self {
            table: SymbolTable::new(),
        }
    }

    pub fn build(&mut self, program: &CobolProgram) -> Result<SymbolTable> {
        // Add program name
        self.table.insert(
            program.name().to_string(),
            Symbol {
                name: program.name().to_string(),
                symbol_type: SymbolType::Program,
                level: None,
            },
        );

        // Add data items
        if let Some(data) = &program.data {
            if let Some(ws) = &data.working_storage {
                for item in &ws.items {
                    self.add_data_item(item);
                }
            }
        }

        // Add paragraphs
        if let Some(proc) = &program.procedure {
            for para in &proc.paragraphs {
                self.table.insert(
                    para.name.clone(),
                    Symbol {
                        name: para.name.clone(),
                        symbol_type: SymbolType::Paragraph,
                        level: None,
                    },
                );
            }
        }

        Ok(std::mem::replace(&mut self.table, SymbolTable::new()))
    }

    fn add_data_item(&mut self, item: &DataItem) {
        if let Some(name) = &item.name {
            self.table.insert(
                name.clone(),
                Symbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Variable,
                    level: Some(item.level),
                },
            );
        }

        for child in &item.children {
            self.add_data_item(child);
        }
    }
}

// Made with Bob

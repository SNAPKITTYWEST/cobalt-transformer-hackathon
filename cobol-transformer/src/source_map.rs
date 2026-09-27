use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl SourceLocation {
    pub fn new(file: PathBuf, line: usize, column: usize, offset: usize) -> Self {
        Self {
            file,
            line,
            column,
            offset,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl SourceSpan {
    pub fn new(start: SourceLocation, end: SourceLocation) -> Self {
        Self { start, end }
    }

    pub fn single(location: SourceLocation) -> Self {
        Self {
            start: location.clone(),
            end: location,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceMapping {
    pub expanded_line: usize,
    pub expanded_column: usize,
    pub original_file: PathBuf,
    pub original_line: usize,
    pub original_column: usize,
}

#[derive(Debug, Default, Clone)]
pub struct SourceMap {
    mappings: Vec<SourceMapping>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    pub fn add_mapping(&mut self, mapping: SourceMapping) {
        self.mappings.push(mapping);
    }

    pub fn get_original(&self, expanded_line: usize, expanded_column: usize) -> Option<&SourceMapping> {
        self.mappings.iter().find(|m| {
            m.expanded_line == expanded_line && m.expanded_column == expanded_column
        })
    }

    pub fn all_mappings(&self) -> &[SourceMapping] {
        &self.mappings
    }
}

// Made with Bob

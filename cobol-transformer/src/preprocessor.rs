use crate::diagnostics::{Diagnostic, DiagnosticCollector};
use crate::source_map::{SourceMap, SourceMapping};
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PreprocessedSource {
    pub content: String,
    pub source_map: SourceMap,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct Preprocessor {
    copy_paths: Vec<PathBuf>,
    defines: HashMap<String, String>,
    source_map: SourceMap,
    diagnostics: DiagnosticCollector,
    processed_files: Vec<PathBuf>,
}

impl Preprocessor {
    pub fn new() -> Self {
        Self {
            copy_paths: vec![PathBuf::from(".")],
            defines: HashMap::new(),
            source_map: SourceMap::new(),
            diagnostics: DiagnosticCollector::new(),
            processed_files: Vec::new(),
        }
    }

    pub fn add_copy_path(&mut self, path: PathBuf) {
        self.copy_paths.push(path);
    }

    pub fn process(&mut self, source: &str, file: &Path) -> Result<PreprocessedSource> {
        self.processed_files.clear();
        self.processed_files.push(file.to_path_buf());

        let mut output = String::new();
        let mut line_num = 1;
        let mut expanded_line = 1;

        let lines: Vec<&str> = source.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();

            // Handle compiler directives
            if trimmed.starts_with(">>") {
                self.process_directive(trimmed, file, line_num)?;
                i += 1;
                line_num += 1;
                continue;
            }

            // Handle COPY statements
            if trimmed.to_uppercase().starts_with("COPY") {
                let copy_result = self.process_copy(trimmed, file, line_num)?;
                output.push_str(&copy_result);
                
                // Add source mapping for each line in the copied content
                for _ in copy_result.lines() {
                    self.source_map.add_mapping(SourceMapping {
                        expanded_line,
                        expanded_column: 1,
                        original_file: file.to_path_buf(),
                        original_line: line_num,
                        original_column: 1,
                    });
                    expanded_line += 1;
                }
                
                i += 1;
                line_num += 1;
                continue;
            }

            // Handle REPLACE statements
            if trimmed.to_uppercase().starts_with("REPLACE") {
                self.process_replace(trimmed)?;
                i += 1;
                line_num += 1;
                continue;
            }

            // Regular line - apply replacements
            let processed_line = self.apply_replacements(line);
            output.push_str(&processed_line);
            output.push('\n');

            self.source_map.add_mapping(SourceMapping {
                expanded_line,
                expanded_column: 1,
                original_file: file.to_path_buf(),
                original_line: line_num,
                original_column: 1,
            });

            expanded_line += 1;
            i += 1;
            line_num += 1;
        }

        Ok(PreprocessedSource {
            content: output,
            source_map: self.source_map.clone(),
            diagnostics: self.diagnostics.all().to_vec(),
        })
    }

    fn process_directive(&mut self, directive: &str, file: &Path, line: usize) -> Result<()> {
        let upper = directive.to_uppercase();

        if upper.starts_with(">>SOURCE FORMAT FREE") || upper.starts_with(">>SOURCE FREE") {
            // Format directive - handled by format normalizer
            Ok(())
        } else if upper.starts_with(">>DEFINE") {
            self.process_define(directive)
        } else if upper.starts_with(">>SET") {
            self.process_set(directive)
        } else if upper.starts_with(">>IF") {
            // Conditional compilation - simplified implementation
            Ok(())
        } else {
            self.diagnostics.add(Diagnostic::warning(
                "PREPROC-001",
                format!("Unknown compiler directive: {}", directive),
                file.to_path_buf(),
                line,
                1,
            ));
            Ok(())
        }
    }

    fn process_define(&mut self, directive: &str) -> Result<()> {
        // >>DEFINE name AS value
        let parts: Vec<&str> = directive.split_whitespace().collect();
        if parts.len() >= 4 && parts[2].to_uppercase() == "AS" {
            let name = parts[1].to_string();
            let value = parts[3..].join(" ");
            self.defines.insert(name, value);
        }
        Ok(())
    }

    fn process_set(&mut self, directive: &str) -> Result<()> {
        // >>SET name value
        let parts: Vec<&str> = directive.split_whitespace().collect();
        if parts.len() >= 3 {
            let name = parts[1].to_string();
            let value = parts[2..].join(" ");
            self.defines.insert(name, value);
        }
        Ok(())
    }

    fn process_copy(&mut self, statement: &str, current_file: &Path, line: usize) -> Result<String> {
        // Parse COPY statement: COPY "filename" or COPY filename
        let upper = statement.to_uppercase();
        let copy_start = upper.find("COPY").unwrap() + 4;
        let rest = statement[copy_start..].trim();

        // Extract filename
        let filename = if rest.starts_with('"') {
            // Quoted filename
            let end_quote = rest[1..].find('"').context("Unterminated quote in COPY")?;
            &rest[1..end_quote + 1]
        } else if rest.starts_with('\'') {
            // Single-quoted filename
            let end_quote = rest[1..].find('\'').context("Unterminated quote in COPY")?;
            &rest[1..end_quote + 1]
        } else {
            // Unquoted filename - take until period or end
            rest.split('.').next().unwrap_or(rest).trim()
        };

        // Find the file in copy paths
        let mut found_path: Option<PathBuf> = None;
        
        for copy_path in &self.copy_paths {
            let candidate = copy_path.join(filename);
            if candidate.exists() {
                found_path = Some(candidate);
                break;
            }
            
            // Try with .cpy extension
            let candidate_cpy = copy_path.join(format!("{}.cpy", filename));
            if candidate_cpy.exists() {
                found_path = Some(candidate_cpy);
                break;
            }

            // Try with .cbl extension
            let candidate_cbl = copy_path.join(format!("{}.cbl", filename));
            if candidate_cbl.exists() {
                found_path = Some(candidate_cbl);
                break;
            }
        }

        let copy_file = found_path.context(format!("COPY file not found: {}", filename))?;

        // Check for circular dependencies
        if self.processed_files.contains(&copy_file) {
            self.diagnostics.add(Diagnostic::error(
                "PREPROC-002",
                format!("Circular COPY dependency detected: {}", copy_file.display()),
                current_file.to_path_buf(),
                line,
                1,
            ));
            return Ok(String::new());
        }

        // Read and process the copy file
        let copy_content = std::fs::read_to_string(&copy_file)
            .context(format!("Failed to read COPY file: {}", copy_file.display()))?;

        self.processed_files.push(copy_file.clone());
        
        // Recursively process the copied content
        let mut nested_preprocessor = Preprocessor::new();
        nested_preprocessor.copy_paths = self.copy_paths.clone();
        nested_preprocessor.defines = self.defines.clone();
        nested_preprocessor.processed_files = self.processed_files.clone();
        
        let processed = nested_preprocessor.process(&copy_content, &copy_file)?;
        
        // Merge diagnostics
        for diag in processed.diagnostics {
            self.diagnostics.add(diag);
        }

        self.processed_files.pop();

        Ok(processed.content)
    }

    fn process_replace(&mut self, statement: &str) -> Result<()> {
        let upper = statement.to_uppercase();
        
        if upper.contains("REPLACE OFF") {
            self.defines.clear();
            return Ok(());
        }

        // REPLACE ==old== BY ==new==
        // Simplified implementation
        if let Some(by_pos) = upper.find(" BY ") {
            let before = statement[..by_pos].trim();
            let after = statement[by_pos + 4..].trim();
            
            // Extract patterns (simplified - should handle == delimiters properly)
            let old_pattern = before.replace("REPLACE", "").trim().to_string();
            let new_pattern = after.trim_end_matches('.').trim().to_string();
            
            self.defines.insert(old_pattern, new_pattern);
        }

        Ok(())
    }

    fn apply_replacements(&self, line: &str) -> String {
        let mut result = line.to_string();
        
        for (pattern, replacement) in &self.defines {
            result = result.replace(pattern, replacement);
        }
        
        result
    }
}

impl Default for Preprocessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_directive() {
        let mut preprocessor = Preprocessor::new();
        let source = ">>DEFINE MAX-SIZE AS 100\n       MOVE MAX-SIZE TO COUNTER.";
        let result = preprocessor.process(source, Path::new("test.cbl")).unwrap();
        
        assert!(result.content.contains("100"));
    }

    #[test]
    fn test_replace_statement() {
        let mut preprocessor = Preprocessor::new();
        preprocessor.process_replace("REPLACE ==OLD== BY ==NEW==").unwrap();
        
        let line = "This is OLD text";
        let result = preprocessor.apply_replacements(line);
        
        assert!(result.contains("NEW"));
    }
}

// Made with Bob

use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub end_line: Option<usize>,
    pub end_column: Option<usize>,
    pub construct: Option<String>,
}

impl Diagnostic {
    pub fn error(code: &str, message: String, file: PathBuf, line: usize, column: usize) -> Self {
        Self {
            severity: Severity::Error,
            code: code.to_string(),
            message,
            file,
            line,
            column,
            end_line: None,
            end_column: None,
            construct: None,
        }
    }

    pub fn warning(code: &str, message: String, file: PathBuf, line: usize, column: usize) -> Self {
        Self {
            severity: Severity::Warning,
            code: code.to_string(),
            message,
            file,
            line,
            column,
            end_line: None,
            end_column: None,
            construct: None,
        }
    }

    pub fn with_span(mut self, end_line: usize, end_column: usize) -> Self {
        self.end_line = Some(end_line);
        self.end_column = Some(end_column);
        self
    }

    pub fn with_construct(mut self, construct: String) -> Self {
        self.construct = Some(construct);
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity_str = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };

        write!(
            f,
            "{}:{}:{}: {}: [{}] {}",
            self.file.display(),
            self.line,
            self.column,
            severity_str,
            self.code,
            self.message
        )?;

        if let Some(construct) = &self.construct {
            write!(f, " (construct: {})", construct)?;
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Error)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Warning)
    }

    pub fn all(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

// Made with Bob

//! wasm-bindgen wrapper around the `cobol-transformer` library.
//!
//! Every exported function runs the real Rust pipeline from
//! `cobol-transformer/src` (preprocessor -> lexer -> parser -> symbol table /
//! CFG / transform passes / code generator) and returns a JSON string so the
//! browser can render results without any COBOL logic living in JavaScript.
//!
//! Response shape (all functions except `version`):
//! {
//!   "ok": bool,
//!   "operation": "parse" | "tokens" | "ast" | "symbols" | "cfg" | "generate"
//!                | "round-trip" | "detect-format" | "normalize-format",
//!   "output": String,          // text exactly as the CLI would print it
//!   "kind": "text" | "debug" | "cobol",
//!   "error": null | { "stage", "message", "line", "column" },
//!   "diagnostics": [ { "severity", "code", "message", "line", "column" } ],
//!   "stats": { "tokens", "program", "format", "round_trip" },
//!   "tokens": null | [ { "kind", "lexeme", "line", "column" } ]
//! }

use std::path::{Path, PathBuf};

use cobol_transformer::ast::CobolProgram;
use cobol_transformer::cfg::ControlFlowGraphBuilder;
use cobol_transformer::codegen::CodeGenerator;
use cobol_transformer::diagnostics::{Diagnostic, Severity};
use cobol_transformer::format::{FormatNormalizer, SourceFormat};
use cobol_transformer::lexer::{Lexer, Token};
use cobol_transformer::parser::Parser;
use cobol_transformer::preprocessor::{PreprocessedSource, Preprocessor};
use cobol_transformer::source_map::SourceMap;
use cobol_transformer::symbol_table::SymbolTableBuilder;
use cobol_transformer::transform::TransformEngine;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Transform passes the crate's `TransformEngine::apply_pass` recognises.
const KNOWN_PASSES: &[&str] = &["normalize", "modernize"];

#[derive(Serialize, Default)]
struct ErrorInfo {
    stage: &'static str,
    message: String,
    line: Option<usize>,
    column: Option<usize>,
}

#[derive(Serialize)]
struct DiagOut {
    severity: &'static str,
    code: String,
    message: String,
    line: usize,
    column: usize,
}

#[derive(Serialize, Default)]
struct Stats {
    tokens: Option<usize>,
    program: Option<String>,
    format: Option<&'static str>,
    round_trip: Option<bool>,
}

#[derive(Serialize)]
struct TokenOut {
    kind: String,
    lexeme: String,
    line: usize,
    column: usize,
}

#[derive(Serialize)]
struct Response {
    ok: bool,
    operation: &'static str,
    output: String,
    kind: &'static str,
    error: Option<ErrorInfo>,
    diagnostics: Vec<DiagOut>,
    stats: Stats,
    tokens: Option<Vec<TokenOut>>,
}

impl Response {
    fn new(operation: &'static str) -> Self {
        Self {
            ok: true,
            operation,
            output: String::new(),
            kind: "text",
            error: None,
            diagnostics: Vec::new(),
            stats: Stats::default(),
            tokens: None,
        }
    }

    fn fail(mut self, err: ErrorInfo) -> Self {
        self.ok = false;
        self.error = Some(err);
        self
    }

    fn json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|e| {
            format!(
                "{{\"ok\":false,\"operation\":\"{}\",\"output\":\"\",\"kind\":\"text\",\"error\":{{\"stage\":\"serialize\",\"message\":{:?},\"line\":null,\"column\":null}},\"diagnostics\":[],\"stats\":{{}},\"tokens\":null}}",
                self.operation, e.to_string()
            )
        })
    }
}

/// Options accepted by `run` (all optional).
#[derive(Deserialize, Default)]
#[serde(default)]
struct Options {
    /// File name reported in diagnostics and passed to the preprocessor.
    file_name: Option<String>,
    /// Transform passes for `generate` (applied in order).
    passes: Vec<String>,
    /// "auto" | "fixed" | "free" for `normalize-format`, and for the
    /// optional pre-lex normalisation step.
    format: Option<String>,
    /// Run `FormatNormalizer` before the preprocessor. The CLI does not do
    /// this; it is exposed here so fixed-format sources with sequence
    /// numbers in columns 1-6 can be tried.
    normalize_first: bool,
}

struct Front {
    pre: PreprocessedSource,
    token_count: usize,
    program: CobolProgram,
}

fn severity_str(s: &Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

fn diag_out(d: &Diagnostic) -> DiagOut {
    DiagOut {
        severity: severity_str(&d.severity),
        code: d.code.clone(),
        message: d.message.clone(),
        line: d.line,
        column: d.column,
    }
}

/// Map a line in the preprocessed text back to the original source line.
fn original_line(map: &SourceMap, expanded_line: usize) -> usize {
    map.all_mappings()
        .iter()
        .find(|m| m.expanded_line == expanded_line)
        .map(|m| m.original_line)
        .unwrap_or(expanded_line)
}

/// The lexer reports errors as "... at LINE:COL"; pull the position out.
fn lexer_position(message: &str) -> Option<(usize, usize)> {
    let idx = message.rfind(" at ")?;
    let tail = &message[idx + 4..];
    let mut parts = tail.trim().splitn(2, ':');
    let line = parts.next()?.trim().parse().ok()?;
    let column = parts.next()?.trim().parse().ok()?;
    Some((line, column))
}

fn resolve_format(name: Option<&str>, source: &str) -> SourceFormat {
    match name.map(|s| s.to_ascii_lowercase()) {
        Some(ref s) if s == "fixed" => SourceFormat::Fixed,
        Some(ref s) if s == "free" => SourceFormat::Free,
        _ => FormatNormalizer::detect_format(source),
    }
}

fn format_name(f: SourceFormat) -> &'static str {
    match f {
        SourceFormat::Fixed => "fixed",
        SourceFormat::Free => "free",
    }
}

/// Preprocess -> lex -> parse, exactly as `cobol-transform` does in main.rs.
fn front_end(source: &str, opts: &Options, resp: &mut Response) -> Result<Front, ErrorInfo> {
    let file = PathBuf::from(opts.file_name.clone().unwrap_or_else(|| "input.cob".into()));

    let normalized;
    let text: &str = if opts.normalize_first {
        let fmt = resolve_format(opts.format.as_deref(), source);
        resp.stats.format = Some(format_name(fmt));
        normalized = FormatNormalizer::new(fmt).normalize(source).map_err(|e| ErrorInfo {
            stage: "format",
            message: format!("{:#}", e),
            ..Default::default()
        })?;
        &normalized
    } else {
        source
    };

    let mut pre = Preprocessor::new();
    let pre = pre.process(text, Path::new(&file)).map_err(|e| ErrorInfo {
        stage: "preprocess",
        message: format!("{:#}", e),
        ..Default::default()
    })?;
    resp.diagnostics.extend(pre.diagnostics.iter().map(diag_out));

    let mut lexer = Lexer::new(&pre.content);
    let tokens: Vec<Token> = lexer.tokenize().map_err(|e| {
        let message = format!("{:#}", e);
        let pos = lexer_position(&message);
        ErrorInfo {
            stage: "lex",
            line: pos.map(|(l, _)| original_line(&pre.source_map, l)),
            column: pos.map(|(_, c)| c),
            message,
        }
    })?;
    let token_count = tokens.len();
    resp.stats.tokens = Some(token_count);

    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            let loc = parser.current_location();
            return Err(ErrorInfo {
                stage: "parse",
                message: format!("{:#}", e),
                line: Some(original_line(&pre.source_map, loc.line)),
                column: Some(loc.column),
            });
        }
    };
    resp.stats.program = Some(program.name().to_string());

    Ok(Front { pre, token_count, program })
}

fn parse_options(options_json: &str) -> Result<Options, ErrorInfo> {
    if options_json.trim().is_empty() {
        return Ok(Options::default());
    }
    serde_json::from_str(options_json).map_err(|e| ErrorInfo {
        stage: "options",
        message: format!("invalid options JSON: {}", e),
        ..Default::default()
    })
}

fn op_parse(source: &str, opts: &Options) -> Response {
    let mut resp = Response::new("parse");
    match front_end(source, opts, &mut resp) {
        Ok(f) => {
            resp.output = format!(
                "Lexed {} tokens\nParsed program: {}\nParse successful!\n",
                f.token_count,
                f.program.name()
            );
            let _ = f.pre;
            resp
        }
        Err(e) => resp.fail(e),
    }
}

fn op_tokens(source: &str, opts: &Options) -> Response {
    let mut resp = Response::new("tokens");
    let file = PathBuf::from(opts.file_name.clone().unwrap_or_else(|| "input.cob".into()));
    let mut pre = Preprocessor::new();
    let pre = match pre.process(source, Path::new(&file)) {
        Ok(p) => p,
        Err(e) => {
            return resp.fail(ErrorInfo {
                stage: "preprocess",
                message: format!("{:#}", e),
                ..Default::default()
            })
        }
    };
    resp.diagnostics.extend(pre.diagnostics.iter().map(diag_out));
    let mut lexer = Lexer::new(&pre.content);
    match lexer.tokenize() {
        Ok(tokens) => {
            resp.kind = "debug";
            resp.stats.tokens = Some(tokens.len());
            // Same text the CLI prints with --diagnostics.
            resp.output = tokens.iter().map(|t| format!("{:?}\n", t)).collect();
            resp.tokens = Some(
                tokens
                    .iter()
                    .map(|t| TokenOut {
                        kind: format!("{:?}", t.kind),
                        lexeme: t.lexeme.clone(),
                        line: original_line(&pre.source_map, t.location.line),
                        column: t.location.column,
                    })
                    .collect(),
            );
            resp
        }
        Err(e) => {
            let message = format!("{:#}", e);
            let pos = lexer_position(&message);
            resp.fail(ErrorInfo {
                stage: "lex",
                line: pos.map(|(l, _)| original_line(&pre.source_map, l)),
                column: pos.map(|(_, c)| c),
                message,
            })
        }
    }
}

fn op_ast(source: &str, opts: &Options) -> Response {
    let mut resp = Response::new("ast");
    match front_end(source, opts, &mut resp) {
        Ok(f) => {
            resp.kind = "debug";
            resp.output = format!("{:#?}", f.program);
            resp
        }
        Err(e) => resp.fail(e),
    }
}

fn op_symbols(source: &str, opts: &Options) -> Response {
    let mut resp = Response::new("symbols");
    match front_end(source, opts, &mut resp) {
        Ok(f) => match SymbolTableBuilder::new().build(&f.program) {
            Ok(table) => {
                resp.kind = "debug";
                resp.output = format!("{:#?}", table);
                resp
            }
            Err(e) => resp.fail(ErrorInfo {
                stage: "symbols",
                message: format!("{:#}", e),
                ..Default::default()
            }),
        },
        Err(e) => resp.fail(e),
    }
}

fn op_cfg(source: &str, opts: &Options) -> Response {
    let mut resp = Response::new("cfg");
    match front_end(source, opts, &mut resp) {
        Ok(f) => match ControlFlowGraphBuilder::new().build(&f.program) {
            Ok(cfg) => {
                resp.kind = "debug";
                resp.output = format!("{:#?}", cfg);
                resp
            }
            Err(e) => resp.fail(ErrorInfo {
                stage: "cfg",
                message: format!("{:#}", e),
                ..Default::default()
            }),
        },
        Err(e) => resp.fail(e),
    }
}

fn op_generate(source: &str, opts: &Options) -> Response {
    let mut resp = Response::new("generate");
    let mut f = match front_end(source, opts, &mut resp) {
        Ok(f) => f,
        Err(e) => return resp.fail(e),
    };
    let symbols = match SymbolTableBuilder::new().build(&f.program) {
        Ok(s) => s,
        Err(e) => {
            return resp.fail(ErrorInfo {
                stage: "symbols",
                message: format!("{:#}", e),
                ..Default::default()
            })
        }
    };
    let mut engine = TransformEngine::new();
    for pass in &opts.passes {
        if let Err(e) = engine.apply_pass(pass, &mut f.program, &symbols) {
            return resp.fail(ErrorInfo {
                stage: "transform",
                message: format!("pass {}: {:#}", pass, e),
                ..Default::default()
            });
        }
    }
    match CodeGenerator::new().generate(&f.program) {
        Ok(code) => {
            resp.kind = "cobol";
            resp.output = code;
            resp
        }
        Err(e) => resp.fail(ErrorInfo {
            stage: "codegen",
            message: format!("{:#}", e),
            ..Default::default()
        }),
    }
}

/// Mirrors `round_trip_command` in main.rs: parse, generate, re-lex and
/// re-parse the generated code, then compare with
/// `CobolProgram::semantically_equivalent`.
fn op_round_trip(source: &str, opts: &Options) -> Response {
    let mut resp = Response::new("round-trip");
    let f = match front_end(source, opts, &mut resp) {
        Ok(f) => f,
        Err(e) => return resp.fail(e),
    };
    let generated = match CodeGenerator::new().generate(&f.program) {
        Ok(g) => g,
        Err(e) => {
            return resp.fail(ErrorInfo {
                stage: "codegen",
                message: format!("{:#}", e),
                ..Default::default()
            })
        }
    };
    let tokens2 = match Lexer::new(&generated).tokenize() {
        Ok(t) => t,
        Err(e) => {
            let message = format!("{:#}", e);
            let pos = lexer_position(&message);
            return resp.fail(ErrorInfo {
                stage: "round-trip lex",
                line: pos.map(|(l, _)| l),
                column: pos.map(|(_, c)| c),
                message: format!("{} (in generated code)", message),
            });
        }
    };
    let mut parser2 = Parser::new(tokens2);
    let program2 = match parser2.parse() {
        Ok(p) => p,
        Err(e) => {
            let loc = parser2.current_location();
            return resp.fail(ErrorInfo {
                stage: "round-trip parse",
                message: format!("{:#} (in generated code)", e),
                line: Some(loc.line),
                column: Some(loc.column),
            });
        }
    };
    let passed = f.program.semantically_equivalent(&program2);
    resp.stats.round_trip = Some(passed);
    resp.kind = "text";
    resp.output = format!(
        "Round-trip validation {}\n\n--- generated code (second-pass input) ---\n{}",
        if passed { "PASSED" } else { "FAILED" },
        generated
    );
    resp
}

fn op_detect_format(source: &str) -> Response {
    let mut resp = Response::new("detect-format");
    let fmt = FormatNormalizer::detect_format(source);
    resp.stats.format = Some(format_name(fmt));
    resp.output = format!("{}\n", format_name(fmt));
    resp
}

fn op_normalize_format(source: &str, opts: &Options) -> Response {
    let mut resp = Response::new("normalize-format");
    let fmt = resolve_format(opts.format.as_deref(), source);
    resp.stats.format = Some(format_name(fmt));
    match FormatNormalizer::new(fmt).normalize(source) {
        Ok(out) => {
            resp.kind = "cobol";
            resp.output = out;
            resp
        }
        Err(e) => resp.fail(ErrorInfo {
            stage: "format",
            message: format!("{:#}", e),
            ..Default::default()
        }),
    }
}

fn dispatch(operation: &str, source: &str, opts: &Options) -> Response {
    match operation {
        "parse" => op_parse(source, opts),
        "tokens" => op_tokens(source, opts),
        "ast" => op_ast(source, opts),
        "symbols" => op_symbols(source, opts),
        "cfg" => op_cfg(source, opts),
        "generate" => op_generate(source, opts),
        "round-trip" => op_round_trip(source, opts),
        "detect-format" => op_detect_format(source),
        "normalize-format" => op_normalize_format(source, opts),
        _ => Response::new("parse").fail(ErrorInfo {
            stage: "options",
            message: format!("unknown operation: {}", operation),
            ..Default::default()
        }),
    }
}

// ---------------------------------------------------------------------------
// Exported API
// ---------------------------------------------------------------------------

/// Crate versions: `{"wrapper": "...", "passes": [...], "operations": [...]}`.
#[wasm_bindgen]
pub fn version() -> String {
    serde_json::json!({
        "wrapper": env!("CARGO_PKG_VERSION"),
        "passes": KNOWN_PASSES,
        "operations": ["parse", "tokens", "ast", "symbols", "cfg", "generate",
                        "round-trip", "detect-format", "normalize-format"],
    })
    .to_string()
}

/// Generic entry point: `operation` is one of the names listed by
/// `version()`, `options_json` is an optional JSON object
/// (`file_name`, `passes`, `format`, `normalize_first`).
#[wasm_bindgen]
pub fn run(operation: &str, source: &str, options_json: &str) -> String {
    match parse_options(options_json) {
        Ok(opts) => dispatch(operation, source, &opts).json(),
        Err(e) => Response::new("parse").fail(e).json(),
    }
}

/// `cobol-transform parse FILE`
#[wasm_bindgen]
pub fn parse(source: &str) -> String {
    op_parse(source, &Options::default()).json()
}

/// `cobol-transform FILE --diagnostics` (token stream)
#[wasm_bindgen]
pub fn tokens(source: &str) -> String {
    op_tokens(source, &Options::default()).json()
}

/// `cobol-transform FILE --dump-ast`
#[wasm_bindgen]
pub fn dump_ast(source: &str) -> String {
    op_ast(source, &Options::default()).json()
}

/// `cobol-transform FILE --dump-symbols`
#[wasm_bindgen]
pub fn dump_symbols(source: &str) -> String {
    op_symbols(source, &Options::default()).json()
}

/// `cobol-transform FILE --dump-cfg`
#[wasm_bindgen]
pub fn dump_cfg(source: &str) -> String {
    op_cfg(source, &Options::default()).json()
}

/// `cobol-transform transform FILE -p PASS...` (passes comma-separated)
#[wasm_bindgen]
pub fn generate(source: &str, passes: &str) -> String {
    let opts = Options {
        passes: passes
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        ..Default::default()
    };
    op_generate(source, &opts).json()
}

/// `cobol-transform round-trip FILE`
#[wasm_bindgen]
pub fn round_trip(source: &str) -> String {
    op_round_trip(source, &Options::default()).json()
}

/// `FormatNormalizer::detect_format`
#[wasm_bindgen]
pub fn detect_format(source: &str) -> String {
    op_detect_format(source).json()
}

/// `FormatNormalizer::normalize` with format "auto" | "fixed" | "free"
#[wasm_bindgen]
pub fn normalize_format(source: &str, format: &str) -> String {
    let opts = Options {
        format: Some(format.to_string()),
        ..Default::default()
    };
    op_normalize_format(source, &opts).json()
}

mod lexer;
mod parser;
mod ast;
mod preprocessor;
mod symbol_table;
mod type_system;
mod cfg;
mod dataflow;
mod transform;
mod codegen;
mod diagnostics;
mod source_map;
mod format;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "cobol-transform")]
#[command(about = "COBOL source-to-source transformer", long_about = None)]
struct Cli {
    /// Input COBOL source file
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,

    /// Output file path
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Output format (fixed or free)
    #[arg(long, value_name = "FORMAT")]
    format: Option<String>,

    /// Transformation to apply
    #[arg(long, value_name = "TRANSFORM")]
    transform: Option<String>,

    /// Dump AST to stdout
    #[arg(long)]
    dump_ast: bool,

    /// Dump symbol table to stdout
    #[arg(long)]
    dump_symbols: bool,

    /// Dump control flow graph to stdout
    #[arg(long)]
    dump_cfg: bool,

    /// Show diagnostics
    #[arg(long)]
    diagnostics: bool,

    /// Perform round-trip validation
    #[arg(long)]
    round_trip: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and validate COBOL source
    Parse {
        /// Input file
        input: PathBuf,
    },
    /// Transform COBOL source
    Transform {
        /// Input file
        input: PathBuf,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Transformation passes
        #[arg(short, long)]
        passes: Vec<String>,
    },
    /// Validate round-trip transformation
    RoundTrip {
        /// Input file
        input: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Parse { input }) => {
            parse_command(&input)?;
        }
        Some(Commands::Transform { input, output, passes }) => {
            transform_command(&input, output.as_ref(), &passes)?;
        }
        Some(Commands::RoundTrip { input }) => {
            round_trip_command(&input)?;
        }
        None => {
            if let Some(ref input) = cli.input {
                default_command(&cli, input)?;
            } else {
                eprintln!("Error: No input file specified");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn parse_command(input: &PathBuf) -> Result<()> {
    use crate::preprocessor::Preprocessor;
    use crate::lexer::Lexer;
    use crate::parser::Parser as CobolParser;

    let source = std::fs::read_to_string(input)?;
    
    // Preprocess
    let mut preprocessor = Preprocessor::new();
    let preprocessed = preprocessor.process(&source, input)?;
    
    // Lex
    let mut lexer = Lexer::new(&preprocessed.content);
    let tokens = lexer.tokenize()?;
    
    println!("Lexed {} tokens", tokens.len());
    
    // Parse
    let mut parser = CobolParser::new(tokens);
    let program = parser.parse()?;
    
    println!("Parsed program: {}", program.name());
    println!("Parse successful!");
    
    Ok(())
}

fn transform_command(input: &PathBuf, output: Option<&PathBuf>, passes: &[String]) -> Result<()> {
    use crate::preprocessor::Preprocessor;
    use crate::lexer::Lexer;
    use crate::parser::Parser as CobolParser;
    use crate::symbol_table::SymbolTableBuilder;
    use crate::transform::TransformEngine;
    use crate::codegen::CodeGenerator;

    let source = std::fs::read_to_string(input)?;
    
    // Preprocess
    let mut preprocessor = Preprocessor::new();
    let preprocessed = preprocessor.process(&source, input)?;
    
    // Lex and parse
    let mut lexer = Lexer::new(&preprocessed.content);
    let tokens = lexer.tokenize()?;
    let mut parser = CobolParser::new(tokens);
    let mut program = parser.parse()?;
    
    // Build symbol table
    let mut symbol_builder = SymbolTableBuilder::new();
    let symbols = symbol_builder.build(&program)?;
    
    // Apply transformations
    let mut engine = TransformEngine::new();
    for pass_name in passes {
        engine.apply_pass(pass_name, &mut program, &symbols)?;
    }
    
    // Generate code
    let mut generator = CodeGenerator::new();
    let output_code = generator.generate(&program)?;
    
    // Write output
    if let Some(out_path) = output {
        std::fs::write(out_path, output_code)?;
        println!("Transformed code written to {}", out_path.display());
    } else {
        println!("{}", output_code);
    }
    
    Ok(())
}

fn round_trip_command(input: &PathBuf) -> Result<()> {
    use crate::preprocessor::Preprocessor;
    use crate::lexer::Lexer;
    use crate::parser::Parser as CobolParser;
    use crate::codegen::CodeGenerator;

    let source = std::fs::read_to_string(input)?;
    
    // First pass
    let mut preprocessor = Preprocessor::new();
    let preprocessed = preprocessor.process(&source, input)?;
    let mut lexer = Lexer::new(&preprocessed.content);
    let tokens = lexer.tokenize()?;
    let mut parser = CobolParser::new(tokens);
    let program1 = parser.parse()?;
    
    // Generate
    let mut generator = CodeGenerator::new();
    let generated = generator.generate(&program1)?;
    
    // Second pass
    let mut lexer2 = Lexer::new(&generated);
    let tokens2 = lexer2.tokenize()?;
    let mut parser2 = CobolParser::new(tokens2);
    let program2 = parser2.parse()?;
    
    // Compare
    if program1.semantically_equivalent(&program2) {
        println!("Round-trip validation PASSED");
        Ok(())
    } else {
        eprintln!("Round-trip validation FAILED");
        std::process::exit(1);
    }
}

fn default_command(cli: &Cli, input: &PathBuf) -> Result<()> {
    use crate::preprocessor::Preprocessor;
    use crate::lexer::Lexer;
    use crate::parser::Parser as CobolParser;
    use crate::symbol_table::SymbolTableBuilder;
    use crate::codegen::CodeGenerator;

    let source = std::fs::read_to_string(input)?;
    
    let mut preprocessor = Preprocessor::new();
    let preprocessed = preprocessor.process(&source, input)?;
    
    let mut lexer = Lexer::new(&preprocessed.content);
    let tokens = lexer.tokenize()?;
    
    if cli.diagnostics {
        for token in &tokens {
            println!("{:?}", token);
        }
    }
    
    let mut parser = CobolParser::new(tokens);
    let program = parser.parse()?;
    
    if cli.dump_ast {
        println!("{:#?}", program);
    }
    
    if cli.dump_symbols {
        let mut symbol_builder = SymbolTableBuilder::new();
        let symbols = symbol_builder.build(&program)?;
        println!("{:#?}", symbols);
    }
    
    if cli.dump_cfg {
        use crate::cfg::ControlFlowGraphBuilder;
        let mut cfg_builder = ControlFlowGraphBuilder::new();
        let cfg = cfg_builder.build(&program)?;
        println!("{:#?}", cfg);
    }
    
    if cli.round_trip {
        return round_trip_command(input);
    }
    
    // Default: generate output
    let mut generator = CodeGenerator::new();
    let output_code = generator.generate(&program)?;
    
    if let Some(output_path) = &cli.output {
        std::fs::write(output_path, output_code)?;
        println!("Output written to {}", output_path.display());
    } else {
        println!("{}", output_code);
    }
    
    Ok(())
}

// Made with Bob

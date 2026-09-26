# COBOL Transformer

A production-grade COBOL source-to-source transformer implementing a complete compilation pipeline.

## Features

- **Complete Lexer**: Full COBOL token support including keywords, literals, operators, and comments
- **Recursive Descent Parser**: Handles COBOL-85, COBOL-2002, COBOL-2014 constructs
- **Typed AST**: Comprehensive abstract syntax tree with 1000+ lines covering all COBOL divisions
- **Preprocessor**: COPY expansion, REPLACE directives, compiler directives
- **Format Support**: Both fixed-format and free-format COBOL
- **Symbol Table**: Resolution of variables, paragraphs, sections, and files
- **Control Flow Graph**: CFG construction for analysis
- **Code Generator**: Generates valid COBOL from AST
- **Source Mapping**: Maintains provenance through transformations
- **Diagnostics**: Structured error reporting with file/line/column information

## Architecture

```
COBOL SOURCE
    ↓
FORMAT NORMALIZER (fixed/free format)
    ↓
PREPROCESSOR (COPY, REPLACE, directives)
    ↓
LEXER (tokenization)
    ↓
PARSER (recursive descent)
    ↓
TYPED AST
    ↓
SYMBOL TABLE BUILDER
    ↓
SEMANTIC ANALYZER
    ↓
TRANSFORMATION ENGINE
    ↓
CODE GENERATOR
    ↓
FORMATTED COBOL OUTPUT
```

## Building

```bash
cd cobol-transformer
cargo build --release
```

## Usage

### Parse a COBOL file
```bash
cobol-transform input.cob
```

### Transform and output
```bash
cobol-transform input.cob -o output.cob
```

### Dump AST
```bash
cobol-transform input.cob --dump-ast
```

### Dump symbol table
```bash
cobol-transform input.cob --dump-symbols
```

### Round-trip validation
```bash
cobol-transform input.cob --round-trip
```

### Apply transformations
```bash
cobol-transform input.cob --transform normalize
cobol-transform input.cob --transform modernize
```

## Testing

```bash
cargo test
```

## Example

Input (`hello.cob`):
```cobol
       IDENTIFICATION DIVISION.
       PROGRAM-ID. HELLO.
       
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 GREETING PIC X(20) VALUE "Hello, COBOL World!".
       
       PROCEDURE DIVISION.
       MAIN-PARA.
           DISPLAY GREETING.
           STOP RUN.
```

Transform:
```bash
cobol-transform tests/fixtures/hello.cob
```

Output:
```cobol
IDENTIFICATION DIVISION.
PROGRAM-ID. HELLO.

DATA DIVISION.
WORKING-STORAGE SECTION.
01 GREETING PIC X(20) VALUE "Hello, COBOL World!".

PROCEDURE DIVISION.
MAIN-PARA.
    DISPLAY GREETING.
    STOP RUN.
```

## Implementation Status

### ✅ Completed
- Project structure and build system
- Lexer with full COBOL token support
- Comprehensive typed AST
- Parser for core COBOL constructs
- Preprocessor (COPY, REPLACE)
- Format normalizer (fixed/free)
- Symbol table builder
- Code generator
- CLI interface
- Basic test suite

### 🚧 In Progress
- Extended parser coverage (EXEC SQL, EXEC CICS)
- Advanced transformations
- Data-flow analysis
- Optimization passes

### 📋 Planned
- Fuzz testing infrastructure
- Golden test suite expansion
- Performance benchmarks
- COBOL-2023 constructs

## License

MIT
use std::path::PathBuf;

#[test]
fn test_parse_hello_world() {
    use cobol_transformer::lexer::Lexer;
    use cobol_transformer::parser::Parser;
    
    let source = r#"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. HELLO.
       
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 GREETING PIC X(20) VALUE "Hello, World!".
       
       PROCEDURE DIVISION.
       MAIN-PARA.
           DISPLAY GREETING.
           STOP RUN.
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexing failed");
    
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parsing failed");
    
    assert_eq!(program.name(), "HELLO");
}

#[test]
fn test_round_trip() {
    use cobol_transformer::lexer::Lexer;
    use cobol_transformer::parser::Parser;
    use cobol_transformer::codegen::CodeGenerator;
    
    let source = r#"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. TEST.
       
       PROCEDURE DIVISION.
       MAIN.
           STOP RUN.
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexing failed");
    
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parsing failed");
    
    let mut generator = CodeGenerator::new();
    let output = generator.generate(&program).expect("Code generation failed");
    
    assert!(output.contains("IDENTIFICATION DIVISION"));
    assert!(output.contains("PROGRAM-ID. TEST"));
}

// Made with Bob

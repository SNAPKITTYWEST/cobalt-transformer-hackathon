use crate::source_map::SourceLocation;
use anyhow::{Result, bail};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords - Division components
    Identification,
    Environment,
    Data,
    Procedure,
    Division,
    Program,
    
    // Keywords - Divisions (compound forms)
    IdentificationDivision,
    EnvironmentDivision,
    DataDivision,
    ProcedureDivision,
    
    // Keywords - Sections
    ConfigurationSection,
    InputOutputSection,
    FileSection,
    WorkingStorageSection,
    LocalStorageSection,
    LinkageSection,
    ReportSection,
    ScreenSection,
    
    // Keywords - Paragraphs
    ProgramId,
    Author,
    Installation,
    DateWritten,
    DateCompiled,
    Security,
    Remarks,
    SourceComputer,
    ObjectComputer,
    SpecialNames,
    FileControl,
    
    // Keywords - Statements
    Accept,
    Add,
    Alter,
    Call,
    Cancel,
    Close,
    Compute,
    Continue,
    Delete,
    Display,
    Divide,
    Evaluate,
    Else,
    Exec,
    Exit,
    Go,
    GoBack,
    If,
    Initialize,
    Inspect,
    Merge,
    Move,
    Multiply,
    Open,
    Perform,
    Read,
    Rewrite,
    Search,
    Set,
    Sort,
    Start,
    Stop,
    String,
    Subtract,
    Unstring,
    Write,
    
    // Keywords - Clauses
    To,
    From,
    By,
    Giving,
    Into,
    Using,
    Returning,
    Through,
    Thru,
    Until,
    Varying,
    After,
    Before,
    With,
    On,
    At,
    End,
    When,
    Also,
    Not,
    And,
    Or,
    
    // Keywords - Data Description
    Pic,
    Picture,
    Usage,
    Value,
    Values,
    Occurs,
    Redefines,
    Renames,
    Sign,
    Sync,
    Synchronized,
    Justified,
    Just,
    Blank,
    External,
    Global,
    Based,
    Depending,
    Indexed,
    Ascending,
    Descending,
    Key,
    
    // Keywords - File Operations
    Select,
    Assign,
    Organization,
    Access,
    Record,
    Alternate,
    Status,
    Sequential,
    Random,
    Dynamic,
    Relative,
    
    // Keywords - Conditions
    Equal,
    Greater,
    Less,
    Than,
    
    // Keywords - Special
    Is,
    Are,
    Of,
    In,
    
    // End markers
    EndIf,
    EndPerform,
    EndEvaluate,
    EndRead,
    EndWrite,
    EndRewrite,
    EndStart,
    EndDelete,
    EndSearch,
    EndString,
    EndUnstring,
    EndAdd,
    EndSubtract,
    EndMultiply,
    EndDivide,
    EndCompute,
    EndCall,
    
    // Figurative constants
    Zero,
    Zeros,
    Zeroes,
    Space,
    Spaces,
    HighValue,
    HighValues,
    LowValue,
    LowValues,
    Quote,
    Quotes,
    Null,
    Nulls,
    
    // Data types
    Comp,
    Comp1,
    Comp2,
    Comp3,
    Comp4,
    Comp5,
    Binary,
    PackedDecimal,
    DisplayKeyword,
    National,
    Index,
    Pointer,
    Object,
    Reference,
    
    // Literals
    NumericLiteral(String),
    AlphanumericLiteral(String),
    NationalLiteral(String),
    HexLiteral(String),
    BooleanLiteral(bool),
    
    // Identifiers
    Identifier(String),
    LevelNumber(u8),
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Power,
    LeftParen,
    RightParen,
    
    // Punctuation
    Period,
    Comma,
    Semicolon,
    Colon,
    
    // Special
    Eof,
    Newline,
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub location: SourceLocation,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, location: SourceLocation) -> Self {
        Self {
            kind,
            lexeme,
            location,
        }
    }
}

pub struct Lexer {
    source: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
    file: PathBuf,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            file: PathBuf::from("<input>"),
        }
    }

    pub fn with_file(source: &str, file: PathBuf) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            file,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();
            
            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;
            
            // Skip comments but keep them for documentation
            if matches!(token.kind, TokenKind::Comment(_)) {
                tokens.push(token);
                continue;
            }

            tokens.push(token);
        }

        tokens.push(Token::new(
            TokenKind::Eof,
            String::new(),
            self.current_location(),
        ));

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token> {
        let start_location = self.current_location();
        let c = self.advance();

        match c {
            '*' if self.column == 1 || self.peek_back() == Some('\n') => {
                // Check for *> inline comment
                if self.peek() == Some('>') {
                    self.advance(); // consume '>'
                    let comment = self.read_until_newline();
                    Ok(Token::new(
                        TokenKind::Comment(format!("*>{}", comment)),
                        format!("*>{}", comment),
                        start_location,
                    ))
                } else {
                    // Traditional * comment line
                    let comment = self.read_until_newline();
                    Ok(Token::new(
                        TokenKind::Comment(comment.clone()),
                        comment,
                        start_location,
                    ))
                }
            }
            '*' if self.peek() == Some('>') => {
                // *> inline comment anywhere in line
                self.advance(); // consume '>'
                let comment = self.read_until_newline();
                Ok(Token::new(
                    TokenKind::Comment(format!("*>{}", comment)),
                    format!("*>{}", comment),
                    start_location,
                ))
            }
            '+' => Ok(Token::new(TokenKind::Plus, "+".to_string(), start_location)),
            '-' => {
                if self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    self.read_number(true)
                } else {
                    Ok(Token::new(TokenKind::Minus, "-".to_string(), start_location))
                }
            }
            '*' => Ok(Token::new(TokenKind::Star, "*".to_string(), start_location)),
            '/' => Ok(Token::new(TokenKind::Slash, "/".to_string(), start_location)),
            '(' => Ok(Token::new(TokenKind::LeftParen, "(".to_string(), start_location)),
            ')' => Ok(Token::new(TokenKind::RightParen, ")".to_string(), start_location)),
            '.' => Ok(Token::new(TokenKind::Period, ".".to_string(), start_location)),
            ',' => Ok(Token::new(TokenKind::Comma, ",".to_string(), start_location)),
            ';' => Ok(Token::new(TokenKind::Semicolon, ";".to_string(), start_location)),
            ':' => Ok(Token::new(TokenKind::Colon, ":".to_string(), start_location)),
            '"' => self.read_string_literal('"'),
            '\'' => self.read_string_literal('\''),
            'X' | 'x' if self.peek() == Some('"') || self.peek() == Some('\'') => {
                self.advance(); // consume quote
                let quote = self.peek_back().unwrap();
                self.read_hex_literal(quote)
            }
            'N' | 'n' if self.peek() == Some('"') || self.peek() == Some('\'') => {
                self.advance(); // consume quote
                let quote = self.peek_back().unwrap();
                self.read_national_literal(quote)
            }
            c if c.is_ascii_digit() => self.read_number(false),
            c if c.is_alphabetic() || c == '_' => self.read_identifier_or_keyword(),
            '\n' => {
                self.line += 1;
                self.column = 1;
                Ok(Token::new(TokenKind::Newline, "\n".to_string(), start_location))
            }
            _ => bail!("Unexpected character: {} at {}:{}", c, self.line, self.column),
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Result<Token> {
        let start_location = self.current_location();
        let mut lexeme = String::new();
        lexeme.push(self.peek_back().unwrap());

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                lexeme.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let upper = lexeme.to_uppercase();
        let kind = self.keyword_or_identifier(&upper, &lexeme);

        Ok(Token::new(kind, lexeme, start_location))
    }

    fn keyword_or_identifier(&self, upper: &str, lexeme: &str) -> TokenKind {
        match upper.as_ref() {
            // Division components
            "IDENTIFICATION" | "ID" => TokenKind::Identification,
            "ENVIRONMENT" => TokenKind::Environment,
            "DATA" => TokenKind::Data,
            "PROCEDURE" => TokenKind::Procedure,
            "DIVISION" => TokenKind::Division,
            "PROGRAM" => TokenKind::Program,
            
            // Divisions (compound forms with hyphens)
            "IDENTIFICATION-DIVISION" | "ID-DIVISION" => TokenKind::IdentificationDivision,
            "ENVIRONMENT-DIVISION" => TokenKind::EnvironmentDivision,
            "DATA-DIVISION" => TokenKind::DataDivision,
            "PROCEDURE-DIVISION" => TokenKind::ProcedureDivision,
            
            // Sections
            "CONFIGURATION" => TokenKind::ConfigurationSection,
            "INPUT-OUTPUT" => TokenKind::InputOutputSection,
            "FILE" => TokenKind::FileSection,
            "WORKING-STORAGE" => TokenKind::WorkingStorageSection,
            "LOCAL-STORAGE" => TokenKind::LocalStorageSection,
            "LINKAGE" => TokenKind::LinkageSection,
            "REPORT" => TokenKind::ReportSection,
            "SCREEN" => TokenKind::ScreenSection,
            
            // Paragraphs
            "PROGRAM-ID" => TokenKind::ProgramId,
            "AUTHOR" => TokenKind::Author,
            "INSTALLATION" => TokenKind::Installation,
            "DATE-WRITTEN" => TokenKind::DateWritten,
            "DATE-COMPILED" => TokenKind::DateCompiled,
            "SECURITY" => TokenKind::Security,
            "REMARKS" => TokenKind::Remarks,
            "SOURCE-COMPUTER" => TokenKind::SourceComputer,
            "OBJECT-COMPUTER" => TokenKind::ObjectComputer,
            "SPECIAL-NAMES" => TokenKind::SpecialNames,
            "FILE-CONTROL" => TokenKind::FileControl,
            
            // Statements
            "ACCEPT" => TokenKind::Accept,
            "ADD" => TokenKind::Add,
            "ALTER" => TokenKind::Alter,
            "CALL" => TokenKind::Call,
            "CANCEL" => TokenKind::Cancel,
            "CLOSE" => TokenKind::Close,
            "COMPUTE" => TokenKind::Compute,
            "CONTINUE" => TokenKind::Continue,
            "DELETE" => TokenKind::Delete,
            "DISPLAY" => TokenKind::Display,
            "DIVIDE" => TokenKind::Divide,
            "EVALUATE" => TokenKind::Evaluate,
            "ELSE" => TokenKind::Else,
            "EXEC" => TokenKind::Exec,
            "EXIT" => TokenKind::Exit,
            "GO" => TokenKind::Go,
            "GOBACK" => TokenKind::GoBack,
            "IF" => TokenKind::If,
            "INITIALIZE" => TokenKind::Initialize,
            "INSPECT" => TokenKind::Inspect,
            "MERGE" => TokenKind::Merge,
            "MOVE" => TokenKind::Move,
            "MULTIPLY" => TokenKind::Multiply,
            "OPEN" => TokenKind::Open,
            "PERFORM" => TokenKind::Perform,
            "READ" => TokenKind::Read,
            "REWRITE" => TokenKind::Rewrite,
            "SEARCH" => TokenKind::Search,
            "SET" => TokenKind::Set,
            "SORT" => TokenKind::Sort,
            "START" => TokenKind::Start,
            "STOP" => TokenKind::Stop,
            "STRING" => TokenKind::String,
            "SUBTRACT" => TokenKind::Subtract,
            "UNSTRING" => TokenKind::Unstring,
            "WRITE" => TokenKind::Write,
            
            // Clauses
            "TO" => TokenKind::To,
            "FROM" => TokenKind::From,
            "BY" => TokenKind::By,
            "GIVING" => TokenKind::Giving,
            "INTO" => TokenKind::Into,
            "USING" => TokenKind::Using,
            "RETURNING" => TokenKind::Returning,
            "THROUGH" | "THRU" => TokenKind::Thru,
            "UNTIL" => TokenKind::Until,
            "VARYING" => TokenKind::Varying,
            "AFTER" => TokenKind::After,
            "BEFORE" => TokenKind::Before,
            "WITH" => TokenKind::With,
            "ON" => TokenKind::On,
            "AT" => TokenKind::At,
            "END" => TokenKind::End,
            "WHEN" => TokenKind::When,
            "ALSO" => TokenKind::Also,
            "NOT" => TokenKind::Not,
            "AND" => TokenKind::And,
            "OR" => TokenKind::Or,
            
            // Data Description
            "PIC" | "PICTURE" => TokenKind::Picture,
            "USAGE" => TokenKind::Usage,
            "VALUE" => TokenKind::Value,
            "VALUES" => TokenKind::Values,
            "OCCURS" => TokenKind::Occurs,
            "REDEFINES" => TokenKind::Redefines,
            "RENAMES" => TokenKind::Renames,
            "SIGN" => TokenKind::Sign,
            "SYNC" | "SYNCHRONIZED" => TokenKind::Synchronized,
            "JUSTIFIED" | "JUST" => TokenKind::Justified,
            "BLANK" => TokenKind::Blank,
            "EXTERNAL" => TokenKind::External,
            "GLOBAL" => TokenKind::Global,
            "BASED" => TokenKind::Based,
            "DEPENDING" => TokenKind::Depending,
            "INDEXED" => TokenKind::Indexed,
            "ASCENDING" => TokenKind::Ascending,
            "DESCENDING" => TokenKind::Descending,
            "KEY" => TokenKind::Key,
            
            // File Operations
            "SELECT" => TokenKind::Select,
            "ASSIGN" => TokenKind::Assign,
            "ORGANIZATION" => TokenKind::Organization,
            "ACCESS" => TokenKind::Access,
            "RECORD" => TokenKind::Record,
            "ALTERNATE" => TokenKind::Alternate,
            "STATUS" => TokenKind::Status,
            "SEQUENTIAL" => TokenKind::Sequential,
            "RANDOM" => TokenKind::Random,
            "DYNAMIC" => TokenKind::Dynamic,
            "RELATIVE" => TokenKind::Relative,
            
            // Conditions
            "EQUAL" => TokenKind::Equal,
            "GREATER" => TokenKind::Greater,
            "LESS" => TokenKind::Less,
            "THAN" => TokenKind::Than,
            
            // Special
            "IS" => TokenKind::Is,
            "ARE" => TokenKind::Are,
            "OF" => TokenKind::Of,
            "IN" => TokenKind::In,
            
            // End markers
            "END-IF" => TokenKind::EndIf,
            "END-PERFORM" => TokenKind::EndPerform,
            "END-EVALUATE" => TokenKind::EndEvaluate,
            "END-READ" => TokenKind::EndRead,
            "END-WRITE" => TokenKind::EndWrite,
            "END-REWRITE" => TokenKind::EndRewrite,
            "END-START" => TokenKind::EndStart,
            "END-DELETE" => TokenKind::EndDelete,
            "END-SEARCH" => TokenKind::EndSearch,
            "END-STRING" => TokenKind::EndString,
            "END-UNSTRING" => TokenKind::EndUnstring,
            "END-ADD" => TokenKind::EndAdd,
            "END-SUBTRACT" => TokenKind::EndSubtract,
            "END-MULTIPLY" => TokenKind::EndMultiply,
            "END-DIVIDE" => TokenKind::EndDivide,
            "END-COMPUTE" => TokenKind::EndCompute,
            "END-CALL" => TokenKind::EndCall,
            
            // Figurative constants
            "ZERO" => TokenKind::Zero,
            "ZEROS" | "ZEROES" => TokenKind::Zeros,
            "SPACE" => TokenKind::Space,
            "SPACES" => TokenKind::Spaces,
            "HIGH-VALUE" => TokenKind::HighValue,
            "HIGH-VALUES" => TokenKind::HighValues,
            "LOW-VALUE" => TokenKind::LowValue,
            "LOW-VALUES" => TokenKind::LowValues,
            "QUOTE" => TokenKind::Quote,
            "QUOTES" => TokenKind::Quotes,
            "NULL" => TokenKind::Null,
            "NULLS" => TokenKind::Nulls,
            
            // Data types
            "COMP" | "COMPUTATIONAL" => TokenKind::Comp,
            "COMP-1" | "COMPUTATIONAL-1" => TokenKind::Comp1,
            "COMP-2" | "COMPUTATIONAL-2" => TokenKind::Comp2,
            "COMP-3" | "COMPUTATIONAL-3" => TokenKind::Comp3,
            "COMP-4" | "COMPUTATIONAL-4" => TokenKind::Comp4,
            "COMP-5" | "COMPUTATIONAL-5" => TokenKind::Comp5,
            "BINARY" => TokenKind::Binary,
            "PACKED-DECIMAL" => TokenKind::PackedDecimal,
            "NATIONAL" => TokenKind::National,
            "INDEX" => TokenKind::Index,
            "POINTER" => TokenKind::Pointer,
            "OBJECT" => TokenKind::Object,
            "REFERENCE" => TokenKind::Reference,
            
            // Level numbers
            s if s.chars().all(|c| c.is_ascii_digit()) && s.len() <= 2 => {
                if let Ok(level) = s.parse::<u8>() {
                    if level <= 88 {
                        return TokenKind::LevelNumber(level);
                    }
                }
                TokenKind::Identifier(lexeme.to_string())
            }
            
            _ => TokenKind::Identifier(lexeme.to_string()),
        }
    }

    fn read_number(&mut self, negative: bool) -> Result<Token> {
        let start_location = self.current_location();
        let mut lexeme = String::new();
        
        if negative {
            lexeme.push('-');
        } else {
            lexeme.push(self.peek_back().unwrap());
        }

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                lexeme.push(c);
                self.advance();
            } else {
                break;
            }
        }

        Ok(Token::new(
            TokenKind::NumericLiteral(lexeme.clone()),
            lexeme,
            start_location,
        ))
    }

    fn read_string_literal(&mut self, quote: char) -> Result<Token> {
        let start_location = self.current_location();
        let mut lexeme = String::new();
        lexeme.push(quote);

        while let Some(c) = self.peek() {
            if c == quote {
                lexeme.push(c);
                self.advance();
                
                // Check for doubled quote (escape)
                if self.peek() == Some(quote) {
                    lexeme.push(quote);
                    self.advance();
                } else {
                    break;
                }
            } else {
                lexeme.push(c);
                self.advance();
            }
        }

        Ok(Token::new(
            TokenKind::AlphanumericLiteral(lexeme.clone()),
            lexeme,
            start_location,
        ))
    }

    fn read_hex_literal(&mut self, quote: char) -> Result<Token> {
        let start_location = self.current_location();
        let mut lexeme = format!("X{}", quote);

        while let Some(c) = self.peek() {
            if c == quote {
                lexeme.push(c);
                self.advance();
                break;
            } else if c.is_ascii_hexdigit() {
                lexeme.push(c);
                self.advance();
            } else {
                bail!("Invalid hex literal at {}:{}", self.line, self.column);
            }
        }

        Ok(Token::new(
            TokenKind::HexLiteral(lexeme.clone()),
            lexeme,
            start_location,
        ))
    }

    fn read_national_literal(&mut self, quote: char) -> Result<Token> {
        let start_location = self.current_location();
        let mut lexeme = format!("N{}", quote);

        while let Some(c) = self.peek() {
            if c == quote {
                lexeme.push(c);
                self.advance();
                break;
            } else {
                lexeme.push(c);
                self.advance();
            }
        }

        Ok(Token::new(
            TokenKind::NationalLiteral(lexeme.clone()),
            lexeme,
            start_location,
        ))
    }

    fn read_until_newline(&mut self) -> String {
        let mut result = String::new();
        
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            result.push(c);
            self.advance();
        }
        
        result
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        self.column += 1;
        c
    }

    fn peek(&self) -> Option<char> {
        if self.current < self.source.len() {
            Some(self.source[self.current])
        } else {
            None
        }
    }

    fn peek_back(&self) -> Option<char> {
        if self.current > 0 {
            Some(self.source[self.current - 1])
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn current_location(&self) -> SourceLocation {
        SourceLocation::new(
            self.file.clone(),
            self.line,
            self.column,
            self.current,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_keywords() {
        let mut lexer = Lexer::new("IDENTIFICATION DIVISION");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 3); // IDENTIFICATION, DIVISION, EOF
        assert!(matches!(tokens[0].kind, TokenKind::IdentificationDivision));
    }

    #[test]
    fn test_lex_numeric_literal() {
        let mut lexer = Lexer::new("123.45");
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].kind, TokenKind::NumericLiteral(_)));
    }

    #[test]
    fn test_lex_string_literal() {
        let mut lexer = Lexer::new("\"Hello World\"");
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].kind, TokenKind::AlphanumericLiteral(_)));
    }

    #[test]
    fn test_lex_identifier() {
        let mut lexer = Lexer::new("CUSTOMER-NAME");
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].kind, TokenKind::Identifier(_)));
    }

    #[test]
    fn test_lex_level_number() {
        let mut lexer = Lexer::new("01 CUSTOMER-RECORD");
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].kind, TokenKind::LevelNumber(1)));
    }
}

// Made with Bob

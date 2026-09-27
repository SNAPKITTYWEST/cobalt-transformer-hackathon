use anyhow::{Result, bail};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum JclTokenKind {
    // JCL Statement Types
    Job,
    Exec,
    Dd,
    If,
    Then,
    Else,
    Endif,
    Set,
    Include,
    Jcllib,
    Proc,
    Pend,
    
    // Keywords
    Order,
    Cond,
    Class,
    Msgclass,
    Msglevel,
    Notify,
    Region,
    Time,
    Typrun,
    User,
    Password,
    
    // DD Parameters
    Dsn,
    Disp,
    Space,
    Dcb,
    Unit,
    Vol,
    Volume,
    Sysout,
    Dummy,
    Data,
    Dlm,
    
    // DISP values
    New,
    Old,
    Shr,
    Mod,
    Catlg,
    Delete,
    Keep,
    Pass,
    Uncatlg,
    
    // EXEC parameters
    Pgm,
    Parm,
    
    // Operators
    Equals,
    Comma,
    LeftParen,
    RightParen,
    Ampersand,
    Period,
    
    // Literals
    Identifier(String),
    StringLiteral(String),
    NumericLiteral(String),
    SymbolicParameter(String),
    
    // Special
    Comment(String),
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct JclToken {
    pub kind: JclTokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl JclToken {
    pub fn new(kind: JclTokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Self { kind, lexeme, line, column }
    }
}

pub struct JclLexer {
    source: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
}

impl JclLexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<JclToken>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace();
            
            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        tokens.push(JclToken::new(
            JclTokenKind::Eof,
            String::new(),
            self.line,
            self.column,
        ));

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<JclToken> {
        let start_line = self.line;
        let start_column = self.column;
        let c = self.advance();

        match c {
            '/' if self.column == 2 && self.peek() == Some('/') => {
                self.advance(); // consume second '/'
                self.read_jcl_statement()
            }
            '/' if self.peek() == Some('*') => {
                self.advance(); // consume '*'
                let comment = self.read_until_newline();
                Ok(JclToken::new(
                    JclTokenKind::Comment(comment.clone()),
                    comment,
                    start_line,
                    start_column,
                ))
            }
            '=' => Ok(JclToken::new(JclTokenKind::Equals, "=".to_string(), start_line, start_column)),
            ',' => Ok(JclToken::new(JclTokenKind::Comma, ",".to_string(), start_line, start_column)),
            '(' => Ok(JclToken::new(JclTokenKind::LeftParen, "(".to_string(), start_line, start_column)),
            ')' => Ok(JclToken::new(JclTokenKind::RightParen, ")".to_string(), start_line, start_column)),
            '&' => self.read_symbolic_parameter(),
            '.' => Ok(JclToken::new(JclTokenKind::Period, ".".to_string(), start_line, start_column)),
            '\'' => self.read_string_literal(),
            '\n' => {
                self.line += 1;
                self.column = 1;
                Ok(JclToken::new(JclTokenKind::Newline, "\n".to_string(), start_line, start_column))
            }
            c if c.is_alphabetic() || c == '_' => self.read_identifier_or_keyword(),
            c if c.is_ascii_digit() => self.read_number(),
            _ => bail!("Unexpected character: {} at {}:{}", c, self.line, self.column),
        }
    }

    fn read_jcl_statement(&mut self) -> Result<JclToken> {
        let start_line = self.line;
        let start_column = self.column;
        
        // Read the statement name (e.g., JOBNAME, STEP1, DD1)
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '@' || c == '#' || c == '$' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }

        self.skip_whitespace();

        // Read the statement type (JOB, EXEC, DD, etc.)
        let mut stmt_type = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphabetic() {
                stmt_type.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let upper = stmt_type.to_uppercase();
        let kind = match upper.as_str() {
            "JOB" => JclTokenKind::Job,
            "EXEC" => JclTokenKind::Exec,
            "DD" => JclTokenKind::Dd,
            "IF" => JclTokenKind::If,
            "THEN" => JclTokenKind::Then,
            "ELSE" => JclTokenKind::Else,
            "ENDIF" => JclTokenKind::Endif,
            "SET" => JclTokenKind::Set,
            "INCLUDE" => JclTokenKind::Include,
            "JCLLIB" => JclTokenKind::Jcllib,
            "PROC" => JclTokenKind::Proc,
            "PEND" => JclTokenKind::Pend,
            _ => JclTokenKind::Identifier(format!("{} {}", name, stmt_type)),
        };

        Ok(JclToken::new(kind, format!("//{} {}", name, stmt_type), start_line, start_column))
    }

    fn read_identifier_or_keyword(&mut self) -> Result<JclToken> {
        let start_line = self.line;
        let start_column = self.column;
        let mut lexeme = String::new();
        lexeme.push(self.peek_back().unwrap());

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '@' || c == '#' || c == '$' {
                lexeme.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let upper = lexeme.to_uppercase();
        let kind = match upper.as_str() {
            // DD Parameters
            "DSN" => JclTokenKind::Dsn,
            "DISP" => JclTokenKind::Disp,
            "SPACE" => JclTokenKind::Space,
            "DCB" => JclTokenKind::Dcb,
            "UNIT" => JclTokenKind::Unit,
            "VOL" | "VOLUME" => JclTokenKind::Volume,
            "SYSOUT" => JclTokenKind::Sysout,
            "DUMMY" => JclTokenKind::Dummy,
            "DATA" => JclTokenKind::Data,
            "DLM" => JclTokenKind::Dlm,
            
            // DISP values
            "NEW" => JclTokenKind::New,
            "OLD" => JclTokenKind::Old,
            "SHR" => JclTokenKind::Shr,
            "MOD" => JclTokenKind::Mod,
            "CATLG" => JclTokenKind::Catlg,
            "DELETE" => JclTokenKind::Delete,
            "KEEP" => JclTokenKind::Keep,
            "PASS" => JclTokenKind::Pass,
            "UNCATLG" => JclTokenKind::Uncatlg,
            
            // EXEC parameters
            "PGM" => JclTokenKind::Pgm,
            "PARM" => JclTokenKind::Parm,
            
            // JOB parameters
            "CLASS" => JclTokenKind::Class,
            "MSGCLASS" => JclTokenKind::Msgclass,
            "MSGLEVEL" => JclTokenKind::Msglevel,
            "NOTIFY" => JclTokenKind::Notify,
            "REGION" => JclTokenKind::Region,
            "TIME" => JclTokenKind::Time,
            "TYPRUN" => JclTokenKind::Typrun,
            "USER" => JclTokenKind::User,
            "PASSWORD" => JclTokenKind::Password,
            "COND" => JclTokenKind::Cond,
            
            _ => JclTokenKind::Identifier(lexeme.clone()),
        };

        Ok(JclToken::new(kind, lexeme, start_line, start_column))
    }

    fn read_symbolic_parameter(&mut self) -> Result<JclToken> {
        let start_line = self.line;
        let start_column = self.column;
        let mut lexeme = String::from("&");

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                lexeme.push(c);
                self.advance();
            } else {
                break;
            }
        }

        Ok(JclToken::new(
            JclTokenKind::SymbolicParameter(lexeme.clone()),
            lexeme,
            start_line,
            start_column,
        ))
    }

    fn read_string_literal(&mut self) -> Result<JclToken> {
        let start_line = self.line;
        let start_column = self.column;
        let mut lexeme = String::from("'");

        while let Some(c) = self.peek() {
            if c == '\'' {
                lexeme.push(c);
                self.advance();
                
                // Check for doubled quote (escape)
                if self.peek() == Some('\'') {
                    lexeme.push('\'');
                    self.advance();
                } else {
                    break;
                }
            } else {
                lexeme.push(c);
                self.advance();
            }
        }

        Ok(JclToken::new(
            JclTokenKind::StringLiteral(lexeme.clone()),
            lexeme,
            start_line,
            start_column,
        ))
    }

    fn read_number(&mut self) -> Result<JclToken> {
        let start_line = self.line;
        let start_column = self.column;
        let mut lexeme = String::new();
        lexeme.push(self.peek_back().unwrap());

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                lexeme.push(c);
                self.advance();
            } else {
                break;
            }
        }

        Ok(JclToken::new(
            JclTokenKind::NumericLiteral(lexeme.clone()),
            lexeme,
            start_line,
            start_column,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_job_statement() {
        let mut lexer = JclLexer::new("//MYJOB JOB CLASS=A");
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].kind, JclTokenKind::Job));
    }

    #[test]
    fn test_lex_dd_statement() {
        let mut lexer = JclLexer::new("//DD1 DD DSN=MY.DATA.SET,DISP=SHR");
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].kind, JclTokenKind::Dd));
    }

    #[test]
    fn test_lex_symbolic_parameter() {
        let mut lexer = JclLexer::new("&PARM1");
        let tokens = lexer.tokenize().unwrap();
        
        assert!(matches!(tokens[0].kind, JclTokenKind::SymbolicParameter(_)));
    }
}

// Made with Bob

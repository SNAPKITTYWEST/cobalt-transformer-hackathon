use crate::ast::*;
use crate::lexer::{Token, TokenKind};
use crate::source_map::{SourceLocation, SourceSpan};
use crate::diagnostics::{Diagnostic, DiagnosticCollector};
use anyhow::{Result, bail, Context};
use std::path::PathBuf;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    diagnostics: DiagnosticCollector,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            diagnostics: DiagnosticCollector::new(),
        }
    }

    pub fn parse(&mut self) -> Result<CobolProgram> {
        let start = self.current_location();
        
        let identification = self.parse_identification_division()?;
        
        let environment = if self.check_division(&TokenKind::EnvironmentDivision, &TokenKind::Environment) {
            Some(self.parse_environment_division()?)
        } else {
            None
        };
        let data = if self.check_division(&TokenKind::DataDivision, &TokenKind::Data) {
            Some(self.parse_data_division()?)
        } else {
            None
        };
        let procedure = if self.check_division(&TokenKind::ProcedureDivision, &TokenKind::Procedure) {
            Some(self.parse_procedure_division()?)
        } else {
            None
        };
        let end = self.current_location();
        
        Ok(CobolProgram {
            identification,
            environment,
            data,
            procedure,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_identification_division(&mut self) -> Result<IdentificationDivision> {
        let start = self.current_location();
        
        // Handle both "IDENTIFICATION DIVISION" and "IDENTIFICATION-DIVISION"
        if self.check(&TokenKind::IdentificationDivision) {
            self.advance();
        } else {
            self.expect(&TokenKind::Identification)?;
            self.expect(&TokenKind::Division)?;
        }
        self.skip_optional(&TokenKind::Period);
        
        // Handle "PROGRAM-ID" (single token)
        self.expect(&TokenKind::ProgramId)?;
        self.skip_optional(&TokenKind::Period);
        
        let program_id = self.expect_identifier()?;
        self.skip_optional(&TokenKind::Period);
        
        let end = self.current_location();
        
        Ok(IdentificationDivision {
            program_id,
            author: None,
            installation: None,
            date_written: None,
            date_compiled: None,
            security: None,
            remarks: None,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_environment_division(&mut self) -> Result<EnvironmentDivision> {
        let start = self.current_location();
        
        // Handle both "ENVIRONMENT DIVISION" and "ENVIRONMENT-DIVISION"
        if self.check(&TokenKind::EnvironmentDivision) {
            self.advance();
        } else {
            self.expect(&TokenKind::Environment)?;
            self.expect(&TokenKind::Division)?;
        }
        self.skip_optional(&TokenKind::Period);
        let end = self.current_location();
        
        Ok(EnvironmentDivision {
            configuration: None,
            input_output: None,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_data_division(&mut self) -> Result<DataDivision> {
        let start = self.current_location();
        
        // Handle both "DATA DIVISION" and "DATA-DIVISION"
        if self.check(&TokenKind::DataDivision) {
            self.advance();
        } else {
            self.expect(&TokenKind::Data)?;
            self.expect(&TokenKind::Division)?;
        }
        self.skip_optional(&TokenKind::Period);
        
        let mut working_storage = None;
        
        // Check for WORKING-STORAGE SECTION (separate tokens) or WORKING-STORAGE-SECTION (compound)
        self.skip_noise();
        if self.check(&TokenKind::WorkingStorageSection) {
            working_storage = Some(self.parse_working_storage_section()?);
        }
        
        let end = self.current_location();
        
        Ok(DataDivision {
            file_section: None,
            working_storage,
            local_storage: None,
            linkage_section: None,
            report_section: None,
            screen_section: None,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_working_storage_section(&mut self) -> Result<WorkingStorageSection> {
        let start = self.current_location();
        
        // The lexer maps "WORKING-STORAGE" to WorkingStorageSection
        // Then "SECTION" becomes an identifier that we need to skip
        self.expect(&TokenKind::WorkingStorageSection)?;
        
        // Skip "SECTION" if present (it will be an identifier)
        self.skip_noise();
        if let Some(token) = self.peek() {
            if let TokenKind::Identifier(name) = &token.kind {
                if name.to_uppercase() == "SECTION" {
                    self.advance();
                }
            }
        }
        
        self.skip_optional(&TokenKind::Period);
        
        let mut items = Vec::new();
        self.skip_noise();  // Skip newlines after period
        
        eprintln!("DEBUG: After WORKING-STORAGE SECTION, current token: {:?}", self.peek().map(|t| &t.kind));
        
        while self.check_level_number() {
            items.push(self.parse_data_item()?);
        }
        
        let end = self.current_location();
        
        Ok(WorkingStorageSection {
            items,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_data_item(&mut self) -> Result<DataItem> {
        let start = self.current_location();
        let level = self.expect_level_number()?;
        let name = if self.check_identifier() {
            Some(self.expect_identifier()?)
        } else {
            None
        };
        
        let mut picture = None;
        let mut value = None;
        
        while !self.check(&TokenKind::Period) && !self.check_level_number() {
            if self.check(&TokenKind::Picture) {
                self.advance();
                self.skip_optional(&TokenKind::Is);
                picture = Some(self.parse_picture_clause()?);
            } else if self.check(&TokenKind::Value) {
                self.advance();
                self.skip_optional(&TokenKind::Is);
                value = Some(self.parse_value()?);
            } else {
                self.advance();
            }
        }
        
        self.skip_optional(&TokenKind::Period);
        let end = self.current_location();
        
        Ok(DataItem {
            level,
            name,
            picture,
            usage: None,
            value,
            redefines: None,
            renames: None,
            occurs: None,
            sign_clause: None,
            synchronized: false,
            justified: false,
            blank_when_zero: false,
            external: false,
            global: false,
            based: false,
            children: Vec::new(),
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_picture_clause(&mut self) -> Result<PictureClause> {
        let mut picture_string = self.expect_identifier()?;
        
        // Check for size specification in parentheses: X(20), 9(5), etc.
        if self.check(&TokenKind::LeftParen) {
            self.advance(); // consume '('
            if let Some(token) = self.peek() {
                if let TokenKind::NumericLiteral(size_str) = &token.kind {
                    picture_string.push('(');
                    picture_string.push_str(size_str);
                    picture_string.push(')');
                    self.advance(); // consume size
                }
            }
            self.expect(&TokenKind::RightParen)?;
        }
        
        let (category, size, scale, has_sign) = self.analyze_picture(&picture_string);
        
        Ok(PictureClause {
            picture_string,
            category,
            size,
            scale,
            has_sign,
        })
    }

    fn analyze_picture(&self, pic: &str) -> (PictureCategory, usize, i32, bool) {
        let upper = pic.to_uppercase();
        let has_9 = upper.contains('9');
        let has_x = upper.contains('X');
        
        let category = if has_9 {
            PictureCategory::Numeric
        } else if has_x {
            PictureCategory::Alphanumeric
        } else {
            PictureCategory::Alphanumeric
        };
        
        // Extract size from picture string
        // Examples: X(20) -> 20, 9(5) -> 5, XXX -> 3
        let size = if let Some(start) = pic.find('(') {
            if let Some(end) = pic.find(')') {
                // Extract number between parentheses
                if let Ok(n) = pic[start + 1..end].parse::<usize>() {
                    n
                } else {
                    1
                }
            } else {
                1
            }
        } else {
            // Count occurrences of picture characters
            pic.chars().filter(|c| c.is_alphabetic() || c.is_numeric()).count()
        };
        
        (category, size, 0, false)
    }

    fn parse_value(&mut self) -> Result<Value> {
        if self.check_figurative() {
            Ok(Value::Figurative(self.parse_figurative()?))
        } else {
            Ok(Value::Literal(self.parse_literal()?))
        }
    }

    fn parse_procedure_division(&mut self) -> Result<ProcedureDivision> {
        let start = self.current_location();
        
        // Handle both "PROCEDURE DIVISION" and "PROCEDURE-DIVISION"
        if self.check(&TokenKind::ProcedureDivision) {
            self.advance();
        } else {
            self.expect(&TokenKind::Procedure)?;
            self.expect(&TokenKind::Division)?;
        }
        self.skip_optional(&TokenKind::Period);
        
        let mut paragraphs = Vec::new();
        while !self.is_at_end() {
            if self.is_paragraph_name() {
                paragraphs.push(self.parse_paragraph()?);
            } else {
                break;
            }
        }
        
        let end = self.current_location();
        
        Ok(ProcedureDivision {
            using_clause: Vec::new(),
            returning_clause: None,
            declaratives: Vec::new(),
            sections: Vec::new(),
            paragraphs,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_paragraph(&mut self) -> Result<Paragraph> {
        let start = self.current_location();
        let name = self.expect_identifier()?;
        self.skip_optional(&TokenKind::Period);
        
        let mut statements = Vec::new();
        while !self.is_at_end() && !self.is_paragraph_name() {
            if let Some(stmt) = self.parse_statement()? {
                statements.push(stmt);
            } else {
                break;
            }
        }
        
        let end = self.current_location();
        
        Ok(Paragraph {
            name,
            statements,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_statement(&mut self) -> Result<Option<Statement>> {
        // Skip noise before checking for statements
        self.skip_noise();
        
        if self.is_at_end() || self.is_paragraph_name() {
            return Ok(None);
        }
        
        let stmt = if self.check(&TokenKind::Move) {
            Statement::Move(self.parse_move_statement()?)
        } else if self.check(&TokenKind::Display) {
            Statement::Display(self.parse_display_statement()?)
        } else if self.check(&TokenKind::Accept) {
            Statement::Accept(self.parse_accept_statement()?)
        } else if self.check(&TokenKind::If) {
            Statement::If(self.parse_if_statement()?)
        } else if self.check(&TokenKind::Perform) {
            Statement::Perform(self.parse_perform_statement()?)
        } else if self.check(&TokenKind::Stop) {
            Statement::Stop(self.parse_stop_statement()?)
        } else if self.check(&TokenKind::GoBack) {
            Statement::GoBack(self.parse_goback_statement()?)
        } else {
            return Ok(None);
        };
        
        Ok(Some(stmt))
    }

    fn parse_move_statement(&mut self) -> Result<MoveStatement> {
        let start = self.current_location();
        self.expect(&TokenKind::Move)?;
        let source = self.parse_expression()?;
        self.expect(&TokenKind::To)?;
        let target = self.parse_identifier()?;
        self.skip_optional(&TokenKind::Period);
        let end = self.current_location();
        
        Ok(MoveStatement {
            source,
            targets: vec![target],
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_display_statement(&mut self) -> Result<DisplayStatement> {
        let start = self.current_location();
        self.expect(&TokenKind::Display)?;
        let item = self.parse_expression()?;
        self.skip_optional(&TokenKind::Period);
        let end = self.current_location();
        
        Ok(DisplayStatement {
            items: vec![item],
            upon: None,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_accept_statement(&mut self) -> Result<AcceptStatement> {
        let start = self.current_location();
        self.expect(&TokenKind::Accept)?;
        let target = self.parse_identifier()?;
        self.skip_optional(&TokenKind::Period);
        let end = self.current_location();
        
        Ok(AcceptStatement {
            target,
            from: None,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_if_statement(&mut self) -> Result<IfStatement> {
        let start = self.current_location();
        self.expect(&TokenKind::If)?;
        let condition = self.parse_condition()?;
        
        let mut then_statements = Vec::new();
        while !self.check(&TokenKind::Else) && !self.check(&TokenKind::EndIf) && !self.check(&TokenKind::Period) {
            if let Some(stmt) = self.parse_statement()? {
                then_statements.push(stmt);
            } else {
                break;
            }
        }
        
        let else_statements = if self.check(&TokenKind::Else) {
            self.advance();
            let mut stmts = Vec::new();
            while !self.check(&TokenKind::EndIf) && !self.check(&TokenKind::Period) {
                if let Some(stmt) = self.parse_statement()? {
                    stmts.push(stmt);
                } else {
                    break;
                }
            }
            Some(stmts)
        } else {
            None
        };
        
        self.skip_optional(&TokenKind::EndIf);
        self.skip_optional(&TokenKind::Period);
        let end = self.current_location();
        
        Ok(IfStatement {
            condition,
            then_statements,
            else_statements,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_perform_statement(&mut self) -> Result<PerformStatement> {
        let start = self.current_location();
        self.expect(&TokenKind::Perform)?;
        
        let target = if self.check_identifier() {
            let from = self.expect_identifier()?;
            let through = if self.check(&TokenKind::Thru) {
                self.advance();
                Some(self.expect_identifier()?)
            } else {
                None
            };
            PerformTarget::Procedure(ProcedureRange { from, through })
        } else {
            PerformTarget::Inline(Vec::new())
        };
        
        self.skip_optional(&TokenKind::Period);
        let end = self.current_location();
        
        Ok(PerformStatement {
            target,
            times: None,
            until: None,
            varying: None,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_stop_statement(&mut self) -> Result<StopStatement> {
        let start = self.current_location();
        self.expect(&TokenKind::Stop)?;
        
        let stop_type = if self.match_identifier("RUN") {
            StopType::Run
        } else {
            StopType::Run
        };
        
        self.skip_optional(&TokenKind::Period);
        let end = self.current_location();
        
        Ok(StopStatement {
            stop_type,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_goback_statement(&mut self) -> Result<GoBackStatement> {
        let start = self.current_location();
        self.expect(&TokenKind::GoBack)?;
        self.skip_optional(&TokenKind::Period);
        let end = self.current_location();
        
        Ok(GoBackStatement {
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_condition(&mut self) -> Result<Condition> {
        let left = self.parse_expression()?;
        
        if self.check(&TokenKind::Equal) || self.check(&TokenKind::Greater) || self.check(&TokenKind::Less) {
            let operator = self.parse_relation_operator()?;
            let right = self.parse_expression()?;
            let span = SourceSpan::new(left.span().start.clone(), right.span().end.clone());
            
            Ok(Condition::Relation(RelationCondition {
                left,
                operator,
                right,
                span,
            }))
        } else {
            bail!("Expected condition")
        }
    }

    fn parse_relation_operator(&mut self) -> Result<RelationOperator> {
        if self.check(&TokenKind::Equal) {
            self.advance();
            Ok(RelationOperator::Equal)
        } else if self.check(&TokenKind::Greater) {
            self.advance();
            if self.check(&TokenKind::Than) {
                self.advance();
            }
            Ok(RelationOperator::Greater)
        } else if self.check(&TokenKind::Less) {
            self.advance();
            if self.check(&TokenKind::Than) {
                self.advance();
            }
            Ok(RelationOperator::Less)
        } else {
            bail!("Expected relation operator")
        }
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        if self.check_literal() {
            Ok(Expression::Literal(self.parse_literal()?))
        } else if self.check_figurative() {
            Ok(Expression::Figurative(self.parse_figurative()?))
        } else if self.check_identifier() {
            Ok(Expression::Identifier(self.parse_identifier()?))
        } else {
            bail!("Expected expression")
        }
    }

    fn parse_identifier(&mut self) -> Result<Identifier> {
        let start = self.current_location();
        let name = self.expect_identifier()?;
        let end = self.current_location();
        
        Ok(Identifier {
            name,
            qualification: Vec::new(),
            subscripts: Vec::new(),
            reference_modification: None,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_literal(&mut self) -> Result<Literal> {
        if let Some(token) = self.peek() {
            let result = match &token.kind {
                TokenKind::NumericLiteral(val) => Ok(Literal::Numeric(val.clone())),
                TokenKind::AlphanumericLiteral(val) => Ok(Literal::Alphanumeric(val.clone())),
                _ => bail!("Expected literal"),
            };
            if result.is_ok() {
                self.advance();
            }
            result
        } else {
            bail!("Expected literal")
        }
    }

    fn parse_figurative(&mut self) -> Result<FigurativeConstant> {
        if self.check(&TokenKind::Zero) || self.check(&TokenKind::Zeros) {
            self.advance();
            Ok(FigurativeConstant::Zero)
        } else if self.check(&TokenKind::Space) || self.check(&TokenKind::Spaces) {
            self.advance();
            Ok(FigurativeConstant::Space)
        } else {
            bail!("Expected figurative constant")
        }
    }

    // Helper methods
    fn check(&self, kind: &TokenKind) -> bool {
        if let Some(token) = self.peek() {
            std::mem::discriminant(&token.kind) == std::mem::discriminant(kind)
        } else {
            false
        }
    }

    fn check_identifier(&self) -> bool {
        if let Some(token) = self.peek() {
            matches!(token.kind, TokenKind::Identifier(_))
        } else {
            false
        }
    }

    fn check_literal(&self) -> bool {
        if let Some(token) = self.peek() {
            matches!(token.kind, TokenKind::NumericLiteral(_) | TokenKind::AlphanumericLiteral(_))
        } else {
            false
        }
    }

    fn check_figurative(&self) -> bool {
        if let Some(token) = self.peek() {
            matches!(token.kind, TokenKind::Zero | TokenKind::Zeros | TokenKind::Space | TokenKind::Spaces)
        } else {
            false
        }
    }

    fn check_level_number(&self) -> bool {
        if let Some(token) = self.peek() {
            match &token.kind {
                TokenKind::LevelNumber(_) => true,
                // Workaround: Accept numeric literals as level numbers
                // The lexer doesn't distinguish between level numbers and regular numbers
                TokenKind::NumericLiteral(s) => {
                    // Check if it's a valid level number (01-49, 66, 77, 88)
                    if let Ok(n) = s.parse::<u8>() {
                        (1..=49).contains(&n) || n == 66 || n == 77 || n == 88
                    } else {
                        false
                    }
                }
                _ => false
            }
        } else {
            false
        }
    }

    fn check_division(&mut self, compound: &TokenKind, separate: &TokenKind) -> bool {
        // Skip newlines and comments before checking
        self.skip_noise();
        // Check for compound form (e.g., DATA-DIVISION) or separate form (e.g., DATA DIVISION)
        self.check(compound) || self.check(separate)
    }

    fn is_paragraph_name(&self) -> bool {
        // Check if current position (after skipping noise) has identifier followed by period
        let mut pos = self.current;
        
        // Skip noise
        while pos < self.tokens.len() {
            if let Some(token) = self.tokens.get(pos) {
                if matches!(token.kind, TokenKind::Newline | TokenKind::Comment(_)) {
                    pos += 1;
                    continue;
                }
                break;
            } else {
                return false;
            }
        }
        
        // Check for identifier
        if let Some(token) = self.tokens.get(pos) {
            if !matches!(token.kind, TokenKind::Identifier(_)) {
                return false;
            }
        } else {
            return false;
        }
        
        pos += 1;
        
        // Skip noise after identifier
        while pos < self.tokens.len() {
            if let Some(token) = self.tokens.get(pos) {
                if matches!(token.kind, TokenKind::Newline | TokenKind::Comment(_)) {
                    pos += 1;
                    continue;
                }
                break;
            } else {
                return false;
            }
        }
        
        // Check for period
        if let Some(token) = self.tokens.get(pos) {
            matches!(token.kind, TokenKind::Period)
        } else {
            false
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Eof))
    }

    fn skip_noise(&mut self) {
        while let Some(token) = self.peek() {
            if matches!(token.kind, TokenKind::Newline | TokenKind::Comment(_)) {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn peek_ahead(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.current + n)
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1)
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<()> {
        self.skip_noise();
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            bail!("Expected {:?}, found {:?}", kind, self.peek().map(|t| &t.kind))
        }
    }

    fn expect_identifier(&mut self) -> Result<String> {
        self.skip_noise();
        if let Some(token) = self.peek() {
            if let TokenKind::Identifier(name) = &token.kind {
                let result = name.clone();
                self.advance();
                return Ok(result);
            }
        }
        bail!("Expected identifier")
    }

    fn expect_level_number(&mut self) -> Result<u8> {
        if let Some(token) = self.peek() {
            match &token.kind {
                TokenKind::LevelNumber(level) => {
                    let result = *level;
                    self.advance();
                    return Ok(result);
                }
                // Workaround: Accept numeric literals as level numbers
                TokenKind::NumericLiteral(s) => {
                    if let Ok(n) = s.parse::<u8>() {
                        if (1..=49).contains(&n) || n == 66 || n == 77 || n == 88 {
                            self.advance();
                            return Ok(n);
                        }
                    }
                }
                _ => {}
            }
        }
        bail!("Expected level number")
    }

    fn skip_optional(&mut self, kind: &TokenKind) {
        if self.check(kind) {
            self.advance();
        }
    }

    fn match_identifier(&mut self, name: &str) -> bool {
        if let Some(token) = self.peek() {
            if let TokenKind::Identifier(id) = &token.kind {
                if id.to_uppercase() == name.to_uppercase() {
                    self.advance();
                    return true;
                }
            }
        }
        false
    }

    fn current_location(&self) -> SourceLocation {
        if let Some(token) = self.peek() {
            token.location.clone()
        } else if let Some(last) = self.tokens.last() {
            last.location.clone()
        } else {
            SourceLocation::new(PathBuf::from("<input>"), 1, 1, 0)
        }
    }
}

impl Expression {
    fn span(&self) -> &SourceSpan {
        match self {
            Expression::Identifier(id) => &id.span,
            Expression::Literal(_) => {
                // Simplified - should track spans for literals too
                static DUMMY: SourceSpan = SourceSpan {
                    start: SourceLocation {
                        file: PathBuf::new(),
                        line: 0,
                        column: 0,
                        offset: 0,
                    },
                    end: SourceLocation {
                        file: PathBuf::new(),
                        line: 0,
                        column: 0,
                        offset: 0,
                    },
                };
                &DUMMY
            }
            _ => {
                static DUMMY: SourceSpan = SourceSpan {
                    start: SourceLocation {
                        file: PathBuf::new(),
                        line: 0,
                        column: 0,
                        offset: 0,
                    },
                    end: SourceLocation {
                        file: PathBuf::new(),
                        line: 0,
                        column: 0,
                        offset: 0,
                    },
                };
                &DUMMY
            }
        }
    }
}

// Made with Bob

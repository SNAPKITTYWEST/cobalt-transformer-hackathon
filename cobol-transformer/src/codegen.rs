use crate::ast::*;
use anyhow::Result;

pub struct CodeGenerator {
    indent_level: usize,
    output: String,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            output: String::new(),
        }
    }

    pub fn generate(&mut self, program: &CobolProgram) -> Result<String> {
        self.output.clear();
        
        // Generate IDENTIFICATION DIVISION
        self.emit_line("IDENTIFICATION DIVISION.");
        self.emit_line(&format!("PROGRAM-ID. {}.", program.identification.program_id));
        self.emit_line("");

        // Generate ENVIRONMENT DIVISION
        if program.environment.is_some() {
            self.emit_line("ENVIRONMENT DIVISION.");
            self.emit_line("");
        }

        // Generate DATA DIVISION
        if let Some(data) = &program.data {
            self.emit_line("DATA DIVISION.");
            
            if let Some(ws) = &data.working_storage {
                self.emit_line("WORKING-STORAGE SECTION.");
                for item in &ws.items {
                    self.generate_data_item(item)?;
                }
                self.emit_line("");
            }
        }

        // Generate PROCEDURE DIVISION
        if let Some(proc) = &program.procedure {
            self.emit_line("PROCEDURE DIVISION.");
            
            for para in &proc.paragraphs {
                self.generate_paragraph(para)?;
            }
        }

        Ok(self.output.clone())
    }

    fn generate_data_item(&mut self, item: &DataItem) -> Result<()> {
        let mut line = format!("{:02}", item.level);
        
        if let Some(name) = &item.name {
            line.push_str(&format!(" {}", name));
        }
        
        if let Some(pic) = &item.picture {
            line.push_str(&format!(" PIC {}", pic.picture_string));
        }
        
        if let Some(val) = &item.value {
            match val {
                Value::Literal(lit) => {
                    match lit {
                        Literal::Numeric(n) => line.push_str(&format!(" VALUE {}", n)),
                        Literal::Alphanumeric(s) => line.push_str(&format!(" VALUE {}", s)),
                        _ => {}
                    }
                }
                Value::Figurative(fig) => {
                    let fig_str = match fig {
                        FigurativeConstant::Zero => "ZERO",
                        FigurativeConstant::Space => "SPACE",
                        _ => "ZERO",
                    };
                    line.push_str(&format!(" VALUE {}", fig_str));
                }
            }
        }
        
        line.push('.');
        self.emit_line(&line);
        
        for child in &item.children {
            self.generate_data_item(child)?;
        }
        
        Ok(())
    }

    fn generate_paragraph(&mut self, para: &Paragraph) -> Result<()> {
        self.emit_line(&format!("{}.", para.name));
        self.indent_level += 1;
        
        for stmt in &para.statements {
            self.generate_statement(stmt)?;
        }
        
        self.indent_level -= 1;
        self.emit_line("");
        Ok(())
    }

    fn generate_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::Move(s) => {
                let source = self.format_expression(&s.source);
                let target = &s.targets[0].name;
                self.emit_line(&format!("MOVE {} TO {}.", source, target));
            }
            Statement::Display(s) => {
                let item = self.format_expression(&s.items[0]);
                self.emit_line(&format!("DISPLAY {}.", item));
            }
            Statement::Accept(s) => {
                self.emit_line(&format!("ACCEPT {}.", s.target.name));
            }
            Statement::If(s) => {
                let cond = self.format_condition(&s.condition);
                self.emit_line(&format!("IF {}", cond));
                self.indent_level += 1;
                for stmt in &s.then_statements {
                    self.generate_statement(stmt)?;
                }
                self.indent_level -= 1;
                if let Some(else_stmts) = &s.else_statements {
                    self.emit_line("ELSE");
                    self.indent_level += 1;
                    for stmt in else_stmts {
                        self.generate_statement(stmt)?;
                    }
                    self.indent_level -= 1;
                }
                self.emit_line("END-IF.");
            }
            Statement::Perform(s) => {
                match &s.target {
                    PerformTarget::Procedure(range) => {
                        if let Some(through) = &range.through {
                            self.emit_line(&format!("PERFORM {} THRU {}.", range.from, through));
                        } else {
                            self.emit_line(&format!("PERFORM {}.", range.from));
                        }
                    }
                    PerformTarget::Inline(_) => {
                        self.emit_line("PERFORM");
                        self.emit_line("END-PERFORM.");
                    }
                }
            }
            Statement::Stop(s) => {
                match &s.stop_type {
                    StopType::Run => self.emit_line("STOP RUN."),
                    _ => self.emit_line("STOP RUN."),
                }
            }
            Statement::GoBack(_) => {
                self.emit_line("GOBACK.");
            }
            _ => {}
        }
        Ok(())
    }

    fn format_expression(&self, expr: &Expression) -> String {
        match expr {
            Expression::Literal(lit) => match lit {
                Literal::Numeric(n) => n.clone(),
                Literal::Alphanumeric(s) => s.clone(),
                _ => String::new(),
            },
            Expression::Identifier(id) => id.name.clone(),
            Expression::Figurative(fig) => match fig {
                FigurativeConstant::Zero => "ZERO".to_string(),
                FigurativeConstant::Space => "SPACE".to_string(),
                _ => "ZERO".to_string(),
            },
            _ => String::new(),
        }
    }

    fn format_condition(&self, cond: &Condition) -> String {
        match cond {
            Condition::Relation(rel) => {
                let left = self.format_expression(&rel.left);
                let right = self.format_expression(&rel.right);
                let op = match rel.operator {
                    RelationOperator::Equal => "=",
                    RelationOperator::Greater => ">",
                    RelationOperator::Less => "<",
                    _ => "=",
                };
                format!("{} {} {}", left, op, right)
            }
            _ => String::new(),
        }
    }

    fn emit_line(&mut self, line: &str) {
        let indent = "    ".repeat(self.indent_level);
        self.output.push_str(&format!("{}{}\n", indent, line));
    }
}

// Made with Bob

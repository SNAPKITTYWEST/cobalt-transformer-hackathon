use crate::jcl::lexer::{JclToken, JclTokenKind};
use crate::jcl::ast::*;
use anyhow::{Result, bail};
use std::collections::HashMap;

pub struct JclParser {
    tokens: Vec<JclToken>,
    current: usize,
}

impl JclParser {
    pub fn new(tokens: Vec<JclToken>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<JclJob> {
        self.skip_noise();
        
        // Parse JOB card
        let job_card = self.parse_job_card()?;
        let mut job = JclJob::new(job_card.name.clone());
        job.job_card = job_card;

        // Parse steps, procedures, and conditionals
        while !self.is_at_end() {
            self.skip_noise();
            
            if self.is_at_end() {
                break;
            }

            if self.check(&JclTokenKind::Proc) {
                job.add_procedure(self.parse_procedure()?);
            } else if self.check(&JclTokenKind::If) {
                job.conditionals.push(self.parse_conditional()?);
            } else if self.check(&JclTokenKind::Exec) {
                job.add_step(self.parse_step()?);
            } else {
                self.advance();
            }
        }

        Ok(job)
    }

    fn parse_job_card(&mut self) -> Result<JobCard> {
        self.expect(&JclTokenKind::Job)?;
        
        let mut parameters = HashMap::new();
        let mut accounting_info = None;
        let mut programmer_name = None;

        // Parse job parameters
        while !self.check(&JclTokenKind::Newline) && !self.is_at_end() {
            if let Some(token) = self.peek() {
                match &token.kind {
                    JclTokenKind::Identifier(name) => {
                        let param_name = name.clone();
                        self.advance();
                        
                        if self.check(&JclTokenKind::Equals) {
                            self.advance();
                            let value = self.parse_parameter_value()?;
                            parameters.insert(param_name, value);
                        }
                    }
                    JclTokenKind::StringLiteral(s) => {
                        if accounting_info.is_none() {
                            accounting_info = Some(s.clone());
                        } else if programmer_name.is_none() {
                            programmer_name = Some(s.clone());
                        }
                        self.advance();
                    }
                    _ => {
                        self.advance();
                    }
                }
            }
            
            self.skip_optional(&JclTokenKind::Comma);
        }

        Ok(JobCard {
            name: "JOB".to_string(),
            accounting_info,
            programmer_name,
            parameters,
        })
    }

    fn parse_step(&mut self) -> Result<JclStep> {
        // Parse step name from the EXEC statement identifier
        let step_name = if let Some(token) = self.peek() {
            if let JclTokenKind::Identifier(name) = &token.kind {
                let n = name.clone();
                self.advance();
                n
            } else {
                "STEP".to_string()
            }
        } else {
            "STEP".to_string()
        };

        self.expect(&JclTokenKind::Exec)?;
        
        let exec = self.parse_exec_statement()?;
        let mut step = JclStep::new(step_name, exec);

        // Parse DD statements
        self.skip_noise();
        while self.check(&JclTokenKind::Dd) {
            step.add_dd(self.parse_dd_statement()?);
            self.skip_noise();
        }

        Ok(step)
    }

    fn parse_exec_statement(&mut self) -> Result<ExecStatement> {
        let mut exec = ExecStatement {
            program: None,
            procedure: None,
            parameters: HashMap::new(),
            parm: None,
            cond: None,
            region: None,
            time: None,
        };

        // Parse EXEC parameters
        while !self.check(&JclTokenKind::Newline) && !self.is_at_end() {
            if self.check(&JclTokenKind::Pgm) {
                self.advance();
                self.expect(&JclTokenKind::Equals)?;
                exec.program = Some(self.expect_identifier()?);
            } else if self.check(&JclTokenKind::Parm) {
                self.advance();
                self.expect(&JclTokenKind::Equals)?;
                exec.parm = Some(self.parse_parameter_value()?);
            } else if self.check(&JclTokenKind::Cond) {
                self.advance();
                self.expect(&JclTokenKind::Equals)?;
                exec.cond = Some(self.parse_parameter_value()?);
            } else if let Some(token) = self.peek() {
                if let JclTokenKind::Identifier(name) = &token.kind {
                    let param_name = name.clone();
                    self.advance();
                    
                    if self.check(&JclTokenKind::Equals) {
                        self.advance();
                        let value = self.parse_parameter_value()?;
                        
                        match param_name.to_uppercase().as_str() {
                            "REGION" => exec.region = Some(value),
                            "TIME" => exec.time = Some(value),
                            _ => {
                                exec.parameters.insert(param_name, value);
                            }
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
            
            self.skip_optional(&JclTokenKind::Comma);
        }

        Ok(exec)
    }

    fn parse_dd_statement(&mut self) -> Result<DdStatement> {
        self.expect(&JclTokenKind::Dd)?;
        
        let dd_name = if let Some(token) = self.peek() {
            if let JclTokenKind::Identifier(name) = &token.kind {
                let n = name.clone();
                self.advance();
                n
            } else {
                "DD".to_string()
            }
        } else {
            "DD".to_string()
        };

        let mut dd = DdStatement::new(dd_name);

        // Parse DD parameters
        while !self.check(&JclTokenKind::Newline) && !self.is_at_end() {
            if self.check(&JclTokenKind::Dsn) {
                self.advance();
                self.expect(&JclTokenKind::Equals)?;
                dd.dsn = Some(self.parse_parameter_value()?);
            } else if self.check(&JclTokenKind::Disp) {
                self.advance();
                self.expect(&JclTokenKind::Equals)?;
                dd.disp = Some(self.parse_disposition()?);
            } else if self.check(&JclTokenKind::Space) {
                self.advance();
                self.expect(&JclTokenKind::Equals)?;
                dd.space = Some(self.parse_space_allocation()?);
            } else if self.check(&JclTokenKind::Dcb) {
                self.advance();
                self.expect(&JclTokenKind::Equals)?;
                dd.dcb = Some(self.parse_dcb()?);
            } else if self.check(&JclTokenKind::Unit) {
                self.advance();
                self.expect(&JclTokenKind::Equals)?;
                dd.unit = Some(self.parse_parameter_value()?);
            } else if self.check(&JclTokenKind::Volume) {
                self.advance();
                self.expect(&JclTokenKind::Equals)?;
                dd.volume = Some(self.parse_parameter_value()?);
            } else if self.check(&JclTokenKind::Sysout) {
                self.advance();
                self.expect(&JclTokenKind::Equals)?;
                dd.sysout = Some(self.parse_parameter_value()?);
            } else if self.check(&JclTokenKind::Dummy) {
                self.advance();
                dd.dummy = true;
            } else if self.check(&JclTokenKind::Data) {
                self.advance();
                dd.data = true;
            } else if self.check(&JclTokenKind::Dlm) {
                self.advance();
                self.expect(&JclTokenKind::Equals)?;
                dd.dlm = Some(self.parse_parameter_value()?);
            } else if let Some(token) = self.peek() {
                if let JclTokenKind::Identifier(name) = &token.kind {
                    let param_name = name.clone();
                    self.advance();
                    
                    if self.check(&JclTokenKind::Equals) {
                        self.advance();
                        let value = self.parse_parameter_value()?;
                        dd.parameters.insert(param_name, value);
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
            
            self.skip_optional(&JclTokenKind::Comma);
        }

        Ok(dd)
    }

    fn parse_disposition(&mut self) -> Result<Disposition> {
        self.expect(&JclTokenKind::LeftParen)?;
        
        let status = self.parse_disposition_status()?;
        
        self.skip_optional(&JclTokenKind::Comma);
        
        let normal = if !self.check(&JclTokenKind::RightParen) {
            self.parse_disposition_action()?
        } else {
            DispositionAction::Keep
        };
        
        let abnormal = if self.check(&JclTokenKind::Comma) {
            self.advance();
            Some(self.parse_disposition_action()?)
        } else {
            None
        };
        
        self.expect(&JclTokenKind::RightParen)?;
        
        Ok(Disposition {
            status,
            normal_termination: normal,
            abnormal_termination: abnormal,
        })
    }

    fn parse_disposition_status(&mut self) -> Result<DispositionStatus> {
        if self.check(&JclTokenKind::New) {
            self.advance();
            Ok(DispositionStatus::New)
        } else if self.check(&JclTokenKind::Old) {
            self.advance();
            Ok(DispositionStatus::Old)
        } else if self.check(&JclTokenKind::Shr) {
            self.advance();
            Ok(DispositionStatus::Shr)
        } else if self.check(&JclTokenKind::Mod) {
            self.advance();
            Ok(DispositionStatus::Mod)
        } else {
            bail!("Expected disposition status")
        }
    }

    fn parse_disposition_action(&mut self) -> Result<DispositionAction> {
        if self.check(&JclTokenKind::Catlg) {
            self.advance();
            Ok(DispositionAction::Catlg)
        } else if self.check(&JclTokenKind::Delete) {
            self.advance();
            Ok(DispositionAction::Delete)
        } else if self.check(&JclTokenKind::Keep) {
            self.advance();
            Ok(DispositionAction::Keep)
        } else if self.check(&JclTokenKind::Pass) {
            self.advance();
            Ok(DispositionAction::Pass)
        } else if self.check(&JclTokenKind::Uncatlg) {
            self.advance();
            Ok(DispositionAction::Uncatlg)
        } else {
            bail!("Expected disposition action")
        }
    }

    fn parse_space_allocation(&mut self) -> Result<SpaceAllocation> {
        self.expect(&JclTokenKind::LeftParen)?;
        
        // Parse unit (TRK, CYL, or block size)
        let unit = if let Some(token) = self.peek() {
            if let JclTokenKind::Identifier(name) = &token.kind {
                let u = name.to_uppercase();
                self.advance();
                match u.as_str() {
                    "TRK" => SpaceUnit::Trk,
                    "CYL" => SpaceUnit::Cyl,
                    _ => {
                        if let Ok(size) = u.parse::<usize>() {
                            SpaceUnit::Blk(size)
                        } else {
                            SpaceUnit::Trk
                        }
                    }
                }
            } else if let JclTokenKind::NumericLiteral(n) = &token.kind {
                let size = n.parse::<usize>().unwrap_or(0);
                self.advance();
                SpaceUnit::Blk(size)
            } else {
                SpaceUnit::Trk
            }
        } else {
            SpaceUnit::Trk
        };
        
        self.expect(&JclTokenKind::Comma)?;
        
        // Parse primary quantity
        let primary = self.parse_numeric()?;
        
        let secondary = if self.check(&JclTokenKind::Comma) {
            self.advance();
            Some(self.parse_numeric()?)
        } else {
            None
        };
        
        let directory = if self.check(&JclTokenKind::Comma) {
            self.advance();
            Some(self.parse_numeric()?)
        } else {
            None
        };
        
        self.expect(&JclTokenKind::RightParen)?;
        
        Ok(SpaceAllocation {
            unit,
            primary,
            secondary,
            directory,
            rlse: false,
        })
    }

    fn parse_dcb(&mut self) -> Result<Dcb> {
        self.expect(&JclTokenKind::LeftParen)?;
        
        let mut dcb = Dcb {
            recfm: None,
            lrecl: None,
            blksize: None,
            dsorg: None,
        };
        
        while !self.check(&JclTokenKind::RightParen) {
            if let Some(token) = self.peek() {
                if let JclTokenKind::Identifier(name) = &token.kind {
                    let param = name.to_uppercase();
                    self.advance();
                    
                    if self.check(&JclTokenKind::Equals) {
                        self.advance();
                        let value = self.parse_parameter_value()?;
                        
                        match param.as_str() {
                            "RECFM" => dcb.recfm = Some(value),
                            "LRECL" => dcb.lrecl = value.parse().ok(),
                            "BLKSIZE" => dcb.blksize = value.parse().ok(),
                            "DSORG" => dcb.dsorg = Some(value),
                            _ => {}
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
            
            self.skip_optional(&JclTokenKind::Comma);
        }
        
        self.expect(&JclTokenKind::RightParen)?;
        
        Ok(dcb)
    }

    fn parse_procedure(&mut self) -> Result<Procedure> {
        self.expect(&JclTokenKind::Proc)?;
        
        let name = self.expect_identifier()?;
        let mut parameters = Vec::new();
        let mut steps = Vec::new();
        
        // Parse procedure parameters
        while !self.check(&JclTokenKind::Newline) && !self.is_at_end() {
            if let Some(token) = self.peek() {
                if let JclTokenKind::Identifier(param) = &token.kind {
                    parameters.push(param.clone());
                    self.advance();
                } else {
                    break;
                }
            }
            self.skip_optional(&JclTokenKind::Comma);
        }
        
        // Parse procedure steps
        self.skip_noise();
        while !self.check(&JclTokenKind::Pend) && !self.is_at_end() {
            if self.check(&JclTokenKind::Exec) {
                steps.push(self.parse_step()?);
            } else {
                self.advance();
            }
            self.skip_noise();
        }
        
        self.skip_optional(&JclTokenKind::Pend);
        
        Ok(Procedure {
            name,
            parameters,
            steps,
        })
    }

    fn parse_conditional(&mut self) -> Result<Conditional> {
        self.expect(&JclTokenKind::If)?;
        
        let condition = self.parse_parameter_value()?;
        
        self.expect(&JclTokenKind::Then)?;
        
        let mut then_steps = Vec::new();
        self.skip_noise();
        
        while !self.check(&JclTokenKind::Else) && !self.check(&JclTokenKind::Endif) && !self.is_at_end() {
            if self.check(&JclTokenKind::Exec) {
                then_steps.push(self.parse_step()?);
            } else {
                self.advance();
            }
            self.skip_noise();
        }
        
        let else_steps = if self.check(&JclTokenKind::Else) {
            self.advance();
            let mut steps = Vec::new();
            self.skip_noise();
            
            while !self.check(&JclTokenKind::Endif) && !self.is_at_end() {
                if self.check(&JclTokenKind::Exec) {
                    steps.push(self.parse_step()?);
                } else {
                    self.advance();
                }
                self.skip_noise();
            }
            
            Some(steps)
        } else {
            None
        };
        
        self.skip_optional(&JclTokenKind::Endif);
        
        Ok(Conditional {
            condition,
            then_steps,
            else_steps,
        })
    }

    fn parse_parameter_value(&mut self) -> Result<String> {
        if let Some(token) = self.peek() {
            let value = match &token.kind {
                JclTokenKind::Identifier(s) => s.clone(),
                JclTokenKind::StringLiteral(s) => s.clone(),
                JclTokenKind::NumericLiteral(s) => s.clone(),
                JclTokenKind::SymbolicParameter(s) => s.clone(),
                _ => bail!("Expected parameter value"),
            };
            self.advance();
            Ok(value)
        } else {
            bail!("Expected parameter value")
        }
    }

    fn parse_numeric(&mut self) -> Result<usize> {
        if let Some(token) = self.peek() {
            if let JclTokenKind::NumericLiteral(n) = &token.kind {
                let value = n.parse()?;
                self.advance();
                Ok(value)
            } else {
                bail!("Expected numeric value")
            }
        } else {
            bail!("Expected numeric value")
        }
    }

    // Helper methods
    fn check(&self, kind: &JclTokenKind) -> bool {
        if let Some(token) = self.peek() {
            std::mem::discriminant(&token.kind) == std::mem::discriminant(kind)
        } else {
            false
        }
    }

    fn peek(&self) -> Option<&JclToken> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> Option<&JclToken> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1)
    }

    fn expect(&mut self, kind: &JclTokenKind) -> Result<()> {
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
            if let JclTokenKind::Identifier(name) = &token.kind {
                let result = name.clone();
                self.advance();
                return Ok(result);
            }
        }
        bail!("Expected identifier")
    }

    fn skip_optional(&mut self, kind: &JclTokenKind) {
        if self.check(kind) {
            self.advance();
        }
    }

    fn skip_noise(&mut self) {
        while let Some(token) = self.peek() {
            if matches!(token.kind, JclTokenKind::Newline | JclTokenKind::Comment(_)) {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || matches!(self.peek().map(|t| &t.kind), Some(JclTokenKind::Eof))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jcl::lexer::JclLexer;

    #[test]
    fn test_parse_simple_job() {
        let source = r#"
//MYJOB JOB CLASS=A
//STEP1 EXEC PGM=IEFBR14
//DD1 DD DSN=MY.DATA.SET,DISP=SHR
"#;
        let mut lexer = JclLexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = JclParser::new(tokens);
        let job = parser.parse().unwrap();
        
        assert_eq!(job.steps.len(), 1);
        assert_eq!(job.steps[0].dd_statements.len(), 1);
    }
}

// Made with Bob

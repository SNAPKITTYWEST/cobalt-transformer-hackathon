// JCL Abstract Syntax Tree
// Represents the structure of IBM JCL jobs

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct JclJob {
    pub name: String,
    pub job_card: JobCard,
    pub steps: Vec<JclStep>,
    pub procedures: Vec<Procedure>,
    pub conditionals: Vec<Conditional>,
}

#[derive(Debug, Clone)]
pub struct JobCard {
    pub name: String,
    pub accounting_info: Option<String>,
    pub programmer_name: Option<String>,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct JclStep {
    pub name: String,
    pub exec_statement: ExecStatement,
    pub dd_statements: Vec<DdStatement>,
    pub condition: Option<StepCondition>,
}

#[derive(Debug, Clone)]
pub struct ExecStatement {
    pub program: Option<String>,
    pub procedure: Option<String>,
    pub parameters: HashMap<String, String>,
    pub parm: Option<String>,
    pub cond: Option<String>,
    pub region: Option<String>,
    pub time: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DdStatement {
    pub name: String,
    pub dsn: Option<String>,
    pub disp: Option<Disposition>,
    pub space: Option<SpaceAllocation>,
    pub dcb: Option<Dcb>,
    pub unit: Option<String>,
    pub volume: Option<String>,
    pub sysout: Option<String>,
    pub dummy: bool,
    pub data: bool,
    pub dlm: Option<String>,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Disposition {
    pub status: DispositionStatus,
    pub normal_termination: DispositionAction,
    pub abnormal_termination: Option<DispositionAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispositionStatus {
    New,
    Old,
    Shr,
    Mod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispositionAction {
    Catlg,
    Delete,
    Keep,
    Pass,
    Uncatlg,
}

#[derive(Debug, Clone)]
pub struct SpaceAllocation {
    pub unit: SpaceUnit,
    pub primary: usize,
    pub secondary: Option<usize>,
    pub directory: Option<usize>,
    pub rlse: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceUnit {
    Trk,
    Cyl,
    Blk(usize),
}

#[derive(Debug, Clone)]
pub struct Dcb {
    pub recfm: Option<String>,
    pub lrecl: Option<usize>,
    pub blksize: Option<usize>,
    pub dsorg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Procedure {
    pub name: String,
    pub parameters: Vec<String>,
    pub steps: Vec<JclStep>,
}

#[derive(Debug, Clone)]
pub struct Conditional {
    pub condition: String,
    pub then_steps: Vec<JclStep>,
    pub else_steps: Option<Vec<JclStep>>,
}

#[derive(Debug, Clone)]
pub struct StepCondition {
    pub return_code: Option<i32>,
    pub step_name: Option<String>,
    pub operator: ConditionOperator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
}

impl JclJob {
    pub fn new(name: String) -> Self {
        Self {
            name,
            job_card: JobCard {
                name: String::new(),
                accounting_info: None,
                programmer_name: None,
                parameters: HashMap::new(),
            },
            steps: Vec::new(),
            procedures: Vec::new(),
            conditionals: Vec::new(),
        }
    }

    pub fn add_step(&mut self, step: JclStep) {
        self.steps.push(step);
    }

    pub fn add_procedure(&mut self, proc: Procedure) {
        self.procedures.push(proc);
    }

    pub fn get_step(&self, name: &str) -> Option<&JclStep> {
        self.steps.iter().find(|s| s.name == name)
    }

    pub fn get_procedure(&self, name: &str) -> Option<&Procedure> {
        self.procedures.iter().find(|p| p.name == name)
    }
}

impl JclStep {
    pub fn new(name: String, exec: ExecStatement) -> Self {
        Self {
            name,
            exec_statement: exec,
            dd_statements: Vec::new(),
            condition: None,
        }
    }

    pub fn add_dd(&mut self, dd: DdStatement) {
        self.dd_statements.push(dd);
    }

    pub fn get_dd(&self, name: &str) -> Option<&DdStatement> {
        self.dd_statements.iter().find(|dd| dd.name == name)
    }
}

impl ExecStatement {
    pub fn program(name: String) -> Self {
        Self {
            program: Some(name),
            procedure: None,
            parameters: HashMap::new(),
            parm: None,
            cond: None,
            region: None,
            time: None,
        }
    }

    pub fn procedure(name: String) -> Self {
        Self {
            program: None,
            procedure: Some(name),
            parameters: HashMap::new(),
            parm: None,
            cond: None,
            region: None,
            time: None,
        }
    }
}

impl DdStatement {
    pub fn new(name: String) -> Self {
        Self {
            name,
            dsn: None,
            disp: None,
            space: None,
            dcb: None,
            unit: None,
            volume: None,
            sysout: None,
            dummy: false,
            data: false,
            dlm: None,
            parameters: HashMap::new(),
        }
    }

    pub fn with_dsn(mut self, dsn: String) -> Self {
        self.dsn = Some(dsn);
        self
    }

    pub fn with_disp(mut self, disp: Disposition) -> Self {
        self.disp = Some(disp);
        self
    }

    pub fn with_sysout(mut self, class: String) -> Self {
        self.sysout = Some(class);
        self
    }
}

impl Disposition {
    pub fn new(status: DispositionStatus, normal: DispositionAction) -> Self {
        Self {
            status,
            normal_termination: normal,
            abnormal_termination: None,
        }
    }

    pub fn with_abnormal(mut self, abnormal: DispositionAction) -> Self {
        self.abnormal_termination = Some(abnormal);
        self
    }
}

// Made with Bob

use crate::source_map::SourceSpan;
use std::fmt;

#[derive(Debug, Clone)]
pub struct CobolProgram {
    pub identification: IdentificationDivision,
    pub environment: Option<EnvironmentDivision>,
    pub data: Option<DataDivision>,
    pub procedure: Option<ProcedureDivision>,
    pub span: SourceSpan,
}

impl CobolProgram {
    pub fn name(&self) -> &str {
        &self.identification.program_id
    }

    pub fn semantically_equivalent(&self, other: &CobolProgram) -> bool {
        // Simplified semantic equivalence check
        // In production, this would be much more sophisticated
        self.identification.program_id == other.identification.program_id
    }
}

#[derive(Debug, Clone)]
pub struct IdentificationDivision {
    pub program_id: String,
    pub author: Option<String>,
    pub installation: Option<String>,
    pub date_written: Option<String>,
    pub date_compiled: Option<String>,
    pub security: Option<String>,
    pub remarks: Option<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct EnvironmentDivision {
    pub configuration: Option<ConfigurationSection>,
    pub input_output: Option<InputOutputSection>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ConfigurationSection {
    pub source_computer: Option<String>,
    pub object_computer: Option<String>,
    pub special_names: Vec<SpecialName>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct SpecialName {
    pub name: String,
    pub value: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct InputOutputSection {
    pub file_control: Vec<FileControlEntry>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct FileControlEntry {
    pub select_name: String,
    pub assign_to: String,
    pub organization: Option<FileOrganization>,
    pub access_mode: Option<AccessMode>,
    pub record_key: Option<String>,
    pub alternate_keys: Vec<String>,
    pub file_status: Option<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileOrganization {
    Sequential,
    Relative,
    Indexed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessMode {
    Sequential,
    Random,
    Dynamic,
}

#[derive(Debug, Clone)]
pub struct DataDivision {
    pub file_section: Option<FileSection>,
    pub working_storage: Option<WorkingStorageSection>,
    pub local_storage: Option<LocalStorageSection>,
    pub linkage_section: Option<LinkageSection>,
    pub report_section: Option<ReportSection>,
    pub screen_section: Option<ScreenSection>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct FileSection {
    pub file_descriptions: Vec<FileDescription>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct FileDescription {
    pub name: String,
    pub records: Vec<DataItem>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct WorkingStorageSection {
    pub items: Vec<DataItem>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct LocalStorageSection {
    pub items: Vec<DataItem>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct LinkageSection {
    pub items: Vec<DataItem>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ReportSection {
    pub reports: Vec<ReportDescription>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ReportDescription {
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ScreenSection {
    pub screens: Vec<ScreenDescription>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ScreenDescription {
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct DataItem {
    pub level: u8,
    pub name: Option<String>,
    pub picture: Option<PictureClause>,
    pub usage: Option<Usage>,
    pub value: Option<Value>,
    pub redefines: Option<String>,
    pub renames: Option<RenamesClause>,
    pub occurs: Option<OccursClause>,
    pub sign_clause: Option<SignClause>,
    pub synchronized: bool,
    pub justified: bool,
    pub blank_when_zero: bool,
    pub external: bool,
    pub global: bool,
    pub based: bool,
    pub children: Vec<DataItem>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct PictureClause {
    pub picture_string: String,
    pub category: PictureCategory,
    pub size: usize,
    pub scale: i32,
    pub has_sign: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PictureCategory {
    Alphabetic,
    Alphanumeric,
    AlphanumericEdited,
    Numeric,
    NumericEdited,
    National,
    NationalEdited,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Usage {
    Display,
    Comp,
    Comp1,
    Comp2,
    Comp3,
    Comp4,
    Comp5,
    Binary,
    PackedDecimal,
    National,
    Index,
    Pointer,
    ObjectReference,
}

#[derive(Debug, Clone)]
pub enum Value {
    Literal(Literal),
    Figurative(FigurativeConstant),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Numeric(String),
    Alphanumeric(String),
    National(String),
    Hex(String),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FigurativeConstant {
    Zero,
    Space,
    HighValue,
    LowValue,
    Quote,
    Null,
}

#[derive(Debug, Clone)]
pub struct RenamesClause {
    pub from: String,
    pub through: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OccursClause {
    pub times: OccursTimes,
    pub depending_on: Option<String>,
    pub indexed_by: Vec<String>,
    pub keys: Vec<OccursKey>,
}

#[derive(Debug, Clone)]
pub enum OccursTimes {
    Fixed(usize),
    Variable { min: usize, max: usize },
}

#[derive(Debug, Clone)]
pub struct OccursKey {
    pub name: String,
    pub ascending: bool,
}

#[derive(Debug, Clone)]
pub struct SignClause {
    pub leading: bool,
    pub separate: bool,
}

#[derive(Debug, Clone)]
pub struct ProcedureDivision {
    pub using_clause: Vec<String>,
    pub returning_clause: Option<String>,
    pub declaratives: Vec<Declarative>,
    pub sections: Vec<Section>,
    pub paragraphs: Vec<Paragraph>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct Declarative {
    pub name: String,
    pub sections: Vec<Section>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub paragraphs: Vec<Paragraph>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct Paragraph {
    pub name: String,
    pub statements: Vec<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Accept(AcceptStatement),
    Add(AddStatement),
    Alter(AlterStatement),
    Call(CallStatement),
    Cancel(CancelStatement),
    Close(CloseStatement),
    Compute(ComputeStatement),
    Continue(ContinueStatement),
    Delete(DeleteStatement),
    Display(DisplayStatement),
    Divide(DivideStatement),
    Evaluate(EvaluateStatement),
    Exec(ExecStatement),
    Exit(ExitStatement),
    Go(GoStatement),
    GoBack(GoBackStatement),
    If(IfStatement),
    Initialize(InitializeStatement),
    Inspect(InspectStatement),
    Merge(MergeStatement),
    Move(MoveStatement),
    Multiply(MultiplyStatement),
    Open(OpenStatement),
    Perform(PerformStatement),
    Read(ReadStatement),
    Rewrite(RewriteStatement),
    Search(SearchStatement),
    Set(SetStatement),
    Sort(SortStatement),
    Start(StartStatement),
    Stop(StopStatement),
    String(StringStatement),
    Subtract(SubtractStatement),
    Unstring(UnstringStatement),
    Write(WriteStatement),
}

#[derive(Debug, Clone)]
pub struct AcceptStatement {
    pub target: Identifier,
    pub from: Option<AcceptFrom>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum AcceptFrom {
    Date,
    Day,
    DayOfWeek,
    Time,
    Console,
}

#[derive(Debug, Clone)]
pub struct AddStatement {
    pub operands: Vec<Expression>,
    pub to: Option<Vec<Identifier>>,
    pub giving: Option<Vec<Identifier>>,
    pub on_size_error: Option<Vec<Statement>>,
    pub not_on_size_error: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct AlterStatement {
    pub from_paragraph: String,
    pub to_paragraph: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct CallStatement {
    pub program: Expression,
    pub using: Vec<CallParameter>,
    pub returning: Option<Identifier>,
    pub on_exception: Option<Vec<Statement>>,
    pub not_on_exception: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct CallParameter {
    pub mode: ParameterMode,
    pub identifier: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterMode {
    ByReference,
    ByContent,
    ByValue,
}

#[derive(Debug, Clone)]
pub struct CancelStatement {
    pub programs: Vec<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct CloseStatement {
    pub files: Vec<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ComputeStatement {
    pub target: Identifier,
    pub expression: Expression,
    pub on_size_error: Option<Vec<Statement>>,
    pub not_on_size_error: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ContinueStatement {
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct DeleteStatement {
    pub file: String,
    pub invalid_key: Option<Vec<Statement>>,
    pub not_invalid_key: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct DisplayStatement {
    pub items: Vec<Expression>,
    pub upon: Option<DisplayUpon>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum DisplayUpon {
    Console,
    Printer,
    Syserr,
}

#[derive(Debug, Clone)]
pub struct DivideStatement {
    pub dividend: Expression,
    pub divisor: Expression,
    pub into: Option<Vec<Identifier>>,
    pub giving: Option<Vec<Identifier>>,
    pub remainder: Option<Identifier>,
    pub on_size_error: Option<Vec<Statement>>,
    pub not_on_size_error: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct EvaluateStatement {
    pub subjects: Vec<Expression>,
    pub when_clauses: Vec<WhenClause>,
    pub when_other: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct WhenClause {
    pub conditions: Vec<WhenCondition>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum WhenCondition {
    Value(Expression),
    Condition(Condition),
    Any,
}

#[derive(Debug, Clone)]
pub struct ExecStatement {
    pub exec_type: String,
    pub content: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ExitStatement {
    pub exit_type: ExitType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitType {
    Program,
    Perform,
    Section,
    Paragraph,
}

#[derive(Debug, Clone)]
pub struct GoStatement {
    pub target: GoTarget,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum GoTarget {
    To(String),
    Depending(Vec<String>, Identifier),
}

#[derive(Debug, Clone)]
pub struct GoBackStatement {
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Condition,
    pub then_statements: Vec<Statement>,
    pub else_statements: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct InitializeStatement {
    pub targets: Vec<Identifier>,
    pub replacing: Vec<InitializeReplacing>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct InitializeReplacing {
    pub category: InitializeCategory,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitializeCategory {
    Alphabetic,
    Alphanumeric,
    Numeric,
    AlphanumericEdited,
    NumericEdited,
}

#[derive(Debug, Clone)]
pub struct InspectStatement {
    pub target: Identifier,
    pub operation: InspectOperation,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum InspectOperation {
    Tallying(Vec<TallyingClause>),
    Replacing(Vec<ReplacingClause>),
    Converting(ConvertingClause),
}

#[derive(Debug, Clone)]
pub struct TallyingClause {
    pub counter: Identifier,
    pub items: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct ReplacingClause {
    pub from: Expression,
    pub to: Expression,
}

#[derive(Debug, Clone)]
pub struct ConvertingClause {
    pub from: Expression,
    pub to: Expression,
}

#[derive(Debug, Clone)]
pub struct MergeStatement {
    pub file: String,
    pub keys: Vec<MergeKey>,
    pub using: Vec<String>,
    pub output_procedure: Option<ProcedureRange>,
    pub giving: Option<Vec<String>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct MergeKey {
    pub name: String,
    pub ascending: bool,
}

#[derive(Debug, Clone)]
pub struct MoveStatement {
    pub source: Expression,
    pub targets: Vec<Identifier>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct MultiplyStatement {
    pub multiplicand: Expression,
    pub multiplier: Expression,
    pub by: Option<Vec<Identifier>>,
    pub giving: Option<Vec<Identifier>>,
    pub on_size_error: Option<Vec<Statement>>,
    pub not_on_size_error: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct OpenStatement {
    pub files: Vec<OpenFile>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct OpenFile {
    pub name: String,
    pub mode: OpenMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenMode {
    Input,
    Output,
    InputOutput,
    Extend,
}

#[derive(Debug, Clone)]
pub struct PerformStatement {
    pub target: PerformTarget,
    pub times: Option<Expression>,
    pub until: Option<Condition>,
    pub varying: Option<PerformVarying>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum PerformTarget {
    Inline(Vec<Statement>),
    Procedure(ProcedureRange),
}

#[derive(Debug, Clone)]
pub struct ProcedureRange {
    pub from: String,
    pub through: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PerformVarying {
    pub identifier: Identifier,
    pub from: Expression,
    pub by: Expression,
    pub until: Condition,
    pub after: Vec<PerformVarying>,
}

#[derive(Debug, Clone)]
pub struct ReadStatement {
    pub file: String,
    pub into: Option<Identifier>,
    pub key: Option<String>,
    pub at_end: Option<Vec<Statement>>,
    pub not_at_end: Option<Vec<Statement>>,
    pub invalid_key: Option<Vec<Statement>>,
    pub not_invalid_key: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct RewriteStatement {
    pub record: String,
    pub from: Option<Identifier>,
    pub invalid_key: Option<Vec<Statement>>,
    pub not_invalid_key: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct SearchStatement {
    pub target: Identifier,
    pub varying: Option<Identifier>,
    pub at_end: Option<Vec<Statement>>,
    pub when_clauses: Vec<SearchWhen>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct SearchWhen {
    pub condition: Condition,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct SetStatement {
    pub targets: Vec<SetTarget>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum SetTarget {
    To(Identifier, Expression),
    Up(Identifier, Expression),
    Down(Identifier, Expression),
}

#[derive(Debug, Clone)]
pub struct SortStatement {
    pub file: String,
    pub keys: Vec<SortKey>,
    pub input_procedure: Option<ProcedureRange>,
    pub using: Option<Vec<String>>,
    pub output_procedure: Option<ProcedureRange>,
    pub giving: Option<Vec<String>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct SortKey {
    pub name: String,
    pub ascending: bool,
}

#[derive(Debug, Clone)]
pub struct StartStatement {
    pub file: String,
    pub key: Option<StartKey>,
    pub invalid_key: Option<Vec<Statement>>,
    pub not_invalid_key: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct StartKey {
    pub name: String,
    pub relation: KeyRelation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyRelation {
    Equal,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
}

#[derive(Debug, Clone)]
pub struct StopStatement {
    pub stop_type: StopType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum StopType {
    Run,
    Literal(String),
}

#[derive(Debug, Clone)]
pub struct StringStatement {
    pub sources: Vec<StringSource>,
    pub into: Identifier,
    pub pointer: Option<Identifier>,
    pub on_overflow: Option<Vec<Statement>>,
    pub not_on_overflow: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct StringSource {
    pub value: Expression,
    pub delimited_by: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct SubtractStatement {
    pub operands: Vec<Expression>,
    pub from: Option<Vec<Identifier>>,
    pub giving: Option<Vec<Identifier>>,
    pub on_size_error: Option<Vec<Statement>>,
    pub not_on_size_error: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct UnstringStatement {
    pub source: Identifier,
    pub delimited_by: Option<Vec<Expression>>,
    pub into: Vec<UnstringTarget>,
    pub pointer: Option<Identifier>,
    pub tallying: Option<Identifier>,
    pub on_overflow: Option<Vec<Statement>>,
    pub not_on_overflow: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct UnstringTarget {
    pub identifier: Identifier,
    pub delimiter: Option<Identifier>,
    pub count: Option<Identifier>,
}

#[derive(Debug, Clone)]
pub struct WriteStatement {
    pub record: String,
    pub from: Option<Identifier>,
    pub advancing: Option<Advancing>,
    pub at_end_of_page: Option<Vec<Statement>>,
    pub not_at_end_of_page: Option<Vec<Statement>>,
    pub invalid_key: Option<Vec<Statement>>,
    pub not_invalid_key: Option<Vec<Statement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum Advancing {
    Lines(Expression),
    Page,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(Identifier),
    Figurative(FigurativeConstant),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
}

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
}

#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOperator {
    Plus,
    Minus,
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub name: String,
    pub qualification: Vec<String>,
    pub subscripts: Vec<Expression>,
    pub reference_modification: Option<ReferenceModification>,
    pub span: SourceSpan,
}

impl Identifier {
    pub fn simple(name: String, span: SourceSpan) -> Self {
        Self {
            name,
            qualification: Vec::new(),
            subscripts: Vec::new(),
            reference_modification: None,
            span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReferenceModification {
    pub start: Box<Expression>,
    pub length: Option<Box<Expression>>,
}

#[derive(Debug, Clone)]
pub enum Condition {
    Relation(RelationCondition),
    Class(ClassCondition),
    Sign(SignCondition),
    Condition88(Condition88),
    Complex(ComplexCondition),
    Not(Box<Condition>),
}

#[derive(Debug, Clone)]
pub struct RelationCondition {
    pub left: Expression,
    pub operator: RelationOperator,
    pub right: Expression,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationOperator {
    Equal,
    NotEqual,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
}

#[derive(Debug, Clone)]
pub struct ClassCondition {
    pub identifier: Identifier,
    pub class: ClassType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassType {
    Numeric,
    Alphabetic,
    AlphanumericUpper,
    AlphanumericLower,
}

#[derive(Debug, Clone)]
pub struct SignCondition {
    pub identifier: Identifier,
    pub sign: SignType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignType {
    Positive,
    Negative,
    Zero,
}

#[derive(Debug, Clone)]
pub struct Condition88 {
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ComplexCondition {
    pub left: Box<Condition>,
    pub operator: LogicalOperator,
    pub right: Box<Condition>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicalOperator {
    And,
    Or,
}

impl fmt::Display for CobolProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "COBOL Program: {}", self.name())
    }
}

// Made with Bob

#![allow(dead_code)]
//! AST (Abstract Syntax Tree) 定義

/// プログラム全体
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

/// トップレベルの要素
#[derive(Debug, Clone)]
pub enum Item {
    FunctionDef(FunctionDef),
    ClassDef(ClassDef),
    ComponentDef(ComponentDef),
    ServerDef(ServerDef),
    Import(ImportStmt),
    Statement(Statement),
}

/// Import文
#[derive(Debug, Clone)]
pub struct ImportStmt {
    pub module: String,
    pub names: Vec<String>,    // from X import A, B, C
    pub alias: Option<String>, // import X as Y
}

/// 関数定義
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
    pub is_async: bool,
}

/// パラメータ
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_annotation: Option<Type>,
}

/// 型
#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Set(Box<Type>),
    Fn(Vec<Type>, Box<Type>), // Fn[Params] -> RetType
    Custom(String),
}

/// 文
#[derive(Debug, Clone)]
pub enum Statement {
    Let(LetDecl),
    Const(ConstDecl),
    Return(Option<Expression>),
    Expression(Expression),
    If(IfStmt),
    For(ForStmt),
    While(WhileStmt),
    Match(MatchStmt),
    Break,
    Continue,
    // コンポーネント用
    State(StateDecl),
    Render(RenderBlock),
    // 代入
    Assignment(AssignmentStmt),
}

/// 変数宣言 (let, 変更可能)
#[derive(Debug, Clone)]
pub struct LetDecl {
    pub name: String,
    pub value: Expression,
    pub type_annotation: Option<Type>,
}

/// 定数宣言 (const, 変更不可)
#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub value: Expression,
    pub type_annotation: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct AssignmentStmt {
    pub target: Expression,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct StateDecl {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct RenderBlock {
    pub body: Vec<Statement>,
}

/// If文
#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expression,
    pub then_block: Vec<Statement>,
    pub else_block: Option<Vec<Statement>>,
}

/// For文
#[derive(Debug, Clone)]
pub struct ForStmt {
    pub target: String,
    pub iterator: Expression,
    pub body: Vec<Statement>,
}

/// While文
#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expression,
    pub body: Vec<Statement>,
}

/// Match文 (パターンマッチ)
#[derive(Debug, Clone)]
pub struct MatchStmt {
    pub value: Expression,
    pub cases: Vec<MatchCase>,
}

#[derive(Debug, Clone)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Identifier(String), // 変数にバインド
    Wildcard,           // _
    Range(i64, i64),    // 1..10
}

/// 式
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    BinaryOp(Box<BinaryExpr>),
    UnaryOp(Box<UnaryExpr>),
    Call(Box<CallExpr>),
    MemberAccess(Box<MemberExpr>),
    Index(Box<IndexExpr>),
    Lambda(Box<LambdaExpr>),
    Await(Box<Expression>),
    JsxElement(Box<JsxElement>),
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Expression,
    pub op: BinaryOp,
    pub right: Expression,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    In, // x in list
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub operand: Expression,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg, // -x
    Not, // not x
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub func: Expression,
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct MemberExpr {
    pub object: Expression,
    pub member: String,
}

#[derive(Debug, Clone)]
pub struct IndexExpr {
    pub object: Expression,
    pub index: Expression,
}

/// ラムダ式: x -> x * 2 or (a, b) -> a + b
#[derive(Debug, Clone)]
pub struct LambdaExpr {
    pub params: Vec<String>,
    pub body: Expression,
}

/// リテラル
#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Vec<Expression>),
    Dict(Vec<(Expression, Expression)>),
    Set(Vec<Expression>),
    None,
}

// ===== クラス・コンポーネント定義など =====

#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub parent: Option<String>, // 継承
    pub body: Vec<ClassBodyItem>,
}

#[derive(Debug, Clone)]
pub enum ClassBodyItem {
    Field(FieldDef),
    Method(FunctionDef),
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub type_annotation: Type,
}

#[derive(Debug, Clone)]
pub struct ComponentDef {
    pub name: String,
    pub body: Vec<ComponentBodyItem>,
}

#[derive(Debug, Clone)]
pub enum ComponentBodyItem {
    State(StateDecl),
    Method(FunctionDef),
    Render(RenderBlock),
}

#[derive(Debug, Clone)]
pub struct ServerDef {
    pub name: String,
    pub body: Vec<ServerBodyItem>,
}

#[derive(Debug, Clone)]
pub enum ServerBodyItem {
    Route(RouteDef),
}

#[derive(Debug, Clone)]
pub struct RouteDef {
    pub path: String,
    pub method: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct JsxElement {
    pub tag: String,
    pub attributes: Vec<JsxAttribute>,
    pub children: Vec<JsxChild>,
}

#[derive(Debug, Clone)]
pub struct JsxAttribute {
    pub name: String,
    pub value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub enum JsxChild {
    Element(JsxElement),
    Text(String),
    Expression(Expression),
}

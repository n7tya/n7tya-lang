//! Interpreter for n7tya-lang
//!
//! ASTを直接評価するTree-Walkingインタプリタ

use crate::ast::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// 実行時の値
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    None,
    Fn(Rc<FunctionDef>, Rc<RefCell<Env>>), // クロージャ
    BuiltinFn(String),
    Class(String, HashMap<String, Value>), // クラスインスタンス
    Return(Box<Value>),                    // return文の値（制御フロー用）
}

impl Value {
    /// 値を文字列として表示
    pub fn display(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Str(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::List(items) => {
                let strs: Vec<String> = items.iter().map(|v| v.display()).collect();
                format!("[{}]", strs.join(", "))
            }
            Value::None => "none".to_string(),
            Value::Fn(f, _) => format!("<fn {}>", f.name),
            Value::BuiltinFn(name) => format!("<builtin {}>", name),
            Value::Class(name, _) => format!("<{} instance>", name),
            Value::Return(v) => v.display(),
        }
    }

    /// 真偽値として評価
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::None => false,
            _ => true,
        }
    }
}

/// 環境（変数バインディング）
#[derive(Debug, Clone)]
pub struct Env {
    values: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Env>>) -> Self {
        Self {
            values: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.values.get(name) {
            Some(v.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: &str, value: Value) -> bool {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            true
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().set(name, value)
        } else {
            false
        }
    }
}

/// インタプリタ
pub struct Interpreter {
    env: Rc<RefCell<Env>>,
    output: Vec<String>, // printの出力を格納
}

impl Interpreter {
    pub fn new() -> Self {
        let env = Rc::new(RefCell::new(Env::new()));

        // 組み込み関数を登録
        let builtins = [
            "print", "println", "len", "range", "input", "str", "int", "float", "type", "abs",
            "min", "max",
        ];
        for name in builtins {
            env.borrow_mut()
                .define(name, Value::BuiltinFn(name.to_string()));
        }

        Self {
            env,
            output: Vec::new(),
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<Value, String> {
        let mut result = Value::None;

        for item in &program.items {
            result = self.eval_item(item)?;

            // Return値が出たら終了
            if let Value::Return(v) = result {
                return Ok(*v);
            }
        }

        Ok(result)
    }

    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    fn eval_item(&mut self, item: &Item) -> Result<Value, String> {
        match item {
            Item::FunctionDef(f) => {
                let func = Value::Fn(Rc::new(f.clone()), self.env.clone());
                self.env.borrow_mut().define(&f.name, func);
                Ok(Value::None)
            }
            Item::ClassDef(c) => {
                self.env
                    .borrow_mut()
                    .define(&c.name, Value::BuiltinFn(format!("__class_{}", c.name)));
                Ok(Value::None)
            }
            Item::ComponentDef(_) => Ok(Value::None),
            Item::ServerDef(_) => Ok(Value::None),
            Item::Import(_) => Ok(Value::None), // importは実行時には何もしない
            Item::Statement(stmt) => self.eval_statement(stmt),
        }
    }

    fn eval_statement(&mut self, stmt: &Statement) -> Result<Value, String> {
        match stmt {
            Statement::Let(decl) => {
                let value = self.eval_expression(&decl.value)?;
                self.env.borrow_mut().define(&decl.name, value);
                Ok(Value::None)
            }
            Statement::Const(decl) => {
                let value = self.eval_expression(&decl.value)?;
                self.env.borrow_mut().define(&decl.name, value);
                Ok(Value::None)
            }
            Statement::Assignment(a) => {
                let value = self.eval_expression(&a.value)?;
                if let Expression::Identifier(name) = &a.target {
                    if !self.env.borrow_mut().set(name, value.clone()) {
                        self.env.borrow_mut().define(name, value);
                    }
                }
                Ok(Value::None)
            }
            Statement::Return(expr) => {
                let value = if let Some(e) = expr {
                    self.eval_expression(e)?
                } else {
                    Value::None
                };
                Ok(Value::Return(Box::new(value)))
            }
            Statement::If(if_stmt) => {
                let cond = self.eval_expression(&if_stmt.condition)?;
                if cond.is_truthy() {
                    for s in &if_stmt.then_block {
                        let result = self.eval_statement(s)?;
                        if matches!(result, Value::Return(_)) {
                            return Ok(result);
                        }
                    }
                } else if let Some(else_block) = &if_stmt.else_block {
                    for s in else_block {
                        let result = self.eval_statement(s)?;
                        if matches!(result, Value::Return(_)) {
                            return Ok(result);
                        }
                    }
                }
                Ok(Value::None)
            }
            Statement::While(w) => {
                while self.eval_expression(&w.condition)?.is_truthy() {
                    for s in &w.body {
                        let result = self.eval_statement(s)?;
                        if matches!(result, Value::Return(_)) {
                            return Ok(result);
                        }
                    }
                }
                Ok(Value::None)
            }
            Statement::For(f) => {
                let iter_val = self.eval_expression(&f.iterator)?;
                if let Value::List(items) = iter_val {
                    for item in items {
                        self.env.borrow_mut().define(&f.target, item);
                        for s in &f.body {
                            let result = self.eval_statement(s)?;
                            if matches!(result, Value::Return(_)) {
                                return Ok(result);
                            }
                        }
                    }
                }
                Ok(Value::None)
            }
            Statement::Match(m) => {
                let value = self.eval_expression(&m.value)?;
                for case in &m.cases {
                    if self.pattern_matches(&case.pattern, &value) {
                        for s in &case.body {
                            let result = self.eval_statement(s)?;
                            if matches!(result, Value::Return(_)) {
                                return Ok(result);
                            }
                        }
                        break;
                    }
                }
                Ok(Value::None)
            }
            Statement::Break => Ok(Value::None), // TODO: ループ制御
            Statement::Continue => Ok(Value::None),
            Statement::Expression(e) => self.eval_expression(e),
            Statement::State(s) => {
                let value = self.eval_expression(&s.value)?;
                self.env.borrow_mut().define(&s.name, value);
                Ok(Value::None)
            }
            Statement::Render(_) => Ok(Value::None),
        }
    }

    fn pattern_matches(&self, pattern: &Pattern, value: &Value) -> bool {
        match pattern {
            Pattern::Wildcard => true,
            Pattern::Literal(Literal::Int(n)) => matches!(value, Value::Int(v) if v == n),
            Pattern::Literal(Literal::Str(s)) => matches!(value, Value::Str(v) if v == s),
            Pattern::Literal(Literal::Bool(b)) => matches!(value, Value::Bool(v) if v == b),
            Pattern::Identifier(_) => true, // バインド（常にマッチ）
            _ => false,
        }
    }

    fn eval_expression(&mut self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Literal(lit) => self.eval_literal(lit),
            Expression::Identifier(name) => self
                .env
                .borrow()
                .get(name)
                .ok_or_else(|| format!("Undefined variable: {}", name)),
            Expression::BinaryOp(bin) => {
                let left = self.eval_expression(&bin.left)?;
                let right = self.eval_expression(&bin.right)?;
                self.eval_binary_op(&bin.op, left, right)
            }
            Expression::UnaryOp(unary) => {
                let operand = self.eval_expression(&unary.operand)?;
                match unary.op {
                    UnaryOp::Neg => match operand {
                        Value::Int(n) => Ok(Value::Int(-n)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(format!("Cannot negate {:?}", operand)),
                    },
                    UnaryOp::Not => Ok(Value::Bool(!operand.is_truthy())),
                }
            }
            Expression::Call(call) => {
                let callee = self.eval_expression(&call.func)?;
                let mut args = Vec::new();
                for arg in &call.args {
                    args.push(self.eval_expression(arg)?);
                }
                self.call_function(callee, args)
            }
            Expression::MemberAccess(m) => {
                let obj = self.eval_expression(&m.object)?;
                if let Value::Class(_, fields) = obj {
                    fields
                        .get(&m.member)
                        .cloned()
                        .ok_or_else(|| format!("Unknown member: {}", m.member))
                } else {
                    Err(format!("Cannot access member of {:?}", obj))
                }
            }
            Expression::Index(idx) => {
                let obj = self.eval_expression(&idx.object)?;
                let index = self.eval_expression(&idx.index)?;
                match (obj, index) {
                    (Value::List(items), Value::Int(i)) => items
                        .get(i as usize)
                        .cloned()
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    (Value::Str(s), Value::Int(i)) => s
                        .chars()
                        .nth(i as usize)
                        .map(|c| Value::Str(c.to_string()))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    _ => Err("Invalid index operation".to_string()),
                }
            }
            Expression::Lambda(_) => Ok(Value::None), // TODO: Lambda実装
            Expression::Await(inner) => self.eval_expression(inner), // asyncは同期的に実行
            Expression::JsxElement(_) => Ok(Value::None),
        }
    }

    fn eval_literal(&mut self, lit: &Literal) -> Result<Value, String> {
        Ok(match lit {
            Literal::Int(n) => Value::Int(*n),
            Literal::Float(f) => Value::Float(*f),
            Literal::Str(s) => Value::Str(s.clone()),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::None => Value::None,
            Literal::List(items) => {
                let mut values = Vec::new();
                for item in items {
                    values.push(self.eval_expression(item)?);
                }
                Value::List(values)
            }
            Literal::Dict(_) => Value::None, // TODO: Dict実装
            Literal::Set(_) => Value::None,  // TODO: Set実装
        })
    }

    fn eval_binary_op(&self, op: &BinaryOp, left: Value, right: Value) -> Result<Value, String> {
        match (op, &left, &right) {
            // 算術演算
            (BinaryOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (BinaryOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (BinaryOp::Add, Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
            (BinaryOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (BinaryOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (BinaryOp::Div, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (BinaryOp::Mod, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),

            // 比較演算
            (BinaryOp::Eq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
            (BinaryOp::Eq, Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a == b)),
            (BinaryOp::Eq, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            (BinaryOp::Ne, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
            (BinaryOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (BinaryOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (BinaryOp::Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (BinaryOp::Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),

            // 論理演算
            (BinaryOp::And, _, _) => Ok(Value::Bool(left.is_truthy() && right.is_truthy())),
            (BinaryOp::Or, _, _) => Ok(Value::Bool(left.is_truthy() || right.is_truthy())),

            _ => Err(format!(
                "Unsupported operation: {:?} {:?} {:?}",
                left, op, right
            )),
        }
    }

    fn call_function(&mut self, callee: Value, args: Vec<Value>) -> Result<Value, String> {
        match callee {
            Value::Fn(func, closure_env) => {
                // 新しいスコープを作成
                let local_env = Rc::new(RefCell::new(Env::with_parent(closure_env)));

                // 引数をバインド
                for (param, arg) in func.params.iter().zip(args.iter()) {
                    local_env.borrow_mut().define(&param.name, arg.clone());
                }

                // 関数を評価
                let old_env = self.env.clone();
                self.env = local_env;

                let mut result = Value::None;
                for stmt in &func.body {
                    result = self.eval_statement(stmt)?;
                    if let Value::Return(v) = result {
                        self.env = old_env;
                        return Ok(*v);
                    }
                }

                self.env = old_env;
                Ok(result)
            }
            Value::BuiltinFn(name) => self.call_builtin(&name, args),
            _ => Err(format!("Cannot call {:?}", callee)),
        }
    }

    fn call_builtin(&mut self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        crate::builtins::call_builtin(name, args)
    }
}

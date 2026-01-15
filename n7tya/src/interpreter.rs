#![allow(dead_code)]
//! Interpreter for n7tya-lang
//!
//! ASTを直接評価するTree-Walkingインタプリタ

use crate::ast::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::rc::Rc;

/// 実行時の値
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Rc<RefCell<Vec<Value>>>),
    None,
    Fn(Rc<FunctionDef>, Rc<RefCell<Env>>), // クロージャ
    BuiltinFn(String),
    Class(String, Rc<RefCell<HashMap<String, Value>>>), // クラスインスタンス
    Dict(Rc<RefCell<HashMap<String, Value>>>),          // 辞書
    Set(Rc<RefCell<Vec<Value>>>),                       // 集合
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
                let items = items.borrow();
                let strs: Vec<String> = items.iter().map(|v| v.display()).collect();
                format!("[{}]", strs.join(", "))
            }
            Value::None => "none".to_string(),
            Value::Fn(f, _) => format!("<fn {}>", f.name),
            Value::BuiltinFn(name) => format!("<builtin {}>", name),
            Value::Class(name, _) => format!("<{} instance>", name),
            Value::Dict(map) => {
                let map = map.borrow();
                let strs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.display()))
                    .collect();
                format!("{{{}}}", strs.join(", "))
            }
            Value::Set(set) => {
                let set = set.borrow();
                let strs: Vec<String> = set.iter().map(|v| v.display()).collect();
                format!("{{{}}}", strs.join(", "))
            }
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
            Value::List(l) => !l.borrow().is_empty(),
            Value::Dict(d) => !d.borrow().is_empty(),
            Value::Set(s) => !s.borrow().is_empty(),
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
            "min", "max", "sum", "sorted", "reversed", "enumerate", "zip",
            // fs モジュール
            "fs.read_file", "fs.write_file", "fs.exists", "fs.remove", "fs.read_dir",
            // json モジュール
            "json.parse", "json.stringify",
            // http モジュール
            "http.get", "http.post",
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

    pub fn run_server(&mut self, server_def: &ServerDef) -> Result<(), String> {
        let port = 8080;
        let addr = format!("127.0.0.1:{}", port);

        let listener =
            TcpListener::bind(&addr).map_err(|e| format!("Failed to bind port {}: {}", port, e))?;
        println!("Server '{}' listening on http://{}", server_def.name, addr);

        // サーバー自体の環境（グローバル環境のコピーなど）を保持したい場合はここで用意
        // 現状はリクエストごとにグローバルのクローンから開始する形にする
        let global_env = self.env.clone();

        for stream in listener.incoming() {
            let mut stream = stream.map_err(|e| format!("Connection failed: {}", e))?;

            let mut buffer = [0; 1024];
            if stream.read(&mut buffer).is_err() {
                continue;
            }

            let request = String::from_utf8_lossy(&buffer);
            let first_line = request.lines().next().unwrap_or("");
            let parts: Vec<&str> = first_line.split_whitespace().collect();

            let mut response_body = "Not Found".to_string();
            let mut status = "404 Not Found";

            if parts.len() >= 2 {
                let method = parts[0];
                let path = parts[1];

                for item in &server_def.body {
                    let crate::ast::ServerBodyItem::Route(route) = item;
                    if route.method.eq_ignore_ascii_case(method) && route.path == path {
                        // ルートマッチ -> 新しいスコープで実行
                        let request_env =
                            Rc::new(RefCell::new(Env::with_parent(global_env.clone())));
                        self.env = request_env;

                        // リクエスト情報を変数として定義する（将来的な拡張）
                        // self.env.borrow_mut().define("request_path", Value::Str(path.to_string()));

                        let mut route_result = Value::None;
                        for stmt in &route.body {
                            match self.eval_statement(stmt) {
                                Ok(ExecutionResult::Return(v)) => {
                                    route_result = v;
                                    break;
                                }
                                Ok(ExecutionResult::Value(_)) => {}
                                Ok(_) => {} // Break/Continue not valid here
                                Err(e) => {
                                    println!("Error in route handler: {}", e);
                                    status = "500 Internal Server Error";
                                    response_body = format!("Error: {}", e);
                                    break;
                                }
                            }
                        }

                        // Returnされた値があればレスポンスにする
                        if status == "404 Not Found" {
                            // エラーでなければ
                            status = "200 OK"; // デフォルト200
                            if let Value::Str(s) = route_result {
                                response_body = s;
                            } else if let Value::None = route_result {
                                // 何も返さなかった場合は空、あるいはデフォルトメッセージ
                                if response_body == "Not Found" {
                                    response_body = "OK".to_string();
                                }
                            } else {
                                // 文字列以外は文字列化
                                response_body = route_result.display();
                            }
                        }

                        break;
                    }
                }
            }

            let response = format!(
                "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
                status,
                response_body.len(),
                response_body
            );

            stream.write_all(response.as_bytes()).ok();
            stream.flush().ok();
        }

        // Server loop never ends normally usually, but if break loop
        self.env = global_env; // Restore env
        Ok(())
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
            Item::ComponentDef(c) => {
                // コンポーネント定義を環境に登録 (将来的に使用)
                self.env
                    .borrow_mut()
                    .define(&c.name, Value::BuiltinFn(format!("__component_{}", c.name)));
                Ok(Value::None)
            }
            Item::ServerDef(s) => {
                // サーバー定義を実行 (簡易HTTPサーバー起動)
                self.run_server(s)?;
                Ok(Value::None)
            }
            Item::Import(_) => Ok(Value::None),
            Item::Statement(stmt) => self.eval_statement(stmt).map(|res| match res {
                ExecutionResult::Value(v) => v,
                ExecutionResult::Return(v) => v, // トップレベルでのreturnは値として扱う
                _ => Value::None,
            }),
        }
    }

    fn eval_statement(&mut self, stmt: &Statement) -> Result<ExecutionResult, String> {
        match stmt {
            Statement::Let(decl) => {
                let value = self.eval_expression(&decl.value)?;
                self.env.borrow_mut().define(&decl.name, value);
                Ok(ExecutionResult::Value(Value::None))
            }
            Statement::Const(decl) => {
                let value = self.eval_expression(&decl.value)?;
                self.env.borrow_mut().define(&decl.name, value);
                Ok(ExecutionResult::Value(Value::None))
            }
            Statement::Assignment(a) => {
                let value = self.eval_expression(&a.value)?;
                if let Expression::Identifier(name) = &a.target {
                    if !self.env.borrow_mut().set(name, value.clone()) {
                        self.env.borrow_mut().define(name, value);
                    }
                }
                Ok(ExecutionResult::Value(Value::None))
            }
            Statement::Return(expr) => {
                let value = if let Some(e) = expr {
                    self.eval_expression(e)?
                } else {
                    Value::None
                };
                Ok(ExecutionResult::Return(value))
            }
            Statement::If(if_stmt) => {
                let cond = self.eval_expression(&if_stmt.condition)?;
                if cond.is_truthy() {
                    for s in &if_stmt.then_block {
                        let result = self.eval_statement(s)?;
                        if !matches!(result, ExecutionResult::Value(_)) {
                            return Ok(result);
                        }
                    }
                } else if let Some(else_block) = &if_stmt.else_block {
                    for s in else_block {
                        let result = self.eval_statement(s)?;
                        if !matches!(result, ExecutionResult::Value(_)) {
                            return Ok(result);
                        }
                    }
                }
                Ok(ExecutionResult::Value(Value::None))
            }
            Statement::While(w) => {
                while self.eval_expression(&w.condition)?.is_truthy() {
                    for s in &w.body {
                        let result = self.eval_statement(s)?;
                        match result {
                            ExecutionResult::Return(_) => return Ok(result),
                            ExecutionResult::Break => {
                                return Ok(ExecutionResult::Value(Value::None))
                            }
                            ExecutionResult::Continue => break, // 内側のforループを抜けてwhile再評価へ (Rustの挙動とは違うが、この実装ではstmtループを抜ける必要がある)
                            _ => {}
                        }
                    }
                }
                Ok(ExecutionResult::Value(Value::None))
            }
            Statement::For(f) => {
                let iter_val = self.eval_expression(&f.iterator)?;
                if let Value::List(items) = iter_val {
                    let items_vec = items.borrow().clone();
                    for item in items_vec {
                        self.env.borrow_mut().define(&f.target, item);
                        for s in &f.body {
                            let result = self.eval_statement(s)?;
                            match result {
                                ExecutionResult::Return(_) => return Ok(result),
                                ExecutionResult::Break => {
                                    return Ok(ExecutionResult::Value(Value::None))
                                }
                                ExecutionResult::Continue => break,
                                _ => {}
                            }
                        }
                    }
                }
                Ok(ExecutionResult::Value(Value::None))
            }
            Statement::Match(m) => {
                let value = self.eval_expression(&m.value)?;
                for case in &m.cases {
                    if self.pattern_matches(&case.pattern, &value) {
                        // パターン変数のバインド
                        if let Pattern::Identifier(name) = &case.pattern {
                            self.env.borrow_mut().define(name, value.clone());
                        }

                        for s in &case.body {
                            let result = self.eval_statement(s)?;
                            if !matches!(result, ExecutionResult::Value(_)) {
                                return Ok(result);
                            }
                        }
                        break;
                    }
                }
                Ok(ExecutionResult::Value(Value::None))
            }
            Statement::Break => Ok(ExecutionResult::Break),
            Statement::Continue => Ok(ExecutionResult::Continue),
            Statement::Expression(e) => {
                let v = self.eval_expression(e)?;
                Ok(ExecutionResult::Value(v))
            }
            Statement::State(s) => {
                let value = self.eval_expression(&s.value)?;
                self.env.borrow_mut().define(&s.name, value);
                Ok(ExecutionResult::Value(Value::None))
            }
            Statement::Render(_) => Ok(ExecutionResult::Value(Value::None)), // Renderはコンポーネント内でのみ意味を持つが、実行は可能
        }
    }

    fn pattern_matches(&self, pattern: &Pattern, value: &Value) -> bool {
        match pattern {
            Pattern::Wildcard => true,
            Pattern::Literal(Literal::Int(n)) => matches!(value, Value::Int(v) if v == n),
            Pattern::Literal(Literal::Str(s)) => matches!(value, Value::Str(v) if v == s),
            Pattern::Literal(Literal::Bool(b)) => matches!(value, Value::Bool(v) if v == b),
            Pattern::Identifier(_) => true,
            _ => false,
        }
    }

    pub(crate) fn eval_expression(&mut self, expr: &Expression) -> Result<Value, String> {
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
                // メソッド呼び出しの特別処理
                if let Expression::MemberAccess(member) = &call.func {
                    // モジュール関数かどうかをチェック (fs.read_file など)
                    if let Expression::Identifier(module_name) = &member.object {
                        let full_name = format!("{}.{}", module_name, member.member);
                        // ビルトイン関数として存在するかチェック
                        let is_module_fn = matches!(self.env.borrow().get(&full_name), Some(Value::BuiltinFn(_)));
                        if is_module_fn {
                            let mut args = Vec::new();
                            for arg in &call.args {
                                args.push(self.eval_expression(arg)?);
                            }
                            return self.call_builtin(&full_name, args);
                        }
                    }
                    
                    // 通常のメソッド呼び出し
                    let obj = self.eval_expression(&member.object)?;
                    let method_name = &member.member;
                    let mut args = Vec::new();
                    for arg in &call.args {
                        args.push(self.eval_expression(arg)?);
                    }
                    return self.call_method(obj, method_name, args);
                }

                let callee = self.eval_expression(&call.func)?;
                let mut args = Vec::new();
                for arg in &call.args {
                    args.push(self.eval_expression(arg)?);
                }
                self.call_function(callee, args)
            }
            Expression::MemberAccess(m) => {
                let obj = self.eval_expression(&m.object)?;
                match obj {
                    Value::Class(_, fields) => fields
                        .borrow()
                        .get(&m.member)
                        .cloned()
                        .ok_or_else(|| format!("Unknown member: {}", m.member)),
                    Value::Dict(dict) => dict
                        .borrow()
                        .get(&m.member)
                        .cloned()
                        .ok_or_else(|| format!("Key error: {}", m.member)),
                    _ => Err(format!("Cannot access member of {:?}", obj)),
                }
            }
            Expression::Index(idx) => {
                let obj = self.eval_expression(&idx.object)?;
                let index = self.eval_expression(&idx.index)?;
                match (obj, index) {
                    (Value::List(items), Value::Int(i)) => items
                        .borrow()
                        .get(i as usize)
                        .cloned()
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    (Value::Str(s), Value::Int(i)) => s
                        .chars()
                        .nth(i as usize)
                        .map(|c| Value::Str(c.to_string()))
                        .ok_or_else(|| "Index out of bounds".to_string()),
                    (Value::Dict(dict), Value::Str(k)) => dict
                        .borrow()
                        .get(&k)
                        .cloned()
                        .ok_or_else(|| format!("Key error: {}", k)),
                    _ => Err("Invalid index operation".to_string()),
                }
            }
            Expression::Lambda(lambda) => {
                // Lambda式: params, body field needs to be converted to FunctionDef-like structure
                // LambdaExpr has params: Vec<String>, body: Expression
                // FunctionDef has body: Vec<Statement>
                // We wrap expression in Statement::Return or Statement::Expression
                let body_stmts = vec![Statement::Return(Some(lambda.body.clone()))];

                let func_def = FunctionDef {
                    name: "lambda".to_string(), // Anonymous
                    params: lambda
                        .params
                        .iter()
                        .map(|p| Param {
                            name: p.clone(),
                            type_annotation: None,
                        })
                        .collect(),
                    return_type: None,
                    body: body_stmts,
                    is_async: false,
                };

                Ok(Value::Fn(Rc::new(func_def), self.env.clone()))
            }
            Expression::Await(inner) => self.eval_expression(inner),
            Expression::JsxElement(element) => {
                crate::jsx_render::render_jsx(element, self).map(Value::Str)
            }
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
                Value::List(Rc::new(RefCell::new(values)))
            }
            Literal::Dict(items) => {
                let mut map = HashMap::new();
                for (k, v) in items {
                    let key = self.eval_expression(k)?;
                    let value = self.eval_expression(v)?;
                    if let Value::Str(s) = key {
                        map.insert(s, value);
                    } else {
                        return Err("Dict keys must be strings".to_string());
                    }
                }
                Value::Dict(Rc::new(RefCell::new(map)))
            }
            Literal::Set(items) => {
                // Set implementation using Vec for simplicity (or HashSet if Value is Hashable)
                // Since Value contains f64, it's not strictly Hashable. Using Vec for now.
                let mut values = Vec::new();
                for item in items {
                    values.push(self.eval_expression(item)?);
                }
                Value::Set(Rc::new(RefCell::new(values)))
            }
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

            // In 演算子
            (BinaryOp::In, _, Value::List(list)) => Ok(Value::Bool(
                list.borrow().iter().any(|v| self.values_equal(&left, v)),
            )),
            (BinaryOp::In, Value::Str(sub), Value::Str(s)) => Ok(Value::Bool(s.contains(sub))),

            _ => Err(format!(
                "Unsupported operation: {:?} {:?} {:?}",
                left, op, right
            )),
        }
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Str(x), Value::Str(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            // List/Dict/Setの比較はリファレンス等価性か中身か？ Pythonは中身。
            // ここでは簡易的にfalseにしておくか、再帰比較する。
            // 一旦RefCell比較はアドレス比較(同じオブジェクトか)にするのが簡単だが、
            // userは [1] == [1] を期待する。
            // 簡易比較として実装せず、とりあえずfalse
            _ => false, 
        }
    }

    fn call_function(&mut self, callee: Value, args: Vec<Value>) -> Result<Value, String> {
        match callee {
            Value::Fn(func, closure_env) => {
                // 新しいスコープを作成
                let local_env = Rc::new(RefCell::new(Env::with_parent(closure_env)));

                // 引数をバインド
                if args.len() != func.params.len() {
                    return Err(format!(
                        "Expected {} arguments, got {}",
                        func.params.len(),
                        args.len()
                    ));
                }

                for (param, arg) in func.params.iter().zip(args.iter()) {
                    local_env.borrow_mut().define(&param.name, arg.clone());
                }

                // 関数を評価
                let old_env = self.env.clone();
                self.env = local_env;

                for stmt in &func.body {
                    match self.eval_statement(stmt)? {
                        ExecutionResult::Return(v) => {
                            self.env = old_env;
                            return Ok(v);
                        }
                        _ => {}
                    }
                }

                self.env = old_env;
                Ok(Value::None)
            }
            Value::BuiltinFn(name) => self.call_builtin(&name, args),
            _ => Err(format!("Cannot call {:?}", callee)),
        }
    }

    fn call_builtin(&mut self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        crate::builtins::call_builtin(name, args)
    }

    /// メソッド呼び出し (obj.method(args))
    fn call_method(&mut self, obj: Value, method: &str, args: Vec<Value>) -> Result<Value, String> {
        match obj {
            // List メソッド
            Value::List(list) => match method {
                "append" => {
                    if args.len() != 1 {
                        return Err("append() takes exactly 1 argument".to_string());
                    }
                    list.borrow_mut().push(args[0].clone());
                    Ok(Value::None)
                }
                "pop" => {
                    let popped = list.borrow_mut().pop();
                    popped.ok_or_else(|| "pop from empty list".to_string())
                }
                "insert" => {
                    if args.len() != 2 {
                        return Err("insert() takes exactly 2 arguments".to_string());
                    }
                    if let Value::Int(idx) = &args[0] {
                        list.borrow_mut().insert(*idx as usize, args[1].clone());
                        Ok(Value::None)
                    } else {
                        Err("insert() first argument must be int".to_string())
                    }
                }
                "clear" => {
                    list.borrow_mut().clear();
                    Ok(Value::None)
                }
                "index" => {
                    if args.len() != 1 {
                        return Err("index() takes exactly 1 argument".to_string());
                    }
                    let list = list.borrow();
                    for (i, v) in list.iter().enumerate() {
                        if self.values_equal(&args[0], v) {
                            return Ok(Value::Int(i as i64));
                        }
                    }
                    Err("value not in list".to_string())
                }
                "count" => {
                    if args.len() != 1 {
                        return Err("count() takes exactly 1 argument".to_string());
                    }
                    let count = list.borrow().iter().filter(|v| self.values_equal(&args[0], v)).count();
                    Ok(Value::Int(count as i64))
                }
                "copy" => {
                    let copy = list.borrow().clone();
                    Ok(Value::List(Rc::new(RefCell::new(copy))))
                }
                _ => Err(format!("List has no method '{}'", method)),
            },

            // String メソッド
            Value::Str(s) => match method {
                "upper" => Ok(Value::Str(s.to_uppercase())),
                "lower" => Ok(Value::Str(s.to_lowercase())),
                "strip" => Ok(Value::Str(s.trim().to_string())),
                "split" => {
                    let sep = if let Some(Value::Str(sep)) = args.first() {
                        sep.as_str()
                    } else {
                        " "
                    };
                    let parts: Vec<Value> = s.split(sep).map(|p| Value::Str(p.to_string())).collect();
                    Ok(Value::List(Rc::new(RefCell::new(parts))))
                }
                "join" => {
                    if args.len() != 1 {
                        return Err("join() takes exactly 1 argument".to_string());
                    }
                    if let Value::List(list) = &args[0] {
                        let strings: Result<Vec<String>, String> = list.borrow().iter().map(|v| {
                            if let Value::Str(s) = v {
                                Ok(s.clone())
                            } else {
                                Err("join() requires list of strings".to_string())
                            }
                        }).collect();
                        Ok(Value::Str(strings?.join(&s)))
                    } else {
                        Err("join() requires a list argument".to_string())
                    }
                }
                "replace" => {
                    if args.len() != 2 {
                        return Err("replace() takes exactly 2 arguments".to_string());
                    }
                    if let (Value::Str(old), Value::Str(new)) = (&args[0], &args[1]) {
                        Ok(Value::Str(s.replace(old, new)))
                    } else {
                        Err("replace() requires string arguments".to_string())
                    }
                }
                "startswith" => {
                    if args.len() != 1 {
                        return Err("startswith() takes exactly 1 argument".to_string());
                    }
                    if let Value::Str(prefix) = &args[0] {
                        Ok(Value::Bool(s.starts_with(prefix)))
                    } else {
                        Err("startswith() requires string argument".to_string())
                    }
                }
                "endswith" => {
                    if args.len() != 1 {
                        return Err("endswith() takes exactly 1 argument".to_string());
                    }
                    if let Value::Str(suffix) = &args[0] {
                        Ok(Value::Bool(s.ends_with(suffix)))
                    } else {
                        Err("endswith() requires string argument".to_string())
                    }
                }
                "find" => {
                    if args.len() != 1 {
                        return Err("find() takes exactly 1 argument".to_string());
                    }
                    if let Value::Str(sub) = &args[0] {
                        Ok(Value::Int(s.find(sub).map(|i| i as i64).unwrap_or(-1)))
                    } else {
                        Err("find() requires string argument".to_string())
                    }
                }
                "contains" => {
                    if args.len() != 1 {
                        return Err("contains() takes exactly 1 argument".to_string());
                    }
                    if let Value::Str(sub) = &args[0] {
                        Ok(Value::Bool(s.contains(sub)))
                    } else {
                        Err("contains() requires string argument".to_string())
                    }
                }
                _ => Err(format!("Str has no method '{}'", method)),
            },

            // Dict メソッド
            Value::Dict(dict) => match method {
                "keys" => {
                    let keys: Vec<Value> = dict.borrow().keys().map(|k| Value::Str(k.clone())).collect();
                    Ok(Value::List(Rc::new(RefCell::new(keys))))
                }
                "values" => {
                    let values: Vec<Value> = dict.borrow().values().cloned().collect();
                    Ok(Value::List(Rc::new(RefCell::new(values))))
                }
                "items" => {
                    let items: Vec<Value> = dict.borrow().iter().map(|(k, v)| {
                        Value::List(Rc::new(RefCell::new(vec![Value::Str(k.clone()), v.clone()])))
                    }).collect();
                    Ok(Value::List(Rc::new(RefCell::new(items))))
                }
                "get" => {
                    if args.is_empty() || args.len() > 2 {
                        return Err("get() takes 1 or 2 arguments".to_string());
                    }
                    if let Value::Str(key) = &args[0] {
                        let default = args.get(1).cloned().unwrap_or(Value::None);
                        Ok(dict.borrow().get(key).cloned().unwrap_or(default))
                    } else {
                        Err("get() key must be string".to_string())
                    }
                }
                "pop" => {
                    if args.len() != 1 {
                        return Err("pop() takes exactly 1 argument".to_string());
                    }
                    if let Value::Str(key) = &args[0] {
                        dict.borrow_mut().remove(key).ok_or_else(|| format!("Key error: {}", key))
                    } else {
                        Err("pop() key must be string".to_string())
                    }
                }
                "clear" => {
                    dict.borrow_mut().clear();
                    Ok(Value::None)
                }
                "contains" => {
                    if args.len() != 1 {
                        return Err("contains() takes exactly 1 argument".to_string());
                    }
                    if let Value::Str(key) = &args[0] {
                        Ok(Value::Bool(dict.borrow().contains_key(key)))
                    } else {
                        Err("contains() key must be string".to_string())
                    }
                }
                _ => Err(format!("Dict has no method '{}'", method)),
            },

            _ => Err(format!("'{}' has no methods", obj.display())),
        }
    }
}

/// 実行制御結果
#[derive(Debug)]
enum ExecutionResult {
    Value(Value),
    Return(Value),
    Break,
    Continue,
}

//! Type Checker for n7tya-lang
//!
//! ASTを走査し、型の整合性を検証する

use crate::ast::*;
use miette::Result;
use std::collections::HashMap;

/// 型表現（ASTのTypeとは別に、推論結果を表す）
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    Int,
    Float,
    Bool,
    Str,
    None,
    List(Box<TypeInfo>),
    Fn {
        params: Vec<TypeInfo>,
        ret: Box<TypeInfo>,
    },
    Class(String),
    Unknown, // 型推論が未確定
    Error,   // 型エラー
}

/// 型環境（スコープごとの変数・関数の型情報）
#[derive(Debug, Clone)]
pub struct TypeEnv {
    scopes: Vec<HashMap<String, TypeInfo>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut global = HashMap::new();

        // 汎用関数型 (任意の型を受け付ける)
        let any_fn = TypeInfo::Fn {
            params: vec![TypeInfo::Unknown],
            ret: Box::new(TypeInfo::Unknown),
        };
        let any_to_int = TypeInfo::Fn {
            params: vec![TypeInfo::Unknown],
            ret: Box::new(TypeInfo::Int),
        };
        let any_to_str = TypeInfo::Fn {
            params: vec![TypeInfo::Unknown],
            ret: Box::new(TypeInfo::Str),
        };
        let any_to_list = TypeInfo::Fn {
            params: vec![TypeInfo::Unknown],
            ret: Box::new(TypeInfo::List(Box::new(TypeInfo::Unknown))),
        };
        let any_to_bool = TypeInfo::Fn {
            params: vec![TypeInfo::Unknown],
            ret: Box::new(TypeInfo::Bool),
        };
        let any_to_float = TypeInfo::Fn {
            params: vec![TypeInfo::Unknown],
            ret: Box::new(TypeInfo::Float),
        };

        // 入出力
        global.insert("print".to_string(), any_fn.clone());
        global.insert("println".to_string(), any_fn.clone());
        global.insert("input".to_string(), any_to_str.clone());

        // コレクション
        global.insert("len".to_string(), any_to_int.clone());
        global.insert("range".to_string(), any_to_list.clone());
        global.insert("sum".to_string(), any_to_int.clone());
        global.insert("sorted".to_string(), any_to_list.clone());
        global.insert("reversed".to_string(), any_to_list.clone());
        global.insert("enumerate".to_string(), any_to_list.clone());
        global.insert("zip".to_string(), any_to_list.clone());

        // 型変換
        global.insert("str".to_string(), any_to_str.clone());
        global.insert("int".to_string(), any_to_int.clone());
        global.insert("float".to_string(), any_to_float.clone());
        global.insert("type".to_string(), any_to_str.clone());
        global.insert("bool".to_string(), any_to_bool.clone());

        // 数値
        global.insert("abs".to_string(), any_to_int.clone());
        global.insert("min".to_string(), any_to_int.clone());
        global.insert("max".to_string(), any_to_int.clone());

        // fs モジュール
        global.insert("fs.read_file".to_string(), any_to_str.clone());
        global.insert("fs.write_file".to_string(), any_fn.clone());
        global.insert("fs.exists".to_string(), any_to_bool.clone());
        global.insert("fs.remove".to_string(), any_fn.clone());
        global.insert("fs.read_dir".to_string(), any_to_list.clone());

        // json モジュール
        global.insert("json.parse".to_string(), TypeInfo::Fn {
            params: vec![TypeInfo::Str],
            ret: Box::new(TypeInfo::Unknown),
        });
        global.insert("json.stringify".to_string(), any_to_str.clone());

        // http モジュール
        global.insert("http.get".to_string(), any_to_str.clone());
        global.insert("http.post".to_string(), any_to_str.clone());

        // base64 モジュール
        global.insert("base64.encode".to_string(), any_to_str.clone());
        global.insert("base64.decode".to_string(), any_to_str.clone());

        // sqlite モジュール
        global.insert("sqlite.open".to_string(), any_to_int.clone());
        global.insert("sqlite.execute".to_string(), any_to_int.clone());
        global.insert("sqlite.query".to_string(), any_fn.clone()); // List<Dict>だが動的なのでUnknownにする
        global.insert("sqlite.close".to_string(), any_fn.clone());

        Self {
            scopes: vec![global],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: &str, ty: TypeInfo) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<TypeInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }
}

/// 型チェッカー
pub struct TypeChecker {
    env: TypeEnv,
    errors: Vec<String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new(),
            errors: Vec::new(),
        }
    }

    pub fn check(&mut self, program: &Program) -> Result<Vec<String>> {
        for item in &program.items {
            self.check_item(item);
        }
        Ok(self.errors.clone())
    }

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::FunctionDef(f) => self.check_function_def(f),
            Item::ClassDef(c) => self.check_class_def(c),
            Item::ComponentDef(c) => self.check_component_def(c),
            Item::ServerDef(s) => self.check_server_def(s),
            Item::Import(_) => {} // importは型チェック不要
            Item::Statement(s) => {
                self.check_statement(s);
            }
        }
    }

    fn check_function_def(&mut self, f: &FunctionDef) {
        // 関数の型を環境に登録
        let param_types: Vec<TypeInfo> = f
            .params
            .iter()
            .map(|p| self.ast_type_to_type_info(p.type_annotation.as_ref()))
            .collect();
        let ret_type = self.ast_type_to_type_info(f.return_type.as_ref());

        self.env.define(
            &f.name,
            TypeInfo::Fn {
                params: param_types.clone(),
                ret: Box::new(ret_type.clone()),
            },
        );

        // 関数本体のチェック
        self.env.push_scope();

        // パラメータを環境に追加
        for (param, ty) in f.params.iter().zip(param_types.iter()) {
            self.env.define(&param.name, ty.clone());
        }

        for stmt in &f.body {
            self.check_statement(stmt);
        }

        self.env.pop_scope();
    }

    fn check_class_def(&mut self, c: &ClassDef) {
        self.env.define(&c.name, TypeInfo::Class(c.name.clone()));

        self.env.push_scope();
        self.env.define("self", TypeInfo::Class(c.name.clone()));

        for item in &c.body {
            match item {
                ClassBodyItem::Field(f) => {
                    let ty = self.ast_type_to_type_info(Some(&f.type_annotation));
                    self.env.define(&f.name, ty);
                }
                ClassBodyItem::Method(m) => {
                    self.check_function_def(m);
                }
            }
        }

        self.env.pop_scope();
    }

    fn check_component_def(&mut self, c: &ComponentDef) {
        self.env.define(&c.name, TypeInfo::Class(c.name.clone()));

        self.env.push_scope();
        self.env.define("self", TypeInfo::Class(c.name.clone()));

        for item in &c.body {
            match item {
                ComponentBodyItem::State(s) => {
                    let ty = self.infer_expression(&s.value);
                    self.env.define(&s.name, ty);
                }
                ComponentBodyItem::Method(m) => {
                    self.check_function_def(m);
                }
                ComponentBodyItem::Render(r) => {
                    for stmt in &r.body {
                        self.check_statement(stmt);
                    }
                }
            }
        }

        self.env.pop_scope();
    }

    fn check_server_def(&mut self, s: &ServerDef) {
        self.env.define(&s.name, TypeInfo::Class(s.name.clone()));

        self.env.push_scope();

        for item in &s.body {
            match item {
                ServerBodyItem::Route(r) => {
                    for stmt in &r.body {
                        self.check_statement(stmt);
                    }
                }
            }
        }

        self.env.pop_scope();
    }

    fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let(decl) => {
                let ty = self.infer_expression(&decl.value);
                self.env.define(&decl.name, ty);
            }
            Statement::Const(decl) => {
                let ty = self.infer_expression(&decl.value);
                self.env.define(&decl.name, ty);
            }
            Statement::Assignment(a) => {
                let target_ty = self.infer_expression(&a.target);
                let value_ty = self.infer_expression(&a.value);
                if !self.types_compatible(&target_ty, &value_ty) {
                    self.errors.push(format!(
                        "Type mismatch in assignment: expected {:?}, got {:?}",
                        target_ty, value_ty
                    ));
                }
            }
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    let _ = self.infer_expression(e);
                }
            }
            Statement::If(if_stmt) => {
                let cond_ty = self.infer_expression(&if_stmt.condition);
                if cond_ty != TypeInfo::Bool && cond_ty != TypeInfo::Unknown {
                    self.errors
                        .push(format!("If condition must be Bool, got {:?}", cond_ty));
                }
                self.env.push_scope();
                for s in &if_stmt.then_block {
                    self.check_statement(s);
                }
                self.env.pop_scope();
                if let Some(else_block) = &if_stmt.else_block {
                    self.env.push_scope();
                    for s in else_block {
                        self.check_statement(s);
                    }
                    self.env.pop_scope();
                }
            }
            Statement::While(w) => {
                let cond_ty = self.infer_expression(&w.condition);
                if cond_ty != TypeInfo::Bool && cond_ty != TypeInfo::Unknown {
                    self.errors
                        .push(format!("While condition must be Bool, got {:?}", cond_ty));
                }
                self.env.push_scope();
                for s in &w.body {
                    self.check_statement(s);
                }
                self.env.pop_scope();
            }
            Statement::For(f) => {
                let iter_ty = self.infer_expression(&f.iterator);
                let elem_ty = match iter_ty {
                    TypeInfo::List(inner) => *inner,
                    _ => TypeInfo::Unknown,
                };
                self.env.push_scope();
                self.env.define(&f.target, elem_ty);
                for s in &f.body {
                    self.check_statement(s);
                }
                self.env.pop_scope();
            }
            Statement::Match(m) => {
                let _ = self.infer_expression(&m.value);
                for case in &m.cases {
                    self.env.push_scope();
                    for s in &case.body {
                        self.check_statement(s);
                    }
                    self.env.pop_scope();
                }
            }
            Statement::Break | Statement::Continue => {}
            Statement::Expression(e) => {
                let _ = self.infer_expression(e);
            }
            Statement::State(s) => {
                let ty = self.infer_expression(&s.value);
                self.env.define(&s.name, ty);
            }
            Statement::Render(r) => {
                for s in &r.body {
                    self.check_statement(s);
                }
            }
        }
    }

    fn infer_expression(&mut self, expr: &Expression) -> TypeInfo {
        match expr {
            Expression::Literal(lit) => self.infer_literal(lit),
            Expression::Identifier(name) => self.env.lookup(name).unwrap_or_else(|| {
                self.errors.push(format!("Undefined variable: {}", name));
                TypeInfo::Error
            }),
            Expression::BinaryOp(bin) => {
                let left = self.infer_expression(&bin.left);
                let right = self.infer_expression(&bin.right);
                self.infer_binary_op(&bin.op, &left, &right)
            }
            Expression::UnaryOp(unary) => {
                let operand = self.infer_expression(&unary.operand);
                match unary.op {
                    UnaryOp::Neg => operand,
                    UnaryOp::Not => TypeInfo::Bool,
                }
            }
            Expression::Call(call) => {
                // モジュール関数チェック (fs.read_file など)
                if let Expression::MemberAccess(m) = &call.func {
                    if let Expression::Identifier(module_name) = &m.object {
                        let full_name = format!("{}.{}", module_name, m.member);
                        if let Some(ty) = self.env.lookup(&full_name) {
                            return match ty {
                                TypeInfo::Fn { ret, .. } => *ret,
                                _ => TypeInfo::Unknown,
                            };
                        }
                    }
                }
                
                let func_ty = self.infer_expression(&call.func);
                match func_ty {
                    TypeInfo::Fn { ret, .. } => *ret,
                    TypeInfo::Class(name) => TypeInfo::Class(name),
                    TypeInfo::Unknown => TypeInfo::Unknown,
                    _ => {
                        self.errors
                            .push(format!("Attempt to call non-function: {:?}", func_ty));
                        TypeInfo::Error
                    }
                }
            }
            Expression::MemberAccess(m) => {
                let _ = self.infer_expression(&m.object);
                TypeInfo::Unknown
            }
            Expression::Index(idx) => {
                let obj_ty = self.infer_expression(&idx.object);
                let _ = self.infer_expression(&idx.index);
                match obj_ty {
                    TypeInfo::List(inner) => *inner,
                    _ => TypeInfo::Unknown,
                }
            }
            Expression::Lambda(_) => TypeInfo::Unknown,
            Expression::Await(inner) => self.infer_expression(inner),
            Expression::JsxElement(_) => TypeInfo::Unknown,
        }
    }

    fn infer_literal(&self, lit: &Literal) -> TypeInfo {
        match lit {
            Literal::Int(_) => TypeInfo::Int,
            Literal::Float(_) => TypeInfo::Float,
            Literal::Str(_) => TypeInfo::Str,
            Literal::Bool(_) => TypeInfo::Bool,
            Literal::None => TypeInfo::None,
            Literal::List(_) => TypeInfo::List(Box::new(TypeInfo::Unknown)),
            Literal::Dict(_) => TypeInfo::Unknown,
            Literal::Set(_) => TypeInfo::Unknown,
        }
    }

    fn infer_binary_op(&mut self, op: &BinaryOp, left: &TypeInfo, right: &TypeInfo) -> TypeInfo {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if *left == TypeInfo::Str && *right == TypeInfo::Str && matches!(op, BinaryOp::Add)
                {
                    return TypeInfo::Str;
                }
                if (*left == TypeInfo::Int || *left == TypeInfo::Unknown)
                    && (*right == TypeInfo::Int || *right == TypeInfo::Unknown)
                {
                    return TypeInfo::Int;
                }
                if *left == TypeInfo::Float || *right == TypeInfo::Float {
                    return TypeInfo::Float;
                }
                TypeInfo::Unknown
            }
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Gt
            | BinaryOp::Le
            | BinaryOp::Ge
            | BinaryOp::In => TypeInfo::Bool,
            BinaryOp::And | BinaryOp::Or => TypeInfo::Bool,
        }
    }

    fn types_compatible(&self, expected: &TypeInfo, actual: &TypeInfo) -> bool {
        if *expected == TypeInfo::Unknown || *actual == TypeInfo::Unknown {
            return true;
        }
        expected == actual
    }

    fn ast_type_to_type_info(&self, ty: Option<&Type>) -> TypeInfo {
        match ty {
            Some(Type::Int) => TypeInfo::Int,
            Some(Type::Float) => TypeInfo::Float,
            Some(Type::Bool) => TypeInfo::Bool,
            Some(Type::Str) => TypeInfo::Str,
            Some(Type::List(inner)) => {
                TypeInfo::List(Box::new(self.ast_type_to_type_info(Some(inner))))
            }
            Some(Type::Dict(_, _)) => TypeInfo::Unknown,
            Some(Type::Set(_)) => TypeInfo::Unknown,
            Some(Type::Fn(_, _)) => TypeInfo::Unknown,
            Some(Type::Custom(name)) => TypeInfo::Class(name.clone()),
            None => TypeInfo::Unknown,
        }
    }
}

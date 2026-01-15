//! Parser implementation

use crate::ast::*;
use crate::lexer::{Token, TokenInfo};
use miette::Result;

pub struct Parser {
    tokens: Vec<TokenInfo>,
    current: usize,
    indent_level: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenInfo>) -> Self {
        Self {
            tokens,
            current: 0,
            indent_level: 0,
        }
    }

    /// プログラム全体をパース
    pub fn parse(&mut self) -> Result<Program> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            // トップレベルの改行はスキップ
            if self.match_token(Token::Newline) {
                continue;
            }

            if let Some(item) = self.parse_item()? {
                items.push(item);
            } else {
                // パースできないトークンがあった場合、とりあえずエラーにするかスキップするか
                // ここでは1つ進めてみる（簡易的なエラー回復）
                self.advance();
            }
        }

        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Option<Item>> {
        if self.match_token(Token::Def) {
            return Ok(Some(Item::FunctionDef(self.parse_function_def()?)));
        }

        if self.match_token(Token::Class) {
            return Ok(Some(Item::ClassDef(self.parse_class_def()?)));
        }

        if self.match_token(Token::Component) {
            return Ok(Some(Item::ComponentDef(self.parse_component_def()?)));
        }

        if self.match_token(Token::Server) {
            return Ok(Some(Item::ServerDef(self.parse_server_def()?)));
        }

        // Import文
        if self.match_token(Token::Import) {
            return Ok(Some(Item::Import(self.parse_import()?)));
        }
        if self.match_token(Token::From) {
            return Ok(Some(Item::Import(self.parse_from_import()?)));
        }

        // 文としてパースを試みる
        if let Some(stmt) = self.parse_statement()? {
            return Ok(Some(Item::Statement(stmt)));
        }

        Ok(None)
    }

    fn parse_class_def(&mut self) -> Result<ClassDef> {
        let name = self.consume_identifier("Expect class name")?;

        // 継承チェック: class Child Parent
        let parent = if let Some(Token::Identifier(parent_name)) = self.peek_token().cloned() {
            if !matches!(parent_name.as_str(), "Newline") {
                self.advance();
                Some(parent_name)
            } else {
                None
            }
        } else {
            None
        };

        self.consume(Token::Newline, "Expect newline after class name")?;

        let body = self.parse_indented_block(|parser| {
            if parser.match_token(Token::Def) {
                let func = parser.parse_function_def()?;
                return Ok(Some(ClassBodyItem::Method(func)));
            }
            if let Some(Token::Identifier(id)) = parser.peek_token().cloned() {
                parser.advance();
                if parser.match_token(Token::Colon) {
                    let type_annotation = parser.parse_type_annotation()?;
                    parser.consume(Token::Newline, "Expect newline after field definition")?;
                    return Ok(Some(ClassBodyItem::Field(FieldDef {
                        name: id,
                        type_annotation,
                    })));
                } else {
                    return Err(miette::miette!("Expect ':' for field definition"));
                }
            }
            Ok(None)
        })?;

        Ok(ClassDef { name, parent, body })
    }

    fn parse_server_def(&mut self) -> Result<ServerDef> {
        let name = self.consume_identifier("Expect server name")?;
        self.consume(Token::Newline, "Expect newline after server name")?;

        let body = self.parse_indented_block(|parser| {
            // メソッド名を取得（Identifier または Route キーワード）
            let method = if let Some(Token::Identifier(s)) = parser.peek_token().cloned() {
                parser.advance();
                s
            } else if parser.match_token(Token::Route) {
                "route".to_string()
            } else {
                return Ok(None);
            };

            let path_token = parser.peek_token().cloned();
            if let Some(Token::StringLiteral(path) | Token::MultiLineString(path)) = path_token {
                parser.advance(); // consume path
                parser.consume(Token::Newline, "Expect newline after route path")?;
                let body = parser.parse_block()?;
                return Ok(Some(ServerBodyItem::Route(RouteDef { path, method, body })));
            } else {
                return Err(miette::miette!(
                    "Expect string literal (path) after route method, got {:?}",
                    parser.peek_token()
                ));
            }
        })?;

        Ok(ServerDef { name, body })
    }

    fn parse_component_def(&mut self) -> Result<ComponentDef> {
        let name = self.consume_identifier("Expect component name")?;
        self.consume(Token::Newline, "Expect newline after component name")?;

        let body = self.parse_indented_block(|parser| {
            if parser.match_token(Token::State) {
                let state = parser.parse_state_decl()?;
                return Ok(Some(ComponentBodyItem::State(state)));
            }
            if parser.match_token(Token::Def) {
                let func = parser.parse_function_def()?;
                return Ok(Some(ComponentBodyItem::Method(func)));
            }
            if parser.match_token(Token::Render) {
                let render = parser.parse_render_block()?;
                return Ok(Some(ComponentBodyItem::Render(render)));
            }
            // 空行やコメントは parse_indented_block でスキップされるが、
            // 未知のトークンの場合は None を返して終了させる
            Ok(None)
        })?;

        Ok(ComponentDef { name, body })
    }

    fn parse_function_def(&mut self) -> Result<FunctionDef> {
        // "def" は既に消費済み
        let name = self.consume_identifier("Expect function name")?;

        // パラメータ
        let mut params = Vec::new();
        // TODO: パラメータのパース (a: Type, b: Type)
        // コマンドスタイル定義: def name arg1, arg2
        // 括弧スタイル: def name(arg1: Type, arg2: Type) -> RetType
        // n7tya specs: def add a: Int, b: Int -> Int (parentheses optional/omitted in sample)

        while !self.check(Token::Newline)
            && !self.check(Token::Arrow)
            && !self.check(Token::Colon)
            && !self.is_at_end()
        {
            if let Ok(param_name) = self.consume_identifier("") {
                let mut type_annotation = None;
                if self.match_token(Token::Colon) {
                    type_annotation = Some(self.parse_type_annotation()?);
                }

                params.push(Param {
                    name: param_name,
                    type_annotation,
                });

                self.match_token(Token::Comma);
            } else {
                break;
            }
        }

        let mut return_type = None;
        if self.match_token(Token::Arrow) {
            return_type = Some(self.parse_type_annotation()?);
        }

        self.consume(Token::Newline, "Expect newline after function signature")?;

        // 関数本体
        let body = self.parse_block()?;

        Ok(FunctionDef {
            name,
            params,
            return_type,
            body,
            is_async: false, // TODO: async keyword check
        })
    }

    fn parse_type_annotation(&mut self) -> Result<Type> {
        let name = self.consume_identifier("Expect type name")?;

        if name == "List" {
            if self.match_token(Token::Lt) {
                let inner = self.parse_type_annotation()?;
                self.consume(Token::Gt, "Expect '>' after generic type")?;
                return Ok(Type::List(Box::new(inner)));
            } else {
                return Err(miette::miette!("Expect generic argument for List"));
            }
        }

        // generic args <T> (List以外は無視か、将来対応)
        if self.match_token(Token::Lt) {
            while !self.check(Token::Gt) && !self.is_at_end() {
                self.advance();
            }
            self.consume(Token::Gt, "Expect '>' after generic args")?;
        }

        match name.as_str() {
            "Int" => Ok(Type::Int),
            "Float" => Ok(Type::Float),
            "Bool" => Ok(Type::Bool),
            "Str" => Ok(Type::Str),
            _ => Ok(Type::Custom(name)),
        }
    }

    /// ブロック（インデントされた一連の文）をパース
    fn parse_block(&mut self) -> Result<Vec<Statement>> {
        self.parse_indented_block(|parser| parser.parse_statement())
    }

    /// 汎用的なインデントブロックパース
    fn parse_indented_block<T, F>(&mut self, mut parse_fn: F) -> Result<Vec<T>>
    where
        F: FnMut(&mut Self) -> Result<Option<T>>,
    {
        let mut items = Vec::new();

        // インデントが増えていることを確認
        self.indent_level += 1;

        while !self.is_at_end() {
            // 行頭のインデントチェック
            let current_indent = self.count_indent();

            if current_indent < self.indent_level {
                // インデントが戻ったらブロック終了
                break;
            }

            // インデントトークンを消費
            // Note: current_indent > indent_level の場合は、
            // その行のインデントを全て消費してから parse_fn に委ねる。
            // ネストしたブロックは再帰的に parse_indented_block が呼ばれることで処理される。
            for _ in 0..current_indent {
                if self.check(Token::Tab) {
                    self.advance();
                }
            }

            // 空行はスキップ
            if self.match_token(Token::Newline) {
                continue;
            }

            if let Some(item) = parse_fn(self)? {
                items.push(item);
            } else {
                // 認識できないアイテムがあった場合、ブロック終了とみなす
                // エラー回復を入れるならここでスキップ処理が必要
                break;
            }
        }

        self.indent_level -= 1;
        Ok(items)
    }

    /// 現在の行のインデント（タブ数）をカウント（消費はしない）
    fn count_indent(&self) -> usize {
        let mut count = 0;
        let mut i = self.current;
        while i < self.tokens.len() {
            if matches!(self.tokens[i].token, Token::Tab) {
                count += 1;
                i += 1;
            } else {
                break;
            }
        }
        count
    }

    fn parse_statement(&mut self) -> Result<Option<Statement>> {
        if self.match_token(Token::Let) {
            return Ok(Some(Statement::Let(self.parse_let()?)));
        }
        if self.match_token(Token::Const) {
            return Ok(Some(Statement::Const(self.parse_const()?)));
        }
        if self.match_token(Token::State) {
            return Ok(Some(Statement::State(self.parse_state_decl()?)));
        }
        if self.match_token(Token::Render) {
            return Ok(Some(Statement::Render(self.parse_render_block()?)));
        }
        if self.match_token(Token::Return) {
            let expr = if !self.check(Token::Newline) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.consume(Token::Newline, "Expect newline after return")?;
            return Ok(Some(Statement::Return(expr)));
        }
        if self.match_token(Token::Break) {
            self.match_token(Token::Newline);
            return Ok(Some(Statement::Break));
        }
        if self.match_token(Token::Continue) {
            self.match_token(Token::Newline);
            return Ok(Some(Statement::Continue));
        }
        if self.match_token(Token::If) {
            return Ok(Some(Statement::If(self.parse_if()?)));
        }
        if self.match_token(Token::While) {
            return Ok(Some(Statement::While(self.parse_while()?)));
        }
        if self.match_token(Token::For) {
            return Ok(Some(Statement::For(self.parse_for()?)));
        }
        if self.match_token(Token::Match) {
            return Ok(Some(Statement::Match(self.parse_match()?)));
        }

        // 式文 or 代入
        if let Ok(expr) = self.parse_expression() {
            if self.match_token(Token::Assign) {
                let value = self.parse_expression()?;
                self.match_token(Token::Newline);
                return Ok(Some(Statement::Assignment(AssignmentStmt {
                    target: expr,
                    value,
                })));
            }

            self.match_token(Token::Newline);
            return Ok(Some(Statement::Expression(expr)));
        }

        Ok(None)
    }

    fn parse_let(&mut self) -> Result<LetDecl> {
        let name = self.consume_identifier("Expect variable name")?;
        let type_annotation = if self.match_token(Token::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        self.consume(Token::Assign, "Expect '=' after variable name")?;
        let value = self.parse_expression()?;
        self.match_token(Token::Newline);
        Ok(LetDecl {
            name,
            value,
            type_annotation,
        })
    }

    fn parse_const(&mut self) -> Result<ConstDecl> {
        let name = self.consume_identifier("Expect constant name")?;
        let type_annotation = if self.match_token(Token::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        self.consume(Token::Assign, "Expect '=' after constant name")?;
        let value = self.parse_expression()?;
        self.match_token(Token::Newline);
        Ok(ConstDecl {
            name,
            value,
            type_annotation,
        })
    }

    fn parse_import(&mut self) -> Result<ImportStmt> {
        let module = self.consume_identifier("Expect module name")?;
        let alias = if self.match_token(Token::As) {
            Some(self.consume_identifier("Expect alias name")?)
        } else {
            None
        };
        self.match_token(Token::Newline);
        Ok(ImportStmt {
            module,
            names: vec![],
            alias,
        })
    }

    fn parse_from_import(&mut self) -> Result<ImportStmt> {
        let module = self.consume_identifier("Expect module name")?;
        self.consume(Token::Import, "Expect 'import' after module name")?;
        let mut names = Vec::new();
        loop {
            names.push(self.consume_identifier("Expect import name")?);
            if !self.match_token(Token::Comma) {
                break;
            }
        }
        self.match_token(Token::Newline);
        Ok(ImportStmt {
            module,
            names,
            alias: None,
        })
    }

    fn parse_match(&mut self) -> Result<MatchStmt> {
        let value = self.parse_expression()?;
        self.consume(Token::Newline, "Expect newline after match value")?;

        let cases = self.parse_indented_block(|parser| {
            if parser.match_token(Token::Case) {
                let pattern = parser.parse_pattern()?;
                parser.consume(Token::Newline, "Expect newline after case pattern")?;
                let body = parser.parse_block()?;
                return Ok(Some(MatchCase { pattern, body }));
            }
            Ok(None)
        })?;

        Ok(MatchStmt { value, cases })
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        if let Some(Token::IntLiteral(n)) = self.peek_token().cloned() {
            self.advance();
            return Ok(Pattern::Literal(Literal::Int(n)));
        }
        if let Some(Token::StringLiteral(s)) = self.peek_token().cloned() {
            self.advance();
            return Ok(Pattern::Literal(Literal::Str(s)));
        }
        if self.match_token(Token::True) {
            return Ok(Pattern::Literal(Literal::Bool(true)));
        }
        if self.match_token(Token::False) {
            return Ok(Pattern::Literal(Literal::Bool(false)));
        }
        // Wildcard _
        if let Some(Token::Identifier(name)) = self.peek_token().cloned() {
            self.advance();
            if name == "_" {
                return Ok(Pattern::Wildcard);
            }
            return Ok(Pattern::Identifier(name));
        }
        Err(miette::miette!("Invalid pattern"))
    }

    fn parse_state_decl(&mut self) -> Result<StateDecl> {
        let name = self.consume_identifier("Expect state name")?;
        self.consume(Token::Assign, "Expect '='")?;
        let value = self.parse_expression()?;
        self.match_token(Token::Newline);
        Ok(StateDecl { name, value })
    }

    fn parse_render_block(&mut self) -> Result<RenderBlock> {
        self.consume(Token::Newline, "Expect newline after render")?;
        let body = self.parse_block()?;
        Ok(RenderBlock { body })
    }

    fn parse_if(&mut self) -> Result<IfStmt> {
        let condition = self.parse_expression()?;
        self.consume(Token::Newline, "Expect newline after if condition")?;
        let then_block = self.parse_block()?;

        let mut else_block = None;
        if self.match_token(Token::Else) {
            self.consume(Token::Newline, "Expect newline after else")?;
            else_block = Some(self.parse_block()?);
        } else if self.match_token(Token::Elif) {
            // Elif は Else 内の If として扱う（糖衣構文）
            // Pythonのように `elif cond:` -> `else: if cond:`
            let elif_stmt = Statement::If(self.parse_if()?);
            else_block = Some(vec![elif_stmt]);
        }

        Ok(IfStmt {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_while(&mut self) -> Result<WhileStmt> {
        let condition = self.parse_expression()?;
        self.consume(Token::Newline, "Expect newline after while condition")?;
        let body = self.parse_block()?;
        Ok(WhileStmt { condition, body })
    }

    fn parse_for(&mut self) -> Result<ForStmt> {
        let target = self.consume_identifier("Expect for loop variable")?;
        self.consume(Token::In, "Expect 'in' after for loop variable")?;
        let iterator = self.parse_expression()?;
        self.consume(Token::Newline, "Expect newline after for loop header")?;
        let body = self.parse_block()?;
        Ok(ForStmt {
            target,
            iterator,
            body,
        })
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_logic_or()
    }

    fn parse_logic_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_logic_and()?;
        while self.match_token(Token::Or) {
            let right = self.parse_logic_and()?;
            expr = Expression::BinaryOp(Box::new(BinaryExpr {
                left: expr,
                op: BinaryOp::Or,
                right,
            }));
        }
        Ok(expr)
    }

    fn parse_logic_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality()?;
        while self.match_token(Token::And) {
            let right = self.parse_equality()?;
            expr = Expression::BinaryOp(Box::new(BinaryExpr {
                left: expr,
                op: BinaryOp::And,
                right,
            }));
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison()?;
        while self.match_token(Token::Eq) || self.match_token(Token::NotEq) {
            let op = match self.previous().token {
                Token::Eq => BinaryOp::Eq,
                Token::NotEq => BinaryOp::Ne,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            expr = Expression::BinaryOp(Box::new(BinaryExpr {
                left: expr,
                op,
                right,
            }));
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut expr = self.parse_term()?;
        while self.match_token(Token::Lt)
            || self.match_token(Token::Gt)
            || self.match_token(Token::LtEq)
            || self.match_token(Token::GtEq)
        {
            let op = match self.previous().token {
                Token::Lt => BinaryOp::Lt,
                Token::Gt => BinaryOp::Gt,
                Token::LtEq => BinaryOp::Le,
                Token::GtEq => BinaryOp::Ge,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            expr = Expression::BinaryOp(Box::new(BinaryExpr {
                left: expr,
                op,
                right,
            }));
        }
        Ok(expr)
    }

    /// 足し算・引き算
    fn parse_term(&mut self) -> Result<Expression> {
        let mut expr = self.parse_factor()?;

        while self.match_token(Token::Plus) || self.match_token(Token::Minus) {
            let op = match self.previous().token {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            expr = Expression::BinaryOp(Box::new(BinaryExpr {
                left: expr,
                op,
                right,
            }));
        }

        Ok(expr)
    }

    /// 掛け算・割り算・剰余
    fn parse_factor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary()?;

        while self.match_token(Token::Star)
            || self.match_token(Token::Slash)
            || self.match_token(Token::Percent)
        {
            let op = match self.previous().token {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            expr = Expression::BinaryOp(Box::new(BinaryExpr {
                left: expr,
                op,
                right,
            }));
        }

        Ok(expr)
    }

    /// 単項演算子 (-x, not x)
    fn parse_unary(&mut self) -> Result<Expression> {
        if self.match_token(Token::Minus) {
            let operand = self.parse_unary()?;
            return Ok(Expression::UnaryOp(Box::new(UnaryExpr {
                op: UnaryOp::Neg,
                operand,
            })));
        }
        if self.match_token(Token::Not) {
            let operand = self.parse_unary()?;
            return Ok(Expression::UnaryOp(Box::new(UnaryExpr {
                op: UnaryOp::Not,
                operand,
            })));
        }
        self.parse_call()
    }

    /// 関数呼び出し (func arg1, arg2)
    fn parse_call(&mut self) -> Result<Expression> {
        let func = self.parse_postfix()?; // term -> call -> postfix

        // 引数の開始判定
        if self.is_arg_start() {
            let mut args = Vec::new();
            loop {
                // Callの引数には優先度の高い式を渡す（LogicOr未満、つまりすべての式を許容したいが、
                // コマンドスタイルの場合、カンマ区切りと競合しないようにする必要がある）
                // parse_expression() を呼ぶと、 `f a, b` の `a` が `Expression`
                // ここで `parse_expression` を呼んで良い
                args.push(self.parse_expression()?);

                if self.match_token(Token::Comma) {
                    continue;
                } else {
                    break;
                }
            }
            return Ok(Expression::Call(Box::new(CallExpr { func, args })));
        }

        Ok(func)
    }

    fn is_arg_start(&self) -> bool {
        if let Some(token) = self.peek_token() {
            match token {
                Token::Identifier(_)
                | Token::IntLiteral(_)
                | Token::StringLiteral(_)
                | Token::MultiLineString(_)
                | Token::FloatLiteral(_)
                | Token::LParen
                | Token::LBrace
                | Token::LBracket
                | Token::SelfKw => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// メンバアクセス (obj.prop) と 関数呼び出し (obj())
    fn parse_postfix(&mut self) -> Result<Expression> {
        let mut expr = self.parse_atom()?;

        loop {
            if self.match_token(Token::Dot) {
                let member = self.consume_identifier("Expect member name")?;
                expr = Expression::MemberAccess(Box::new(MemberExpr {
                    object: expr,
                    member,
                }));
            } else if self.match_token(Token::LParen) {
                let mut args = Vec::new();
                if !self.check(Token::RParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if self.match_token(Token::Comma) {
                            continue;
                        } else {
                            break;
                        }
                    }
                }
                self.consume(Token::RParen, "Expect ')' after arguments")?;
                expr = Expression::Call(Box::new(CallExpr { func: expr, args }));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    /// 原子的な式 (Identifier, Literal, JSX, Paren)
    fn parse_atom(&mut self) -> Result<Expression> {
        if self.match_token(Token::SelfKw) {
            return Ok(Expression::Identifier("self".to_string())); // SelfKwをIdentifierとして扱うか、専用にするか。一旦Identifier。
        }

        // JSX Element
        if self.match_token(Token::Lt) {
            return Ok(Expression::JsxElement(Box::new(self.parse_jsx_element()?)));
        }

        // リストリテラル [1, 2, 3]
        if self.match_token(Token::LBracket) {
            let mut elements = Vec::new();
            if !self.check(Token::RBracket) {
                loop {
                    // 改行は許可（整形用）
                    while self.match_token(Token::Newline) {}

                    elements.push(self.parse_expression()?);

                    while self.match_token(Token::Newline) {}

                    if self.match_token(Token::Comma) {
                        continue;
                    } else {
                        break;
                    }
                }
            }
            // 末尾カンマ後の改行対応
            while self.match_token(Token::Newline) {}
            self.consume(Token::RBracket, "Expect ']' after list elements")?;
            return Ok(Expression::Literal(Literal::List(elements)));
        }

        // 括弧 (expression)
        if self.match_token(Token::LParen) {
            let expr = self.parse_expression()?;
            self.consume(Token::RParen, "Expect ')' after expression")?;
            return Ok(expr);
        }

        if let Ok(id) = self.consume_identifier("") {
            return Ok(Expression::Identifier(id));
        }

        // リテラル
        let token = self.peek_token().cloned();

        if let Some(token) = token {
            match token {
                Token::IntLiteral(n) => {
                    self.advance();
                    return Ok(Expression::Literal(Literal::Int(n)));
                }
                Token::FloatLiteral(f) => {
                    self.advance();
                    return Ok(Expression::Literal(Literal::Float(f)));
                }
                Token::StringLiteral(s) | Token::MultiLineString(s) => {
                    self.advance();
                    return Ok(Expression::Literal(Literal::Str(s)));
                }
                Token::True => {
                    self.advance();
                    return Ok(Expression::Literal(Literal::Bool(true)));
                }
                Token::False => {
                    self.advance();
                    return Ok(Expression::Literal(Literal::Bool(false)));
                }
                Token::None => {
                    self.advance();
                    return Ok(Expression::Literal(Literal::None));
                }
                _ => {}
            }
        }

        Err(miette::miette!(
            "Expect expression, got {:?}",
            self.peek_token()
        ))
    }

    fn parse_jsx_element(&mut self) -> Result<JsxElement> {
        let tag = self.consume_identifier("Expect tag name")?;

        let mut attributes = Vec::new();
        // 属性パース
        while !self.check(Token::Gt) && !self.check(Token::SelfClose) && !self.is_at_end() {
            if let Ok(name) = self.consume_identifier("") {
                let mut value = None;
                if self.match_token(Token::Assign) {
                    if let Some(token) = self.peek_token().cloned() {
                        match token {
                            Token::StringLiteral(s) => {
                                self.advance();
                                value = Some(Expression::Literal(Literal::Str(s)));
                            }
                            Token::LBrace => {
                                self.advance();
                                let expr = self.parse_expression()?;
                                self.match_token(Token::RBrace);
                                value = Some(expr);
                            }
                            _ => {
                                // エラーだが、とりあえず無視して値なしとするか、エラーにする
                                // ここではIdentifierなどは許可しない（React風）
                            }
                        }
                    }
                }
                attributes.push(JsxAttribute { name, value });
            } else if self.match_token(Token::LBrace) {
                // Spread attributes {...props} (Not supported in AST yet, skip)
                self.advance();
                while !self.check(Token::RBrace) && !self.is_at_end() {
                    self.advance();
                }
                self.match_token(Token::RBrace);
            } else {
                // 未知のトークンが進まなくなるのを防ぐ
                self.advance();
            }
        }

        if self.match_token(Token::SelfClose) {
            return Ok(JsxElement {
                tag,
                attributes,
                children: Vec::new(),
            });
        }

        self.consume(Token::Gt, "Expect '>'")?;

        let mut children = Vec::new();
        // 子要素パース
        while !self.check(Token::CloseTag) && !self.is_at_end() {
            if self.check(Token::Lt) {
                // 子要素の開始
                self.advance();
                let child = self.parse_jsx_element()?;
                children.push(JsxChild::Element(child));
            } else if self.match_token(Token::LBrace) {
                // {expression}
                let expr = self.parse_expression()?;
                self.match_token(Token::RBrace);
                children.push(JsxChild::Expression(expr));
            } else {
                // テキストノード（トークンを文字列化）
                // StringLiteralならそのまま、Identifierなら名前、それ以外はトークンの文字表現
                if let Some(token) = self.peek_token().cloned() {
                    match token {
                        Token::StringLiteral(s) => {
                            self.advance();
                            children.push(JsxChild::Text(s));
                        }
                        Token::Identifier(s) => {
                            self.advance();
                            children.push(JsxChild::Text(s));
                        }
                        Token::Tab | Token::Newline => {
                            // 空白と改行は無視（整形用）
                            self.advance();
                        }
                        _ => {
                            // 他のトークンもテキストとして扱う？
                            // 簡易実装としてIdentifierだけ扱う
                            self.advance();
                        }
                    }
                } else {
                    break;
                }
            }
        }

        self.consume(Token::CloseTag, "Expect '</'")?;
        let close_tag = self.consume_identifier("Expect close tag name")?;

        if tag != close_tag {
            return Err(miette::miette!(
                "Tag mismatch: <{}> ... </{}>",
                tag,
                close_tag
            ));
        }

        self.consume(Token::Gt, "Expect '>' after close tag")?;

        Ok(JsxElement {
            tag,
            attributes,
            children,
        })
    }

    // ===== ヘルパーメソッド =====

    fn advance(&mut self) -> &TokenInfo {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> &TokenInfo {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> &TokenInfo {
        &self.tokens[self.current]
    }

    fn peek_token(&self) -> Option<&Token> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.tokens[self.current].token)
        }
    }

    fn check(&self, token_type: Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        // discriminant check (型のみの一致確認)
        std::mem::discriminant(&self.peek().token) == std::mem::discriminant(&token_type)
    }

    fn match_token(&mut self, token_type: Token) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, token_type: Token, message: &str) -> Result<&TokenInfo> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            // TODO: 正しいエラー位置報告
            Err(miette::miette!("{}", message))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String> {
        if let Some(Token::Identifier(s)) = self.peek_token().cloned() {
            self.advance();
            Ok(s)
        } else {
            Err(miette::miette!("{}", message))
        }
    }
}

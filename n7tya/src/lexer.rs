#![allow(dead_code)]
//! n7tya-lang Lexer (字句解析器)
//!
//! インデントベースの構文をトークンに分解する

use logos::Logos;

/// トークンの種類
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ ]")] // 単一のスペースはスキップ（4スペース or タブはインデントとして認識）
pub enum Token {
    // ===== キーワード =====
    #[token("def")]
    Def,
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("const")]
    Const,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("elif")]
    Elif,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("return")]
    Return,
    #[token("import")]
    Import,
    #[token("from")]
    From,
    #[token("as")]
    As,
    #[token("class")]
    Class,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("match")]
    Match,
    #[token("case")]
    Case,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("pass")]
    Pass,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("yield")]
    Yield,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("none")]
    None,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("in")]
    In,
    #[token("is")]
    Is,
    #[token("component")]
    Component,
    #[token("server")]
    Server,
    #[token("route")]
    Route,
    #[token("test")]
    Test,
    #[token("assert")]
    Assert,
    #[token("self")]
    SelfKw,
    #[token("super")]
    Super,
    #[token("render")]
    Render,
    #[token("state")]
    State,
    #[token("props")]
    Props,

    // ===== リテラル =====
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    IntLiteral(i64),

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    FloatLiteral(f64),

    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    StringLiteral(String),

    // ===== 識別子 =====
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // ===== 演算子 =====
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("=")]
    Assign,
    #[token("==")]
    Eq,
    #[token("!=")]
    NotEq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("->")]
    Arrow,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("..")]
    DotDot,

    // ===== 括弧 =====
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    // ===== JSX =====
    // < と > は Lt/Gt を再利用
    #[token("/>")]
    SelfClose,
    #[token("</")]
    CloseTag,

    // ===== インデント・改行 =====
    #[regex(r"\t|    ")]
    Tab,
    #[token("\n")]
    Newline,

    // ===== コメント =====
    #[regex(r"#[^\n]*", logos::skip)]
    Comment,

    // ===== エラー =====
    Error,
}

/// 字句解析の結果
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub span: std::ops::Range<usize>,
    pub line: usize,
    pub column: usize,
}

/// Lexer構造体
pub struct Lexer<'a> {
    inner: logos::Lexer<'a, Token>,
    source: &'a str,
    line: usize,
    line_start: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: Token::lexer(source),
            source,
            line: 1,
            line_start: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<TokenInfo> {
        let mut tokens = Vec::new();

        while let Some(result) = self.inner.next() {
            let span = self.inner.span();
            let column = span.start - self.line_start + 1;

            let token = match result {
                Ok(t) => t,
                Err(_) => Token::Error,
            };

            // 改行時に行番号を更新
            if matches!(token, Token::Newline) {
                tokens.push(TokenInfo {
                    token,
                    span: span.clone(),
                    line: self.line,
                    column,
                });
                self.line += 1;
                self.line_start = span.end;
                continue;
            }

            tokens.push(TokenInfo {
                token,
                span,
                line: self.line,
                column,
            });
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let source = "let x = 42";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        assert!(matches!(tokens[0].token, Token::Let));
        assert!(matches!(&tokens[1].token, Token::Identifier(s) if s == "x"));
        assert!(matches!(tokens[2].token, Token::Assign));
        assert!(matches!(tokens[3].token, Token::IntLiteral(42)));
    }

    #[test]
    fn test_function_def() {
        let source = "def add a, b\n\treturn a + b";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        assert!(matches!(tokens[0].token, Token::Def));
        assert!(matches!(&tokens[1].token, Token::Identifier(s) if s == "add"));
    }

    #[test]
    fn test_string_literal() {
        let source = r#"let name = "hello""#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        assert!(matches!(&tokens[3].token, Token::StringLiteral(s) if s == "hello"));
    }
}

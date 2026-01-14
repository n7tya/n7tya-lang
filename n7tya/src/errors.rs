#![allow(dead_code)]
//! n7tya-lang Error Definitions
//!
//! mietteを活用した構造化エラー

use miette::{Diagnostic, SourceSpan};
use std::ops::Range;
use thiserror::Error;

/// コンパイルエラーの種類
#[derive(Error, Debug, Diagnostic)]
pub enum N7tyaError {
    #[error("Syntax error: {message}")]
    #[diagnostic(code(n7tya::syntax))]
    Syntax {
        message: String,
        #[label("here")]
        span: SourceSpan,
    },

    #[error("Type error: {message}")]
    #[diagnostic(code(n7tya::type_error))]
    Type {
        message: String,
        #[label("type mismatch here")]
        span: Option<SourceSpan>,
    },

    #[error("Runtime error: {message}")]
    #[diagnostic(code(n7tya::runtime))]
    Runtime { message: String },

    #[error("Undefined variable: {name}")]
    #[diagnostic(
        code(n7tya::undefined),
        help("Did you forget to declare '{name}' with 'let'?")
    )]
    UndefinedVariable {
        name: String,
        #[label("not found")]
        span: Option<SourceSpan>,
    },

    #[error("File error: {message}")]
    #[diagnostic(code(n7tya::io))]
    FileError { message: String },
}

impl N7tyaError {
    pub fn syntax(message: impl Into<String>, range: Range<usize>) -> Self {
        Self::Syntax {
            message: message.into(),
            span: range.into(),
        }
    }

    pub fn type_error(message: impl Into<String>) -> Self {
        Self::Type {
            message: message.into(),
            span: None,
        }
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime {
            message: message.into(),
        }
    }

    pub fn undefined(name: impl Into<String>) -> Self {
        Self::UndefinedVariable {
            name: name.into(),
            span: None,
        }
    }

    pub fn file_error(message: impl Into<String>) -> Self {
        Self::FileError {
            message: message.into(),
        }
    }
}

/// エラー収集用のReporter
pub struct ErrorReporter {
    errors: Vec<N7tyaError>,
    source: Option<String>,
    source_name: Option<String>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            source: None,
            source_name: None,
        }
    }

    pub fn with_source(mut self, name: &str, content: &str) -> Self {
        self.source_name = Some(name.to_string());
        self.source = Some(content.to_string());
        self
    }

    pub fn report(&mut self, error: N7tyaError) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// エラーを表示
    pub fn print_errors(&self) {
        for error in &self.errors {
            eprintln!("{}", error);
        }
    }

    /// エラーの要約を取得
    pub fn summary(&self) -> String {
        if self.errors.is_empty() {
            "No errors".to_string()
        } else {
            format!("{} error(s)", self.errors.len())
        }
    }

    /// 行番号付きでエラーを表示
    pub fn print_errors_with_context(&self) {
        if let Some(source) = &self.source {
            let lines: Vec<&str> = source.lines().collect();
            let filename = self.source_name.as_deref().unwrap_or("<input>");

            for error in &self.errors {
                match error {
                    N7tyaError::Syntax { message, span } => {
                        let (line, col) = offset_to_line_col(source, span.offset());
                        eprintln!("{}:{}:{}: error: {}", filename, line + 1, col + 1, message);
                        if line < lines.len() {
                            eprintln!("  {} | {}", line + 1, lines[line]);
                            eprintln!(
                                "  {} | {}^",
                                " ".repeat((line + 1).to_string().len()),
                                " ".repeat(col)
                            );
                        }
                    }
                    N7tyaError::Type { message, span } => {
                        if let Some(span) = span {
                            let (line, col) = offset_to_line_col(source, span.offset());
                            eprintln!(
                                "{}:{}:{}: type error: {}",
                                filename,
                                line + 1,
                                col + 1,
                                message
                            );
                        } else {
                            eprintln!("{}: type error: {}", filename, message);
                        }
                    }
                    N7tyaError::UndefinedVariable { name, span } => {
                        if let Some(span) = span {
                            let (line, col) = offset_to_line_col(source, span.offset());
                            eprintln!(
                                "{}:{}:{}: error: undefined variable '{}'",
                                filename,
                                line + 1,
                                col + 1,
                                name
                            );
                        } else {
                            eprintln!("{}: error: undefined variable '{}'", filename, name);
                        }
                    }
                    _ => {
                        eprintln!("{}", error);
                    }
                }
            }
        } else {
            self.print_errors();
        }
    }
}

/// バイトオフセットを行番号と列番号に変換
fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

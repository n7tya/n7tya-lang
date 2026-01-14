//! n7tya-lang コンパイラ
//!
//! フルスタックWebアプリを1言語で開発するためのプログラミング言語

mod ast;
mod builtins;
mod errors;
mod interpreter;
mod jsx_render;
mod lexer;
mod parser;
mod python;
mod typechecker;

use interpreter::Interpreter;
use lexer::Lexer;
use miette::{Diagnostic, NamedSource, SourceSpan};
use parser::Parser;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use typechecker::TypeChecker;

/// コンパイラエラー
#[derive(Error, Debug, Diagnostic)]
#[error("Compilation error")]
#[diagnostic(code(n7tya::compile_error))]
pub struct CompileError {
    #[source_code]
    src: NamedSource<String>,

    #[label("error occurred here")]
    span: SourceSpan,

    #[help]
    help: String,
}

fn main() -> miette::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("n7tya-lang v0.1.0");
        println!();
        println!("Usage:");
        println!("  n7tya <file.n7t>    Run a file");
        println!("  n7tya run           Run project");
        println!("  n7tya build         Build project");
        println!("  n7tya test          Run tests");
        println!("  n7tya new <name>    Create new project");
        println!("  n7tya fmt           Format code");
        println!("  n7tya check         Type check");
        println!("  n7tya --version     Show version");
        println!("  n7tya --update      Update n7tya");
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "run" => {
            run_project()?;
        }
        "build" => {
            build_project()?;
        }
        "test" => {
            run_tests()?;
        }
        "new" => {
            if args.len() < 3 {
                println!("Usage: n7tya new <project-name>");
                return Ok(());
            }
            create_project(&args[2])?;
        }
        "fmt" => {
            format_project()?;
        }
        "check" => {
            if args.len() < 3 {
                println!("Usage: n7tya check <file.n7t>");
                return Ok(());
            }
            check_file(&args[2])?;
        }
        file if file.ends_with(".n7t") => {
            run_file(file)?;
        }
        "--version" | "-v" => {
            println!("n7tya-lang v0.1.0");
        }
        "--update" => {
            println!("To update n7tya-lang, please run:");
            println!("  cd <n7tya-repo-dir>");
            println!("  git pull origin main");
            println!("  cargo build --release");
        }
        _ => {
            println!("Unknown command: {}", command);
        }
    }

    Ok(())
}

/// ファイルを実行
fn run_file(path: &str) -> miette::Result<()> {
    let source = fs::read_to_string(path)
        .map_err(|e| miette::miette!("Failed to read file '{}': {}", path, e))?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(program) => {
            // 型チェック
            let mut checker = TypeChecker::new();
            match checker.check(&program) {
                Ok(errors) => {
                    if !errors.is_empty() {
                        println!("Type errors:");
                        for err in &errors {
                            println!("  - {}", err);
                        }
                        return Ok(());
                    }
                }
                Err(e) => {
                    println!("Type check failed: {:?}", e);
                    return Ok(());
                }
            }

            // 実行
            let mut interpreter = Interpreter::new();
            match interpreter.run(&program) {
                Ok(_result) => {
                    // 結果は print で出力されているので追加表示は不要
                }
                Err(e) => {
                    println!("Runtime error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Parse error: {:?}", e);
        }
    }

    Ok(())
}

/// 型チェックのみ実行
fn check_file(path: &str) -> miette::Result<()> {
    let source = fs::read_to_string(path)
        .map_err(|e| miette::miette!("Failed to read file '{}': {}", path, e))?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(program) => {
            let mut checker = TypeChecker::new();
            match checker.check(&program) {
                Ok(errors) => {
                    if errors.is_empty() {
                        println!("✓ No type errors in {}", path);
                    } else {
                        println!("✗ {} type error(s) in {}", errors.len(), path);
                        for err in &errors {
                            println!("  - {}", err);
                        }
                    }
                }
                Err(e) => {
                    println!("Type check failed: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Parse error: {:?}", e);
        }
    }

    Ok(())
}

/// プロジェクトを実行
fn run_project() -> miette::Result<()> {
    // n7tya.toml を探す
    if !PathBuf::from("n7tya.toml").exists() {
        return Err(miette::miette!(
            "No n7tya.toml found. Are you in a n7tya project directory?"
        ));
    }

    // src/main.n7t を実行
    let main_file = "src/main.n7t";
    if PathBuf::from(main_file).exists() {
        run_file(main_file)?;
    } else {
        return Err(miette::miette!("No src/main.n7t found"));
    }

    Ok(())
}

/// 新規プロジェクト作成
fn create_project(name: &str) -> miette::Result<()> {
    let project_dir = PathBuf::from(name);

    if project_dir.exists() {
        return Err(miette::miette!("Directory '{}' already exists", name));
    }

    // ディレクトリ作成
    fs::create_dir_all(project_dir.join("src"))
        .map_err(|e| miette::miette!("Failed to create directory: {}", e))?;

    // n7tya.toml
    let toml_content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"

[dependencies]

[python]
packages = []

[server]
port = 8080
"#,
        name
    );
    fs::write(project_dir.join("n7tya.toml"), toml_content)
        .map_err(|e| miette::miette!("Failed to write n7tya.toml: {}", e))?;

    // src/main.n7t
    let main_content = r#"# n7tya-lang main file

def main
	print "Hello, n7tya!"

main
"#;
    fs::write(project_dir.join("src/main.n7t"), main_content)
        .map_err(|e| miette::miette!("Failed to write main.n7t: {}", e))?;

    println!("Created project '{}'", name);
    println!();
    println!("  cd {}", name);
    println!("  n7tya run");

    Ok(())
}

/// プロジェクトをビルド
fn build_project() -> miette::Result<()> {
    println!("Building project...");

    if !PathBuf::from("n7tya.toml").exists() {
        return Err(miette::miette!(
            "No n7tya.toml found. Are you in a n7tya project directory?"
        ));
    }

    // srcディレクトリの全.n7tファイルを型チェック
    let src_dir = PathBuf::from("src");
    if !src_dir.exists() {
        return Err(miette::miette!("No src directory found"));
    }

    let mut error_count = 0;
    for entry in fs::read_dir(&src_dir).map_err(|e| miette::miette!("Failed to read src: {}", e))? {
        let entry = entry.map_err(|e| miette::miette!("Failed to read entry: {}", e))?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "n7t") {
            println!("  Checking {}...", path.display());

            let source = fs::read_to_string(&path)
                .map_err(|e| miette::miette!("Failed to read file: {}", e))?;

            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize();
            let mut parser = Parser::new(tokens);

            match parser.parse() {
                Ok(program) => {
                    let mut checker = TypeChecker::new();
                    if let Ok(errors) = checker.check(&program) {
                        if !errors.is_empty() {
                            error_count += errors.len();
                            for err in &errors {
                                println!("    Error: {}", err);
                            }
                        }
                    }
                }
                Err(e) => {
                    error_count += 1;
                    println!("    Parse error: {:?}", e);
                }
            }
        }
    }

    if error_count == 0 {
        println!("✓ Build successful!");
    } else {
        println!("✗ Build failed with {} error(s)", error_count);
    }

    Ok(())
}

/// テストを実行
fn run_tests() -> miette::Result<()> {
    println!("Running tests...");

    // testsディレクトリまたはtest_で始まるファイルを探す
    let test_dirs = vec![PathBuf::from("tests"), PathBuf::from("src")];
    let mut test_count = 0;
    let mut passed = 0;
    let mut failed = 0;

    for dir in test_dirs {
        if !dir.exists() {
            continue;
        }

        for entry in fs::read_dir(&dir).map_err(|e| miette::miette!("Failed to read dir: {}", e))? {
            let entry = entry.map_err(|e| miette::miette!("Failed to read entry: {}", e))?;
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if path.extension().map_or(false, |e| e == "n7t") && name.starts_with("test_") {
                test_count += 1;
                println!("  Running {}...", name);

                let source = fs::read_to_string(&path)
                    .map_err(|e| miette::miette!("Failed to read test file: {}", e))?;

                let mut lexer = Lexer::new(&source);
                let tokens = lexer.tokenize();
                let mut parser = Parser::new(tokens);

                match parser.parse() {
                    Ok(program) => {
                        let mut interpreter = Interpreter::new();
                        match interpreter.run(&program) {
                            Ok(_) => {
                                passed += 1;
                                println!("    ✓ Passed");
                            }
                            Err(e) => {
                                failed += 1;
                                println!("    ✗ Failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        println!("    ✗ Parse error: {:?}", e);
                    }
                }
            }
        }
    }

    if test_count == 0 {
        println!("No tests found. Create files starting with 'test_' in src/ or tests/");
    } else {
        println!();
        println!("{} tests: {} passed, {} failed", test_count, passed, failed);
    }

    Ok(())
}

/// コードをフォーマット
fn format_project() -> miette::Result<()> {
    println!("Formatting code...");

    let src_dir = PathBuf::from("src");
    if !src_dir.exists() {
        // カレントディレクトリの.n7tファイルをフォーマット
        format_directory(&PathBuf::from("."))?;
    } else {
        format_directory(&src_dir)?;
    }

    println!("✓ Formatting complete!");
    Ok(())
}

fn format_directory(dir: &PathBuf) -> miette::Result<()> {
    for entry in fs::read_dir(dir).map_err(|e| miette::miette!("Failed to read dir: {}", e))? {
        let entry = entry.map_err(|e| miette::miette!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "n7t") {
            println!("  Formatting {}...", path.display());

            let source = fs::read_to_string(&path)
                .map_err(|e| miette::miette!("Failed to read file: {}", e))?;

            // シンプルなフォーマット: 末尾空白の削除、一貫したインデント
            let formatted: Vec<String> = source
                .lines()
                .map(|line| {
                    // 先頭のスペースをタブに変換（4スペース=1タブ）
                    let leading_spaces = line.len() - line.trim_start().len();
                    let tabs = leading_spaces / 4;
                    let content = line.trim();
                    if content.is_empty() {
                        String::new()
                    } else {
                        format!("{}{}", "\t".repeat(tabs), content)
                    }
                })
                .collect();

            let formatted_content = formatted.join("\n") + "\n";
            fs::write(&path, formatted_content)
                .map_err(|e| miette::miette!("Failed to write file: {}", e))?;
        }
    }
    Ok(())
}

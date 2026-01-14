#![allow(dead_code)]
#![allow(unused_variables)]
//! JSX Rendering for n7tya-lang
//!
//! JSX要素をHTML文字列に変換するレンダラ

use crate::ast::*;
use crate::interpreter::{Interpreter, Value};

/// JSX要素をHTMLに変換
pub fn render_jsx(element: &JsxElement, interpreter: &mut Interpreter) -> Result<String, String> {
    let mut html = String::new();

    // 開始タグ
    html.push_str(&format!("<{}", element.tag));

    // 属性
    for attr in &element.attributes {
        let value_str = if let Some(expr) = &attr.value {
            match eval_jsx_expression(expr, interpreter)? {
                Value::Str(s) => s,
                v => v.display(),
            }
        } else {
            "true".to_string() // Boolean attribute
        };
        html.push_str(&format!(" {}=\"{}\"", attr.name, escape_html(&value_str)));
    }

    // 子要素がない場合は自己閉じタグ
    if element.children.is_empty() {
        html.push_str(" />");
        return Ok(html);
    }

    html.push('>');

    // 子要素
    for child in &element.children {
        match child {
            JsxChild::Element(child_elem) => {
                html.push_str(&render_jsx(child_elem, interpreter)?);
            }
            JsxChild::Text(text) => {
                html.push_str(&escape_html(text));
            }
            JsxChild::Expression(expr) => {
                let value = eval_jsx_expression(expr, interpreter)?;
                html.push_str(&escape_html(&value.display()));
            }
        }
    }

    // 閉じタグ
    html.push_str(&format!("</{}>", element.tag));

    Ok(html)
}

/// JSX内の式を評価
fn eval_jsx_expression(expr: &Expression, interpreter: &mut Interpreter) -> Result<Value, String> {
    // Interpreterの eval_expression はprivateだが、公開メソッドやリフレクションは使えない
    // 解決策: Interpreterに `eval_jsx_expr_public` のようなメソッドを追加するか、
    // ここで部分的に評価するか。
    // しかし `interpreter` は `&mut Interpreter` なので、メソッドを呼べばOK。
    // ただし `eval_expression` は private なので、pubにするか、`eval_expr_public` を作る必要がある。
    // ここでは `eval_expression` が private である前提で、Interpreterに `pub fn eval_expr(&mut self, e: &Expression)` を追加したと仮定してそれを呼ぶべき。
    // 現状 `interpreter.rs` の `eval_expression` は private なので、pubに変更する修正が必要。
    
    // 一旦、修正済みの `interpreter.rs` で `pub` になっていることを期待して呼び出す、
    // または `interpreter` 自体に評価メソッドを追加する。
    // ここでは `interpreter.eval_expr_public` を呼ぶ形にする。
    
    // しかし Rustの可視性ルールでコンパイルエラーになるため、
    // interpreter.rs 側で `eval_expression` を `pub(crate)` にするのが正解。
    // 今回の変更で `eval_expression` 自体を pub(crate) に変更したいが、
    // replace_file_content で interpreter.rs を修正済みかどうか確認が必要。
    // 修正していないので、まず interpreter.rs の `eval_expression` を修正する。
    
    // 仮実装: まだ呼び出せないので、ダミーから変更しないと動かない。
    // interpreter.rs を修正するステップが必要。
    
    Err("Initialize logic pending pub(crate) access".to_string())
}

/// HTMLエスケープ
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// ComponentDefからHTMLを生成
pub fn render_component(
    component: &ComponentDef,
    _interpreter: &mut Interpreter,
) -> Result<String, String> {
    // コンポーネントのrender部分を見つけてHTMLに変換
    for item in &component.body {
        if let ComponentBodyItem::Render(render) = item {
            // render内の文を評価（JSX要素を探す）
            for stmt in &render.body {
                if let Statement::Expression(Expression::JsxElement(jsx)) = stmt {
                    // ダミーのinterpreterで評価
                    // コンポーネントのプロパティやステートを渡したいが、
                    // 現状の簡易実装では新規Envで実行
                    let mut temp_interpreter = Interpreter::new();
                    return render_jsx(jsx, &mut temp_interpreter);
                }
            }
        }
    }
    Ok("<div>Empty component</div>".to_string())
}

/// フルHTMLページを生成
pub fn generate_html_page(title: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
        }}
    </style>
</head>
<body>
    {}
</body>
</html>"#,
        escape_html(title),
        body
    )
}

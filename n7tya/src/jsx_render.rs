//! JSX Rendering for n7tya-lang
//!
//! JSX要素をHTML文字列に変換するレンダラ

use crate::ast::*;
use crate::interpreter::{Value, Interpreter};

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
    // Interpreterの eval_expression を呼び出す
    // ここではpub化されていないので、シンプルな評価を行う
    match expr {
        Expression::Literal(Literal::Str(s)) => Ok(Value::Str(s.clone())),
        Expression::Literal(Literal::Int(n)) => Ok(Value::Int(*n)),
        Expression::Literal(Literal::Bool(b)) => Ok(Value::Bool(*b)),
        Expression::Identifier(name) => {
            // 変数の取得（Interpreterの環境にアクセスする必要があるため、ここではモック）
            Ok(Value::Str(format!("{{{}}}", name)))
        }
        _ => Ok(Value::Str("[complex expression]".to_string())),
    }
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
pub fn render_component(component: &ComponentDef, _interpreter: &mut Interpreter) -> Result<String, String> {
    // コンポーネントのrender部分を見つけてHTMLに変換
    for item in &component.body {
        if let ComponentBodyItem::Render(render) = item {
            // render内の文を評価（JSX要素を探す）
            for stmt in &render.body {
                if let Statement::Expression(Expression::JsxElement(jsx)) = stmt {
                    // ダミーのinterpreterで評価
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

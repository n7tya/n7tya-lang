//! n7tya-lang Standard Library / Builtins
//!
//! 標準で利用可能な組み込み関数群

use crate::interpreter::Value;
use std::io::{self, Write};

/// 組み込み関数の実行
pub fn call_builtin(name: &str, args: Vec<Value>) -> Result<Value, String> {
    match name {
        "print" => builtin_print(args),
        "println" => builtin_println(args),
        "len" => builtin_len(args),
        "range" => builtin_range(args),
        "input" => builtin_input(args),
        "str" => builtin_str(args),
        "int" => builtin_int(args),
        "float" => builtin_float(args),
        "type" => builtin_type(args),
        "abs" => builtin_abs(args),
        "min" => builtin_min(args),
        "max" => builtin_max(args),
        _ if name.starts_with("__class_") => {
            // クラスコンストラクタ
            let class_name = name.strip_prefix("__class_").unwrap();
            Ok(Value::Class(class_name.to_string(), std::collections::HashMap::new()))
        }
        _ => Err(format!("Unknown builtin function: {}", name)),
    }
}

fn builtin_print(args: Vec<Value>) -> Result<Value, String> {
    let output: Vec<String> = args.iter().map(|v| v.display()).collect();
    println!("{}", output.join(" "));
    Ok(Value::None)
}

fn builtin_println(args: Vec<Value>) -> Result<Value, String> {
    let output: Vec<String> = args.iter().map(|v| v.display()).collect();
    println!("{}", output.join(" "));
    Ok(Value::None)
}

fn builtin_len(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::List(items)) => Ok(Value::Int(items.len() as i64)),
        Some(Value::Str(s)) => Ok(Value::Int(s.len() as i64)),
        _ => Err("len() expects list or string".to_string()),
    }
}

fn builtin_range(args: Vec<Value>) -> Result<Value, String> {
    match args.as_slice() {
        [Value::Int(n)] => {
            let list: Vec<Value> = (0..*n).map(Value::Int).collect();
            Ok(Value::List(list))
        }
        [Value::Int(start), Value::Int(end)] => {
            let list: Vec<Value> = (*start..*end).map(Value::Int).collect();
            Ok(Value::List(list))
        }
        [Value::Int(start), Value::Int(end), Value::Int(step)] => {
            let mut list = Vec::new();
            let mut i = *start;
            if *step > 0 {
                while i < *end {
                    list.push(Value::Int(i));
                    i += step;
                }
            } else if *step < 0 {
                while i > *end {
                    list.push(Value::Int(i));
                    i += step;
                }
            }
            Ok(Value::List(list))
        }
        _ => Err("range() expects 1-3 integer arguments".to_string()),
    }
}

fn builtin_input(args: Vec<Value>) -> Result<Value, String> {
    if let Some(Value::Str(prompt)) = args.first() {
        print!("{}", prompt);
        io::stdout().flush().ok();
    }
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {}", e))?;
    
    Ok(Value::Str(input.trim_end().to_string()))
}

fn builtin_str(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(v) => Ok(Value::Str(v.display())),
        None => Err("str() requires an argument".to_string()),
    }
}

fn builtin_int(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(n)) => Ok(Value::Int(*n)),
        Some(Value::Float(f)) => Ok(Value::Int(*f as i64)),
        Some(Value::Str(s)) => {
            s.parse::<i64>()
                .map(Value::Int)
                .map_err(|_| format!("Cannot convert '{}' to int", s))
        }
        Some(Value::Bool(b)) => Ok(Value::Int(if *b { 1 } else { 0 })),
        _ => Err("int() requires a numeric or string argument".to_string()),
    }
}

fn builtin_float(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(n)) => Ok(Value::Float(*n as f64)),
        Some(Value::Float(f)) => Ok(Value::Float(*f)),
        Some(Value::Str(s)) => {
            s.parse::<f64>()
                .map(Value::Float)
                .map_err(|_| format!("Cannot convert '{}' to float", s))
        }
        _ => Err("float() requires a numeric or string argument".to_string()),
    }
}

fn builtin_type(args: Vec<Value>) -> Result<Value, String> {
    let type_name = match args.first() {
        Some(Value::Int(_)) => "Int",
        Some(Value::Float(_)) => "Float",
        Some(Value::Str(_)) => "Str",
        Some(Value::Bool(_)) => "Bool",
        Some(Value::List(_)) => "List",
        Some(Value::None) => "None",
        Some(Value::Fn(_, _)) => "Fn",
        Some(Value::BuiltinFn(_)) => "BuiltinFn",
        Some(Value::Class(name, _)) => return Ok(Value::Str(name.clone())),
        Some(Value::Return(_)) => "Return",
        None => return Err("type() requires an argument".to_string()),
    };
    Ok(Value::Str(type_name.to_string()))
}

fn builtin_abs(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(n)) => Ok(Value::Int(n.abs())),
        Some(Value::Float(f)) => Ok(Value::Float(f.abs())),
        _ => Err("abs() requires a numeric argument".to_string()),
    }
}

fn builtin_min(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("min() requires at least one argument".to_string());
    }
    
    if let Some(Value::List(list)) = args.first() {
        if list.is_empty() {
            return Err("min() arg is an empty list".to_string());
        }
        // リストの最小値を見つける
        let mut min = match &list[0] {
            Value::Int(n) => *n,
            _ => return Err("min() requires list of integers".to_string()),
        };
        for item in list.iter().skip(1) {
            if let Value::Int(n) = item {
                if *n < min { min = *n; }
            }
        }
        return Ok(Value::Int(min));
    }
    
    // 複数の引数
    let mut min = match &args[0] {
        Value::Int(n) => *n,
        _ => return Err("min() requires integers".to_string()),
    };
    for arg in args.iter().skip(1) {
        if let Value::Int(n) = arg {
            if *n < min { min = *n; }
        }
    }
    Ok(Value::Int(min))
}

fn builtin_max(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("max() requires at least one argument".to_string());
    }
    
    if let Some(Value::List(list)) = args.first() {
        if list.is_empty() {
            return Err("max() arg is an empty list".to_string());
        }
        let mut max = match &list[0] {
            Value::Int(n) => *n,
            _ => return Err("max() requires list of integers".to_string()),
        };
        for item in list.iter().skip(1) {
            if let Value::Int(n) = item {
                if *n > max { max = *n; }
            }
        }
        return Ok(Value::Int(max));
    }
    
    let mut max = match &args[0] {
        Value::Int(n) => *n,
        _ => return Err("max() requires integers".to_string()),
    };
    for arg in args.iter().skip(1) {
        if let Value::Int(n) = arg {
            if *n > max { max = *n; }
        }
    }
    Ok(Value::Int(max))
}

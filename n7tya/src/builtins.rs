//! n7tya-lang Standard Library / Builtins
//!
//! 標準で利用可能な組み込み関数群

use crate::interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::rc::Rc;

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
        "sum" => builtin_sum(args),
        "sorted" => builtin_sorted(args),
        "reversed" => builtin_reversed(args),
        "enumerate" => builtin_enumerate(args),
        "zip" => builtin_zip(args),
        "filter" => builtin_filter(args),
        "map" => builtin_map(args),
        // fs モジュール
        "fs.read_file" => builtin_fs_read_file(args),
        "fs.write_file" => builtin_fs_write_file(args),
        "fs.exists" => builtin_fs_exists(args),
        "fs.remove" => builtin_fs_remove(args),
        "fs.read_dir" => builtin_fs_read_dir(args),
        // json モジュール
        "json.parse" => builtin_json_parse(args),
        "json.stringify" => builtin_json_stringify(args),
        // http モジュール
        "http.get" => builtin_http_get(args),
        "http.post" => builtin_http_post(args),
        _ if name.starts_with("__class_") => {
            // クラスコンストラクタ
            let class_name = name.strip_prefix("__class_").unwrap();
            Ok(Value::Class(
                class_name.to_string(),
                Rc::new(RefCell::new(HashMap::new())),
            ))
        }
        _ => Err(format!("Unknown builtin function: {}", name)),
    }
}

fn builtin_print(args: Vec<Value>) -> Result<Value, String> {
    let output: Vec<String> = args.iter().map(|v| v.display()).collect();
    print!("{}", output.join(" "));
    io::stdout().flush().ok();
    Ok(Value::None)
}

fn builtin_println(args: Vec<Value>) -> Result<Value, String> {
    let output: Vec<String> = args.iter().map(|v| v.display()).collect();
    println!("{}", output.join(" "));
    Ok(Value::None)
}

fn builtin_len(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::List(items)) => Ok(Value::Int(items.borrow().len() as i64)),
        Some(Value::Str(s)) => Ok(Value::Int(s.len() as i64)),
        Some(Value::Dict(d)) => Ok(Value::Int(d.borrow().len() as i64)),
        Some(Value::Set(s)) => Ok(Value::Int(s.borrow().len() as i64)),
        _ => Err("len() expects list, string, dict, or set".to_string()),
    }
}

fn builtin_range(args: Vec<Value>) -> Result<Value, String> {
    match args.as_slice() {
        [Value::Int(n)] => {
            let list: Vec<Value> = (0..*n).map(Value::Int).collect();
            Ok(Value::List(Rc::new(RefCell::new(list))))
        }
        [Value::Int(start), Value::Int(end)] => {
            let list: Vec<Value> = (*start..*end).map(Value::Int).collect();
            Ok(Value::List(Rc::new(RefCell::new(list))))
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
            Ok(Value::List(Rc::new(RefCell::new(list))))
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
    io::stdin()
        .read_line(&mut input)
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
        Some(Value::Str(s)) => s
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| format!("Cannot convert '{}' to int", s)),
        Some(Value::Bool(b)) => Ok(Value::Int(if *b { 1 } else { 0 })),
        _ => Err("int() requires a numeric or string argument".to_string()),
    }
}

fn builtin_float(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::Int(n)) => Ok(Value::Float(*n as f64)),
        Some(Value::Float(f)) => Ok(Value::Float(*f)),
        Some(Value::Str(s)) => s
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| format!("Cannot convert '{}' to float", s)),
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
        Some(Value::Dict(_)) => "Dict",
        Some(Value::Set(_)) => "Set",
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
        let list = list.borrow();
        if list.is_empty() {
            return Err("min() arg is an empty list".to_string());
        }
        let mut min = match &list[0] {
            Value::Int(n) => *n,
            _ => return Err("min() requires list of integers".to_string()),
        };
        for item in list.iter().skip(1) {
            if let Value::Int(n) = item {
                if *n < min {
                    min = *n;
                }
            }
        }
        return Ok(Value::Int(min));
    }

    let mut min = match &args[0] {
        Value::Int(n) => *n,
        _ => return Err("min() requires integers".to_string()),
    };
    for arg in args.iter().skip(1) {
        if let Value::Int(n) = arg {
            if *n < min {
                min = *n;
            }
        }
    }
    Ok(Value::Int(min))
}

fn builtin_max(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("max() requires at least one argument".to_string());
    }

    if let Some(Value::List(list)) = args.first() {
        let list = list.borrow();
        if list.is_empty() {
            return Err("max() arg is an empty list".to_string());
        }
        let mut max = match &list[0] {
            Value::Int(n) => *n,
            _ => return Err("max() requires list of integers".to_string()),
        };
        for item in list.iter().skip(1) {
            if let Value::Int(n) = item {
                if *n > max {
                    max = *n;
                }
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
            if *n > max {
                max = *n;
            }
        }
    }
    Ok(Value::Int(max))
}

// ===== 新しいビルトイン関数 =====

fn builtin_sum(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::List(list)) => {
            let list = list.borrow();
            let mut sum = 0i64;
            for item in list.iter() {
                match item {
                    Value::Int(n) => sum += n,
                    _ => return Err("sum() requires list of integers".to_string()),
                }
            }
            Ok(Value::Int(sum))
        }
        _ => Err("sum() expects a list argument".to_string()),
    }
}

fn builtin_sorted(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::List(list)) => {
            let list = list.borrow();
            let mut ints: Vec<i64> = Vec::new();
            for item in list.iter() {
                match item {
                    Value::Int(n) => ints.push(*n),
                    _ => return Err("sorted() requires list of integers".to_string()),
                }
            }
            ints.sort();
            let result: Vec<Value> = ints.into_iter().map(Value::Int).collect();
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }
        _ => Err("sorted() expects a list argument".to_string()),
    }
}

fn builtin_reversed(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::List(list)) => {
            let list = list.borrow();
            let result: Vec<Value> = list.iter().rev().cloned().collect();
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }
        Some(Value::Str(s)) => {
            let reversed: String = s.chars().rev().collect();
            Ok(Value::Str(reversed))
        }
        _ => Err("reversed() expects a list or string argument".to_string()),
    }
}

fn builtin_enumerate(args: Vec<Value>) -> Result<Value, String> {
    match args.first() {
        Some(Value::List(list)) => {
            let list = list.borrow();
            let result: Vec<Value> = list
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    Value::List(Rc::new(RefCell::new(vec![Value::Int(i as i64), v.clone()])))
                })
                .collect();
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }
        _ => Err("enumerate() expects a list argument".to_string()),
    }
}

fn builtin_zip(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("zip() expects exactly 2 arguments".to_string());
    }

    match (&args[0], &args[1]) {
        (Value::List(list1), Value::List(list2)) => {
            let list1 = list1.borrow();
            let list2 = list2.borrow();
            let result: Vec<Value> = list1
                .iter()
                .zip(list2.iter())
                .map(|(a, b)| Value::List(Rc::new(RefCell::new(vec![a.clone(), b.clone()]))))
                .collect();
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }
        _ => Err("zip() expects two list arguments".to_string()),
    }
}

fn builtin_filter(_args: Vec<Value>) -> Result<Value, String> {
    // filter() は高階関数なので、Interpreter 側で実装する必要がある
    Err("filter() is not yet implemented as a builtin".to_string())
}

fn builtin_map(_args: Vec<Value>) -> Result<Value, String> {
    // map() は高階関数なので、Interpreter 側で実装する必要がある
    Err("map() is not yet implemented as a builtin".to_string())
}

// ============================================================
// fs モジュール - ファイルシステム操作
// ============================================================

fn builtin_fs_read_file(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.read_file() takes exactly 1 argument".to_string());
    }
    if let Value::Str(path) = &args[0] {
        match fs::read_to_string(path) {
            Ok(content) => Ok(Value::Str(content)),
            Err(e) => Err(format!("Failed to read file '{}': {}", path, e)),
        }
    } else {
        Err("fs.read_file() expects a string path".to_string())
    }
}

fn builtin_fs_write_file(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("fs.write_file() takes exactly 2 arguments".to_string());
    }
    if let (Value::Str(path), Value::Str(content)) = (&args[0], &args[1]) {
        match fs::write(path, content) {
            Ok(_) => Ok(Value::None),
            Err(e) => Err(format!("Failed to write file '{}': {}", path, e)),
        }
    } else {
        Err("fs.write_file() expects (path: Str, content: Str)".to_string())
    }
}

fn builtin_fs_exists(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.exists() takes exactly 1 argument".to_string());
    }
    if let Value::Str(path) = &args[0] {
        Ok(Value::Bool(std::path::Path::new(path).exists()))
    } else {
        Err("fs.exists() expects a string path".to_string())
    }
}

fn builtin_fs_remove(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.remove() takes exactly 1 argument".to_string());
    }
    if let Value::Str(path) = &args[0] {
        let path_obj = std::path::Path::new(path);
        let result = if path_obj.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        };
        match result {
            Ok(_) => Ok(Value::None),
            Err(e) => Err(format!("Failed to remove '{}': {}", path, e)),
        }
    } else {
        Err("fs.remove() expects a string path".to_string())
    }
}

fn builtin_fs_read_dir(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("fs.read_dir() takes exactly 1 argument".to_string());
    }
    if let Value::Str(path) = &args[0] {
        match fs::read_dir(path) {
            Ok(entries) => {
                let names: Vec<Value> = entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .map(Value::Str)
                    .collect();
                Ok(Value::List(Rc::new(RefCell::new(names))))
            }
            Err(e) => Err(format!("Failed to read directory '{}': {}", path, e)),
        }
    } else {
        Err("fs.read_dir() expects a string path".to_string())
    }
}

// ============================================================
// json モジュール - JSON操作
// ============================================================

fn json_to_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::None,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::None
            }
        }
        serde_json::Value::String(s) => Value::Str(s),
        serde_json::Value::Array(arr) => {
            let values: Vec<Value> = arr.into_iter().map(json_to_value).collect();
            Value::List(Rc::new(RefCell::new(values)))
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_value(v));
            }
            Value::Dict(Rc::new(RefCell::new(map)))
        }
    }
}

fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::None => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(n) => serde_json::Value::Number((*n).into()),
        Value::Float(f) => {
            serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        Value::Str(s) => serde_json::Value::String(s.clone()),
        Value::List(list) => {
            let arr: Vec<serde_json::Value> = list.borrow().iter().map(value_to_json).collect();
            serde_json::Value::Array(arr)
        }
        Value::Dict(dict) => {
            let obj: serde_json::Map<String, serde_json::Value> = dict
                .borrow()
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::Null,
    }
}

fn builtin_json_parse(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("json.parse() takes exactly 1 argument".to_string());
    }
    if let Value::Str(s) = &args[0] {
        match serde_json::from_str::<serde_json::Value>(s) {
            Ok(json) => Ok(json_to_value(json)),
            Err(e) => Err(format!("JSON parse error: {}", e)),
        }
    } else {
        Err("json.parse() expects a string".to_string())
    }
}

fn builtin_json_stringify(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("json.stringify() takes exactly 1 argument".to_string());
    }
    let json = value_to_json(&args[0]);
    match serde_json::to_string(&json) {
        Ok(s) => Ok(Value::Str(s)),
        Err(e) => Err(format!("JSON stringify error: {}", e)),
    }
}

// ============================================================
// http モジュール - HTTPクライアント
// ============================================================

fn builtin_http_get(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("http.get() takes exactly 1 argument".to_string());
    }
    if let Value::Str(url) = &args[0] {
        match ureq::get(url).call() {
            Ok(response) => {
                let body = response.into_string().unwrap_or_default();
                Ok(Value::Str(body))
            }
            Err(e) => Err(format!("HTTP GET error: {}", e)),
        }
    } else {
        Err("http.get() expects a URL string".to_string())
    }
}

fn builtin_http_post(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("http.post() takes at least 2 arguments (url, body)".to_string());
    }
    if let (Value::Str(url), body) = (&args[0], &args[1]) {
        let body_str = match body {
            Value::Str(s) => s.clone(),
            _ => {
                // 自動的にJSONにシリアライズ
                let json = value_to_json(body);
                serde_json::to_string(&json).unwrap_or_default()
            }
        };
        
        match ureq::post(url)
            .set("Content-Type", "application/json")
            .send_string(&body_str)
        {
            Ok(response) => {
                let body = response.into_string().unwrap_or_default();
                Ok(Value::Str(body))
            }
            Err(e) => Err(format!("HTTP POST error: {}", e)),
        }
    } else {
        Err("http.post() expects (url: Str, body)".to_string())
    }
}

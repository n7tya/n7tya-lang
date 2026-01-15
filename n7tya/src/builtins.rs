//! n7tya-lang Standard Library / Builtins
//!
//! 標準で利用可能な組み込み関数群

use crate::interpreter::Value;
use std::cell::RefCell;
use std::collections::HashMap;
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

//! Python Integration for n7tya-lang
//!
//! pyo3を使用したPythonライブラリ連携

use crate::interpreter::Value;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use std::collections::HashMap;

/// Pythonランタイムを初期化
pub fn init_python() -> Result<(), String> {
    pyo3::prepare_freethreaded_python();
    Ok(())
}

/// Pythonモジュールをインポート
pub fn python_import(module_name: &str) -> Result<PyObject, String> {
    Python::with_gil(|py| {
        py.import(module_name)
            .map(|m| m.into_py(py))
            .map_err(|e| format!("Failed to import '{}': {}", module_name, e))
    })
}

/// Pythonモジュールから関数を取得
pub fn get_python_function(module_name: &str, func_name: &str) -> Result<PyObject, String> {
    Python::with_gil(|py| {
        let module = py
            .import(module_name)
            .map_err(|e| format!("Failed to import '{}': {}", module_name, e))?;
        module
            .getattr(func_name)
            .map(|f| f.into_py(py))
            .map_err(|e| {
                format!(
                    "Failed to get '{}' from '{}': {}",
                    func_name, module_name, e
                )
            })
    })
}

/// Python関数を呼び出し
pub fn call_python_function(func: &PyObject, args: Vec<Value>) -> Result<Value, String> {
    Python::with_gil(|py| {
        let py_args: Vec<PyObject> = args.iter().map(|v| value_to_py(py, v)).collect();

        let result = func
            .call1(py, PyTuple::new(py, &py_args).unwrap())
            .map_err(|e| format!("Python call error: {}", e))?;

        py_to_value(py, &result)
    })
}

/// Pythonコードを直接実行
pub fn run_python_code(code: &str) -> Result<Value, String> {
    Python::with_gil(|py| {
        let locals = PyDict::new(py);

        // PyAnyMethods::run を使用
        py.run(c_str(code)?, None, Some(&locals))
            .map_err(|e| format!("Python execution error: {}", e))?;

        // 結果として__result__変数を探す
        if let Ok(Some(result)) = locals.get_item("__result__") {
            py_to_value(py, &result.into_py(py))
        } else {
            Ok(Value::None)
        }
    })
}

// CStr変換ヘルパー（簡易版: 実行時に使用しない）
fn c_str(_s: &str) -> Result<&std::ffi::CStr, String> {
    // pyo3 0.23では run_bound を使うべきだが、
    // ここではスタブとしてエラーを返す
    Err("Direct Python code execution not supported in this version".to_string())
}

/// n7tyaの値をPyObjectに変換
pub fn value_to_py(py: Python, value: &Value) -> PyObject {
    match value {
        Value::Int(n) => n.into_py(py),
        Value::Float(f) => f.into_py(py),
        Value::Str(s) => s.into_py(py),
        Value::Bool(b) => b.into_py(py),
        Value::None => py.None(),
        Value::List(items) => {
            let py_items: Vec<PyObject> = items.iter().map(|v| value_to_py(py, v)).collect();
            PyList::new(py, &py_items).unwrap().into_py(py)
        }
        _ => py.None(),
    }
}

/// PyObjectをn7tyaの値に変換
pub fn py_to_value(py: Python, obj: &PyObject) -> Result<Value, String> {
    let obj_ref = obj.bind(py);

    // 型を判定して変換
    if let Ok(val) = obj_ref.extract::<i64>() {
        return Ok(Value::Int(val));
    }
    if let Ok(val) = obj_ref.extract::<f64>() {
        return Ok(Value::Float(val));
    }
    if let Ok(val) = obj_ref.extract::<bool>() {
        return Ok(Value::Bool(val));
    }
    if let Ok(val) = obj_ref.extract::<String>() {
        return Ok(Value::Str(val));
    }
    if obj_ref.is_none() {
        return Ok(Value::None);
    }
    if let Ok(list) = obj_ref.downcast::<PyList>() {
        let items: Result<Vec<Value>, String> = list
            .iter()
            .map(|item| py_to_value(py, &item.into_py(py)))
            .collect();
        return Ok(Value::List(items?));
    }

    // 未対応の型はNoneとして扱う
    Ok(Value::None)
}

/// Pythonパッケージをインストール（pipを使用）
pub fn install_python_package(package: &str) -> Result<(), String> {
    Python::with_gil(|py| {
        let subprocess = py
            .import("subprocess")
            .map_err(|e| format!("Failed to import subprocess: {}", e))?;

        let args = PyList::new(py, &["pip", "install", package]).unwrap();
        subprocess
            .call_method1("run", (args,))
            .map_err(|e| format!("Failed to install '{}': {}", package, e))?;

        Ok(())
    })
}

/// Pythonモジュールのラッパー
pub struct PythonModule {
    module: PyObject,
}

impl PythonModule {
    /// モジュールをロード
    pub fn load(name: &str) -> Result<Self, String> {
        let module = python_import(name)?;
        Ok(Self { module })
    }

    /// 関数を呼び出す
    pub fn call(&self, func_name: &str, args: Vec<Value>) -> Result<Value, String> {
        Python::with_gil(|py| {
            let func = self
                .module
                .bind(py)
                .getattr(func_name)
                .map_err(|e| format!("Function '{}' not found: {}", func_name, e))?;

            let py_args: Vec<PyObject> = args.iter().map(|v| value_to_py(py, v)).collect();

            let result = func
                .call1(PyTuple::new(py, &py_args).unwrap())
                .map_err(|e| format!("Call error: {}", e))?;

            py_to_value(py, &result.into_py(py))
        })
    }

    /// 属性を取得
    pub fn get_attr(&self, name: &str) -> Result<Value, String> {
        Python::with_gil(|py| {
            let attr = self
                .module
                .bind(py)
                .getattr(name)
                .map_err(|e| format!("Attribute '{}' not found: {}", name, e))?;
            py_to_value(py, &attr.into_py(py))
        })
    }
}

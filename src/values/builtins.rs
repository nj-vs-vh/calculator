use super::Value;
use rand::Rng;

use crate::values::function::Function;

pub type BuiltinFunction = fn(&Value) -> Result<Value, String>;

fn log(arg: &Value) -> Result<Value, String> {
    match arg {
        Value::Float(v) => Ok(Value::Float(v.ln())),
        Value::Int(v) => log(&Value::Float(*v as f32)),
        a => not_defined_for_arg("log", a),
    }
}
fn exp(arg: &Value) -> Result<Value, String> {
    match arg {
        Value::Float(v) => Ok(Value::Float(v.exp())),
        Value::Int(v) => exp(&Value::Float(*v as f32)),
        a => not_defined_for_arg("exp", a),
    }
}
fn print(arg: &Value) -> Result<Value, String> {
    println!("{}", arg);
    Ok(Value::Nothing)
}
fn length(arg: &Value) -> Result<Value, String> {
    match arg {
        Value::String(s) => Ok(Value::Int(s.len() as i32)),
        a => not_defined_for_arg("length", a),
    }
}
fn random(arg: &Value) -> Result<Value, String> {
    let mut rng = rand::thread_rng();
    if let Value::Nothing = arg {
        Ok(Value::Float(rng.gen::<f32>()))
    } else {
        Err("\"random\" built-in function accepts no arguments".into())
    }
}
fn mod_(arg: &Value) -> Result<Value, String> {
    if let Value::Tuple(elements) = arg {
        if let [a, b] = &elements[..] {
            if let (Value::Int(i1), Value::Int(i2)) = (a.clone().as_ref(), b.clone().as_ref()) {
                return Ok(Value::Int(i1 % i2));
            }
        }
    }
    Err("\"mod\" accepts two integer arguments".into())
}

pub fn builtin(name: &str) -> Option<Function> {
    match name {
        "log" => Some(Function::Builtin(log)),
        "exp" => Some(Function::Builtin(exp)),
        "print" => Some(Function::Builtin(print)),
        "length" => Some(Function::Builtin(length)),
        "random" => Some(Function::Builtin(random)),
        "mod" => Some(Function::Builtin(mod_)),
        _ => None,
    }
}

fn not_defined_for_arg(func_name: &str, arg: &Value) -> Result<Value, String> {
    Err(format!(
        "\"{}\" built-in function is not defined for arg of type \"{}\"",
        func_name,
        arg.type_name()
    ))
}

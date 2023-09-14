use crate::errors::RuntimeError;
use crate::parser::{BinaryOp, Expression};
use crate::values::Value;

pub fn eval(expressions: &[Expression]) -> Result<Vec<Box<Value>>, RuntimeError> {
    let mut results: Vec<Box<Value>> = Vec::new();
    for expr in expressions {
        results.push(eval_expression(expr)?);
    }
    return Ok(results);
}

#[macro_export]
macro_rules! apply {
    ( $func:expr, $left:expr, $right:expr, $op_name:expr ) => {{
        let maybe_res = $func(&$left, &$right);
        match maybe_res {
            Some(v) => Ok(Box::new(v)),
            None => Err(RuntimeError {
                errmsg: format!("{} is not defined for {} and {}", $op_name, $left, $right),
            }),
        }
    }};
}

fn eval_expression(expr: &Expression) -> Result<Box<Value>, RuntimeError> {
    match expr {
        Expression::Value(v) => Ok(v.clone()),
        Expression::Bin(binary_operation) => {
            let right_value = eval_expression(&binary_operation.right)?;
            if binary_operation.op == BinaryOp::Assign {
                // TBD
                return Ok(right_value);
            }
            let left_value = eval_expression(&binary_operation.left)?;
            match binary_operation.op {
                BinaryOp::Add => apply!(add, left_value, right_value, "addition"),
                BinaryOp::Sub => apply!(sub, left_value, right_value, "addition"),
                BinaryOp::Mul => apply!(mul, left_value, right_value, "addition"),
                BinaryOp::Div => apply!(div, left_value, right_value, "addition"),
                BinaryOp::Pow => apply!(pow, left_value, right_value, "addition"),
                _ => todo!(),
            }
        }
        _ => Err(RuntimeError {
            errmsg: "TBD".into(),
        }),
    }
}

fn add(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => Some(Value::Float(f1 + f2)),
        _ => None,
    }
}
fn sub(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => Some(Value::Float(f1 - f2)),
        _ => None,
    }
}
fn mul(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => Some(Value::Float(f1 * f2)),
        _ => None,
    }
}
fn div(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => Some(Value::Float(f1 / f2)),
        _ => None,
    }
}
fn pow(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => Some(Value::Float(f1.powf(*f2))),
        _ => None,
    }
}

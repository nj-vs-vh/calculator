use std::collections::HashMap;

use crate::errors::{ParserError, RuntimeError};
use crate::parser::{BinaryOp, Expression};
use crate::values::Value;

pub fn eval(expressions: &[Expression]) -> Result<Vec<Box<Value>>, RuntimeError> {
    let mut results: Vec<Box<Value>> = Vec::new();
    let mut variables: HashMap<String, Box<Value>> = HashMap::new();
    for expr in expressions {
        results.push(eval_expression(expr, &mut variables)?);
    }
    return Ok(results);
}

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

fn eval_expression(
    expr: &Expression,
    variables: &mut HashMap<String, Box<Value>>,
) -> Result<Box<Value>, RuntimeError> {
    match expr {
        Expression::Value(v) => Ok(v.clone()),
        Expression::Variable(var_name) => {
            variables
                .get(var_name)
                .map(|ref_| ref_.clone())
                .ok_or(RuntimeError {
                    errmsg: format!("reference to non-existent variable \"{}\"", var_name),
                })
        }
        Expression::Bin(binary_operation) => {
            let right_value = eval_expression(&binary_operation.right, variables)?;
            if binary_operation.op == BinaryOp::Assign {
                if let Expression::Variable(var_name) = binary_operation.left.clone().as_ref() {
                    variables.insert(var_name.clone(), right_value.clone());
                    return Ok(right_value);
                } else {
                    return Err(RuntimeError {
                        errmsg: format!("can only assign to variables"),
                    });
                };
            }
            let left_value = eval_expression(&binary_operation.left, variables)?;
            match binary_operation.op {
                BinaryOp::Add => apply!(add, left_value, right_value, "addition"),
                BinaryOp::Sub => apply!(sub, left_value, right_value, "subtraction"),
                BinaryOp::Mul => apply!(mul, left_value, right_value, "multiplication"),
                BinaryOp::Div => apply!(div, left_value, right_value, "division"),
                BinaryOp::Pow => apply!(pow, left_value, right_value, "power"),
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

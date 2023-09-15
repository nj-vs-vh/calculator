use std::collections::HashMap;
use std::ops::Deref;

use crate::errors::RuntimeError;
use crate::parser::{BinaryOp, Expression, UnaryOp};
use crate::values::functions::builtin;
use crate::values::Value;

macro_rules! apply_bin {
    ( $func:expr, $left:expr, $right:expr, $op_name:expr ) => {{
        let maybe_res = $func(&$left, &$right);
        match maybe_res {
            Some(v) => Ok(Box::new(v)),
            None => Err(RuntimeError {
                errmsg: format!(
                    "{} is not defined for {} and {}",
                    $op_name,
                    $left.type_name(),
                    $right.type_name()
                ),
            }),
        }
    }};
}

macro_rules! apply_un {
    ( $func:expr, $left:expr, $op_name:expr ) => {{
        let maybe_res = $func(&$left);
        match maybe_res {
            Some(v) => Ok(Box::new(v)),
            None => Err(RuntimeError {
                errmsg: format!("{} is not defined for {}", $op_name, $left.type_name(),),
            }),
        }
    }};
}

pub fn eval(
    expr: &Expression,
    variables: &mut HashMap<String, Box<Value>>,
) -> Result<Box<Value>, RuntimeError> {
    match expr {
        Expression::Value(v) => Ok(v.clone()),
        Expression::Variable(var_name) => {
            if let Some(value) = variables.get(var_name).map(|ref_| ref_.clone()) {
                return Ok(value);
            } else if let Some(builtin_func) = builtin(&var_name) {
                return Ok(Box::new(Value::Function(builtin_func)));
            } else {
                return Err(RuntimeError {
                    errmsg: format!("reference to non-existent variable \"{}\"", var_name),
                });
            }
        }
        Expression::Scope(scope) => {
            if scope.body.is_empty() {
                return Err(RuntimeError {
                    errmsg: "empty scope".into(),
                });
            }
            let mut results: Vec<Box<Value>> = Vec::new();
            let mut local_variables = variables.clone();
            for expr in scope.body.iter() {
                let expr_value = eval(expr, &mut local_variables)?;
                if let Value::Returned(v) = expr_value.clone().deref() {
                    if scope.is_returnable {
                        return Ok(v.clone());
                    } else {
                        // returned value is passed wrapped up to the first returnable scope
                        return Ok(expr_value.clone());
                    }
                }
                results.push(expr_value);
            }
            return Ok(results[results.len() - 1].clone());
        }
        Expression::Bin(binary_operation) => {
            let right_value = eval(&binary_operation.right, variables)?;
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
            let left_value = eval(&binary_operation.left, variables)?;
            match binary_operation.op {
                BinaryOp::Add => apply_bin!(add, left_value, right_value, "addition"),
                BinaryOp::Sub => apply_bin!(sub, left_value, right_value, "subtraction"),
                BinaryOp::Mul => apply_bin!(mul, left_value, right_value, "multiplication"),
                BinaryOp::Div => apply_bin!(div, left_value, right_value, "division"),
                BinaryOp::Pow => apply_bin!(pow, left_value, right_value, "power"),
                BinaryOp::IsEq => apply_bin!(eq, left_value, right_value, "equality"),
                BinaryOp::IsLt => apply_bin!(lt, left_value, right_value, "less-than"),
                BinaryOp::IsGt => apply_bin!(gt, left_value, right_value, "greater-than"),
                BinaryOp::FunctionCall => {
                    if let Value::Function(func) = left_value.clone().as_ref() {
                        match func.call(&right_value) {
                            Ok(result) => Ok(Box::new(result)),
                            Err(message) => Err(RuntimeError { errmsg: message }),
                        }
                    } else {
                        Err(RuntimeError {
                            errmsg: "not callable".into(),
                        })
                    }
                }
                BinaryOp::Assign => panic!("rtl assign"),
            }
        }
        Expression::Un(unary_operation) => {
            let operand = eval(&unary_operation.operand, variables)?;
            match unary_operation.op {
                UnaryOp::Neg => apply_un!(neg, operand, "negation"),
                UnaryOp::Return => Ok(Box::new(Value::Returned(operand))),
            }
        }
        Expression::If(if_) => {
            let condition = eval(&if_.condition, variables)?;
            if let Value::Bool(b) = condition.clone().as_ref() {
                if *b {
                    Ok(eval(&if_.if_true, variables)?)
                } else if let Some(if_false_expr) = if_.if_false.clone() {
                    Ok(eval(&if_false_expr, variables)?)
                } else {
                    Ok(Box::new(Value::Nothing))
                }
            } else {
                Err(RuntimeError {
                    errmsg: format!(
                        "if condition must evaluate to bool, got {}",
                        condition.type_name()
                    ),
                })
            }
        }
    }
}

fn add(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => Some(Value::Float(f1 + f2)),
        (Value::Int(i1), Value::Float(f2)) => Some(Value::Float(*i1 as f32 + *f2)),
        (Value::Float(_), Value::Int(_)) => add(b, a),
        (Value::Int(i1), Value::Int(i2)) => Some(Value::Int(i1 + i2)),
        (Value::String(s1), Value::String(s2)) => {
            let mut res = s1.clone();
            res.push_str(s2);
            Some(Value::String(res))
        }
        (Value::Bool(b1), Value::Bool(b2)) => Some(Value::Bool(*b1 || *b2)),
        _ => None,
    }
}
fn sub(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => Some(Value::Float(f1 - f2)),
        (Value::Int(i1), Value::Float(f2)) => Some(Value::Float(*i1 as f32 - *f2)),
        (Value::Float(f1), Value::Int(i2)) => Some(Value::Float(*f1 - *i2 as f32)),
        (Value::Int(i1), Value::Int(i2)) => Some(Value::Int(i1 - i2)),
        _ => None,
    }
}
fn mul(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => Some(Value::Float(f1 * f2)),
        (Value::Int(i1), Value::Float(f2)) => Some(Value::Float(*i1 as f32 * *f2)),
        (Value::Float(_), Value::Int(_)) => mul(b, a),
        (Value::Int(i1), Value::Int(i2)) => Some(Value::Int(i1 * i2)),
        (Value::Bool(b1), Value::Bool(b2)) => Some(Value::Bool(*b1 && *b2)),
        _ => None,
    }
}
fn div(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => Some(Value::Float(f1 / f2)),
        (Value::Int(i1), Value::Float(f2)) => Some(Value::Float(*i1 as f32 / *f2)),
        (Value::Float(f1), Value::Int(i2)) => Some(Value::Float(*f1 / *i2 as f32)),
        (Value::Int(i1), Value::Int(i2)) => Some(Value::Float((*i1 as f32) / (*i2 as f32))),
        _ => None,
    }
}
fn pow(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => Some(Value::Float(f1.powf(*f2))),
        (Value::Int(i1), Value::Float(f2)) => Some(Value::Float((*i1 as f32).powf(*f2))),
        (Value::Float(f1), Value::Int(i2)) => Some(Value::Float(f1.powi(*i2))),
        (Value::Int(i1), Value::Int(i2)) => Some(if *i2 > 0 {
            Value::Int(i1.pow(*i2 as u32))
        } else {
            Value::Float((*i1 as f32).powi(*i2))
        }),
        (Value::Bool(b1), Value::Bool(b2)) => Some(Value::Bool(b1 ^ b2)),
        _ => None,
    }
}
fn lt(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => Some(Value::Bool(f1 < f2)),
        (Value::Int(i1), Value::Float(f2)) => Some(Value::Bool((*i1 as f32) < *f2)),
        (Value::Float(f1), Value::Int(i2)) => Some(Value::Bool(*f1 < *i2 as f32)),
        (Value::Int(i1), Value::Int(i2)) => Some(Value::Bool(i1 < i2)),
        _ => None,
    }
}
fn gt(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Float(f1), Value::Float(f2)) => Some(Value::Bool(f1 > f2)),
        (Value::Int(i1), Value::Float(f2)) => Some(Value::Bool((*i1 as f32) > *f2)),
        (Value::Float(f1), Value::Int(i2)) => Some(Value::Bool(*f1 > *i2 as f32)),
        (Value::Int(i1), Value::Int(i2)) => Some(Value::Bool(i1 > i2)),
        _ => None,
    }
}
fn eq(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Int(i1), Value::Float(f2)) => Some(Value::Bool((*i1 as f32) == *f2)),
        (Value::Float(f1), Value::Int(i2)) => Some(Value::Bool(*f1 == *i2 as f32)),
        (a, b) => Some(Value::Bool(a == b)),
    }
}

fn neg(v: &Value) -> Option<Value> {
    match v {
        Value::Float(v) => Some(Value::Float(-v)),
        Value::Int(v) => Some(Value::Int(-v)),
        Value::Bool(b) => Some(Value::Bool(!b)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use crate::tokenize;
    use rstest::rstest;

    #[rstest]
    #[case("1", Value::Int(1))]
    #[case("1;", Value::Int(1))]
    #[case("(1);", Value::Int(1))]
    #[case("(((1)))", Value::Int(1))]
    #[case("1 + 1;", Value::Int(2))]
    #[case("1 + 2 * 3 ^ 2 * 5 + 10;", Value::Int(101))]
    #[case("1 + (2 * (3 ^ 2) * 5) + 10;", Value::Int(101))]
    #[case("10 / 5 / 2", Value::Float(1.0))]
    #[case("10 * 5 / 2", Value::Float(25.0))]
    #[case("5 / 5 * 2", Value::Float(2.0))]
    #[case("a = 5; b = 6; a + b", Value::Int(11))]
    #[case("a = 5; b = 6; d = c = a + b; d", Value::Int(11))]
    #[case("2 + -3", Value::Int(-1))]
    #[case("-3 ^ 4", Value::Int(-81))]
    #[case("log(1)", Value::Float(0.0))]
    #[case("exp(0)", Value::Float(1.0))]
    #[case("a = exp; a(0)", Value::Float(1.0))]
    #[case("{1} + {2}", Value::Int(3))]
    #[case("{1} + {2}", Value::Int(3))]
    #[case("True", Value::Bool(true))]
    #[case("tRuE", Value::Bool(true))]
    #[case("true + false", Value::Bool(true))]
    #[case("false + true", Value::Bool(true))]
    #[case("false + false", Value::Bool(false))]
    #[case("true + true", Value::Bool(true))]
    #[case("true * false", Value::Bool(false))]
    #[case("true * true", Value::Bool(true))]
    #[case("false * false", Value::Bool(false))]
    #[case("-false", Value::Bool(true))]
    #[case("1 == 1", Value::Bool(true))]
    #[case("- 1 == 1", Value::Bool(false))]
    #[case("-(1 == 2)", Value::Bool(true))]
    #[case("1 == \"foo\"", Value::Bool(false))]
    #[case("\"foo\" == \"foo\"", Value::Bool(true))]
    #[case("1 < 2", Value::Bool(true))]
    #[case("2 > 1", Value::Bool(true))]
    #[case("2 > 1 == true", Value::Bool(true))]
    #[case("true == 2 > 1", Value::Bool(true))]
    #[case("2.5 > 1", Value::Bool(true))]
    #[case("2.5 > 1.5", Value::Bool(true))]
    #[case("if true 1", Value::Int(1))]
    #[case("a = if true 1; b = a", Value::Int(1))]
    #[case("a = if true {1}; b = a", Value::Int(1))]
    #[case("if false 1", Value::Nothing)]
    #[case("if false 1 else 2", Value::Int(2))]
    #[case("a = 3; b = 5; res = if a < b 1 else 2; res", Value::Int(1))]
    #[case("a = 3; b = 5; res = if (a < b) { 1 } else { 2 }; res", Value::Int(1))]
    #[case("return 1; 2; 3; 4; 5; 6;", Value::Int(1))]
    #[case("if !(1 == 2) {return 1}; return 2", Value::Int(1))]
    #[case("if (1 == 2) {return 1}; return 2", Value::Int(2))]
    #[case("if (1 == 2) {return 1}; 2;", Value::Int(2))]
    fn test_runtime_basic(#[case] code: &str, #[case] expected_result: Value) {
        let code_ = String::from(code);
        let tokens = tokenize(&code_).unwrap();
        let ast = parse(&tokens).unwrap();
        let result = eval(&ast, &mut HashMap::new());
        assert_eq!(result.unwrap().as_ref().to_owned(), expected_result);
    }
}

use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use crate::errors::RuntimeError;
use crate::parser::{BinaryOp, Expression, UnaryOp};
use crate::values::builtins::builtin;
use crate::values::function::Function;
use crate::values::Value;

macro_rules! apply_bin {
    ( $func:expr, $left:expr, $right:expr, $op_name:expr ) => {{
        let maybe_res = $func(&$left, &$right);
        match maybe_res {
            Some(v) => Ok(Rc::new(v)),
            None => Err(format!(
                "{} is not defined for {} and {}",
                $op_name,
                $left.type_name(),
                $right.type_name()
            )),
        }
    }};
}

macro_rules! apply_un {
    ( $func:expr, $left:expr, $op_name:expr, $parent:expr ) => {{
        let maybe_res = $func(&$left);
        match maybe_res {
            Some(v) => Ok(Rc::new(v)),
            None => Err(RuntimeError {
                errmsg: format!("{} is not defined for {}", $op_name, $left.type_name(),),
                traceback: vec![$parent.clone()],
            }),
        }
    }};
}

pub fn eval(
    expression: &Expression,
    vars: &mut HashMap<String, Rc<Value>>,
) -> Result<Rc<Value>, RuntimeError> {
    let new_error = |errmsg: String| RuntimeError {
        errmsg,
        traceback: vec![expression.clone()],
    };
    let extend_traceback = |e: RuntimeError| RuntimeError {
        errmsg: e.errmsg,
        traceback: [e.traceback, vec![expression.clone()]].concat(),
    };
    match expression {
        Expression::Value(v) => Ok(Rc::clone(v)),
        Expression::Variable(var_name) => {
            if let Some(value) = vars.get(var_name).map(|v| Rc::clone(v)) {
                return Ok(value);
            } else if let Some(builtin_func) = builtin(&var_name) {
                return Ok(Rc::new(Value::Function(builtin_func)));
            } else {
                return Err(new_error(format!(
                    "reference to non-existent variable \"{}\"",
                    var_name
                )));
            }
        }
        Expression::Scope {
            body,
            is_returnable,
        } => {
            if body.is_empty() {
                return Ok(Rc::new(Value::Nothing));
            }
            let mut results: Vec<Rc<Value>> = Vec::new();
            for expr in body.iter() {
                let expr_value = eval(expr, vars)?;
                if let Value::Returned(v) = expr_value.clone().deref() {
                    if *is_returnable {
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
        Expression::BinaryOperation { op, left, right } => match op {
            BinaryOp::Assign => eval_assignment(&left, &right, vars).map_err(new_error),
            BinaryOp::FunctionCall => {
                let left_value = eval(&left, vars)?;
                if let Value::Function(func) = left_value.as_ref() {
                    match func {
                        Function::Builtin(builtin_func) => {
                            let arg_value = eval(&right, vars).map_err(extend_traceback)?;
                            builtin_func(&arg_value)
                                .map(|v| Rc::new(v))
                                .map_err(new_error)
                        }
                        Function::UserDefined(func) => {
                            let mut local_vars = vars.clone();
                            eval_assignment(&func.params, &right, &mut local_vars)
                                .map_err(new_error)?;
                            eval(&func.body, &mut local_vars).map_err(extend_traceback)
                        }
                    }
                } else {
                    Err(new_error(format!(
                        "\"{}\" is not callable",
                        left_value.type_name()
                    )))
                }
            }
            ltr_op => {
                let right_value = eval(&right, vars).map_err(extend_traceback)?;
                let left_value = eval(&left, vars).map_err(extend_traceback)?;
                match ltr_op {
                    BinaryOp::Add => apply_bin!(add, left_value, right_value, "addition"),
                    BinaryOp::Sub => apply_bin!(sub, left_value, right_value, "subtraction"),
                    BinaryOp::Mul => apply_bin!(mul, left_value, right_value, "multiplication"),
                    BinaryOp::Div => apply_bin!(div, left_value, right_value, "division"),
                    BinaryOp::Pow => apply_bin!(pow, left_value, right_value, "power"),
                    BinaryOp::IsEq => apply_bin!(eq, left_value, right_value, "equality"),
                    BinaryOp::IsLt => apply_bin!(lt, left_value, right_value, "less-than"),
                    BinaryOp::IsGt => apply_bin!(gt, left_value, right_value, "greater-than"),
                    BinaryOp::FormTuple => Ok(Rc::new(Value::Tuple(vec![left_value, right_value]))),
                    BinaryOp::AppendToTuple => {
                        if let Value::Tuple(left_tuple) = left_value.to_owned().as_ref() {
                            let mut left_tuple_copy = left_tuple.clone();
                            left_tuple_copy.push(right_value);
                            Ok(Rc::new(Value::Tuple(left_tuple_copy)))
                        } else {
                            Err("internal error: can't append to non-tuple value".into())
                        }
                    }
                    _ => panic!("RTL op "),
                }
                .map_err(new_error)
            }
        },
        Expression::UnaryOperation { op, operand } => {
            let operand = eval(&operand, vars).map_err(extend_traceback)?;
            match op {
                UnaryOp::Neg => apply_un!(neg, operand, "negation", expression),
                UnaryOp::Return => Ok(Rc::new(Value::Returned(operand))),
            }
            .map_err(extend_traceback)
        }
        Expression::If {
            condition,
            if_true,
            if_false,
        } => {
            let condition = eval(&condition, vars)?;
            if let Value::Bool(b) = condition.clone().as_ref() {
                if *b {
                    Ok(eval(&if_true, vars)?)
                } else if let Some(if_false_expr) = if_false {
                    Ok(eval(&if_false_expr, vars)?)
                } else {
                    Ok(Rc::new(Value::Nothing))
                }
            } else {
                Err(new_error(format!(
                    "if condition must evaluate to bool, got {}",
                    condition.type_name()
                )))
            }
        }
        Expression::While {
            condition,
            body,
            if_completed: _, // TBD
        } => {
            let mut last_result = Rc::new(Value::Nothing);
            loop {
                let condition = eval(&condition, vars)?;
                if let Value::Bool(run_loop_iteration) = condition.clone().as_ref() {
                    if *run_loop_iteration {
                        last_result = eval(&body, vars)?;
                        if let Value::Returned(_) = last_result.clone().as_ref() {
                            return Ok(last_result);
                        }
                    } else {
                        return Ok(last_result);
                    }
                } else {
                    return Err(new_error(format!(
                        "while loop condition must evaluate to bool, got {}",
                        condition.type_name()
                    )));
                }
            }
        }
    }
}

pub fn eval_assignment(
    left: &Expression,
    right: &Expression,
    vars: &mut HashMap<String, Rc<Value>>,
) -> Result<Rc<Value>, String> {
    if let Expression::Variable(var_name) = left {
        let right_value = eval(right, vars).map_err(|e| e.errmsg)?;
        vars.insert(var_name.clone(), right_value.clone());
        Ok(right_value)
    } else if let Expression::BinaryOperation {
        op: op_left,
        left: ll,
        right: rl,
    } = left
    {
        if let Expression::BinaryOperation {
            op: op_right,
            left: lr,
            right: rr,
        } = right
        {
            if op_left != op_right {
                return Err(format!("right-hand side of the assignment doesn't match the pattern, expected binary operation {:?}", op_left));
            }
            let res_left = eval_assignment(ll, lr, vars)?;
            let res_right = eval_assignment(rl, rr, vars)?;
            return eval(
                &Expression::BinaryOperation {
                    op: *op_left,
                    left: Box::new(Expression::Value(res_left)),
                    right: Box::new(Expression::Value(res_right)),
                },
                vars,
            )
            .map_err(|e| e.errmsg);
        } else {
            Err(
                "right-hand side of the assignment doesn't match the pattern, expected binary operation".into(),
            )
        }
    } else if let Expression::UnaryOperation {
        op: op_left,
        operand: operand_left,
    } = left
    {
        if let Expression::UnaryOperation {
            op: op_right,
            operand: operand_right,
        } = right
        {
            if op_left != op_right {
                return Err(format!("right-hand side of the assignment doesn't match the pattern, expected unary operation {:?}", op_left));
            }
            let res_operand = eval_assignment(&operand_left, &operand_right, vars)?;
            return eval(
                &Expression::UnaryOperation {
                    op: *op_left,
                    operand: Box::new(Expression::Value(res_operand)),
                },
                vars,
            )
            .map_err(|e| e.errmsg);
        } else {
            Err(
                "right-hand side of the assignment doesn't match the pattern, expected unary operation".into(),
            )
        }
    } else {
        Err("assignment is only possible to a variable or a simple expression".into())
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
        (Value::String(s), Value::Int(i)) => Some(Value::String(s.repeat(*i as usize))),
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
    #[case("()", Value::Nothing)]
    #[case("();", Value::Nothing)]
    #[case("{};", Value::Nothing)]
    #[case("a = {};a", Value::Nothing)]
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
    #[case("return return 1", Value::Returned(Rc::new(Value::Int(1))))]
    #[case("if !(1 == 2) {return 1}; return 2", Value::Int(1))]
    #[case("if (1 == 2) {return 1}; return 2", Value::Int(2))]
    #[case("if (1 == 2) {return 1}; 2;", Value::Int(2))]
    #[case("while (1 == 2) {};", Value::Nothing)]
    #[case(
        "a = 1; while true { str = \"b\" * a; a = a + 1; print(str); if length(str) > 5 { return str; } };",
        Value::String("bbbbbb".into())
    )]
    #[case("func foo(a) a; foo(1)", Value::Int(1))]
    #[case("func foo(a) a + 1; foo(0)", Value::Int(1))]
    #[case(
        "func fib(n) if (n < 3) 1 else fib(n - 1) + fib(n - 2); fib(12)",
        Value::Int(144)
    )]
    #[case(
        "func fib(n); {if (n < 3) {return 1} else {return fib(n - 1) + fib(n - 2)}}; fib(12)",
        Value::Int(144)
    )]
    #[case(
        "func fib(n); {if (n < 3) {return 1;} else {return fib(n - 1) + fib(n - 2);}}; fib(12)",
        Value::Int(144)
    )]
    #[case("1, 2", Value::Tuple(vec![Rc::new(Value::Int(1)), Rc::new(Value::Int(2))]))]
    #[case("(1, 2)", Value::Tuple(vec![Rc::new(Value::Int(1)), Rc::new(Value::Int(2))]))]
    #[case("(1, 2, 3)", Value::Tuple(vec![
        Rc::new(Value::Int(1)),
        Rc::new(Value::Int(2)),
        Rc::new(Value::Int(3)),
    ]))]
    #[case("((1, 2), 3)", Value::Tuple(vec![
        Rc::new(Value::Tuple(vec![
            Rc::new(Value::Int(1)),
            Rc::new(Value::Int(2)),
        ])),
        Rc::new(Value::Int(3)),
    ]))]
    #[case("a, b = 1, 2", Value::Tuple(vec![Rc::new(Value::Int(1)), Rc::new(Value::Int(2))]))]
    #[case("-b = -1; b", Value::Int(1))]
    #[case("a = -b = -1; a", Value::Int(-1))]
    #[case("tup = a, b = 1, 2; tup", Value::Tuple(vec![Rc::new(Value::Int(1)), Rc::new(Value::Int(2))]))]
    #[case("a, b = 1, 2; a + b", Value::Int(3))]
    #[case("a, (b, c) = 1, (2, 3); a + b + c", Value::Int(6))]
    #[case("sum = a + b = 3 + 7; a", Value::Int(3))]
    #[case("func add(a, b) a + b; add(1, 2)", Value::Int(3))]
    #[case("func add(a, (b + c)) a + b + c; add(1, (2 + 3))", Value::Int(6))]
    fn test_runtime_basic(#[case] code: &str, #[case] expected_result: Value) {
        let code_ = String::from(code);
        let tokens = tokenize(&code_).unwrap();
        let ast = parse(&tokens).unwrap();
        let result = eval(&ast, &mut HashMap::new());
        assert_eq!(result.unwrap().as_ref().to_owned(), expected_result);
    }
}

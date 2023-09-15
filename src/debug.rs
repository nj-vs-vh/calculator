use crate::{
    parser::Expression,
    values::{function::Function, Value},
};

pub fn print_tree(expr: &Expression) {
    println!("{}", format_tree(expr));
}

fn format_tree(expr: &Expression) -> String {
    match expr {
        Expression::Value(bv) => {
            let bv_clone = bv.clone();
            let v = bv_clone.as_ref();
            match v {
                Value::Function(Function::UserDefined(func)) => format_subexpressions(
                    &format!("Function {}({})", func.name, func.arg_name),
                    [&func.body].iter().map(|&e| e),
                    1,
                ),
                _ => format!("{:?}", v),
            }
        }
        Expression::Variable(name) => format!("{}", name),
        Expression::BinaryOperation { op, left, right } => format_subexpressions(
            &format!("{:?}", op),
            [left, right].iter().map(|&e| e.as_ref()),
            2,
        ),
        Expression::UnaryOperation { op, operand } => format_subexpressions(
            &format!("{:?}", op),
            [operand].iter().map(|&e| e.as_ref()),
            1,
        ),
        Expression::Scope {
            body,
            is_returnable: _,
        } => format_subexpressions("┬── scope ───", body.iter(), body.len()),
        Expression::If {
            condition,
            if_true,
            if_false,
        } => {
            if let Some(if_false) = if_false {
                format_subexpressions(
                    &format!("IfElse"),
                    [condition, if_true, if_false].iter().map(|&e| e.as_ref()),
                    3,
                )
            } else {
                format_subexpressions(
                    &format!("If"),
                    [condition, if_true].iter().map(|&e| e.as_ref()),
                    2,
                )
            }
        }
        Expression::While {
            condition,
            body,
            if_completed: _,
        } => format_subexpressions(
            &format!("While"),
            [condition, body].iter().map(|&e| e.as_ref()),
            2,
        ),
    }
}

fn format_subexpressions<'a>(
    title: &str,
    subexpr_iter: impl Iterator<Item = &'a Expression>,
    subexpr_count: usize,
) -> String {
    let mut res: String = title.into();
    res.push('\n');
    for (idx, expr) in subexpr_iter.enumerate() {
        let is_last_subexpr = idx >= subexpr_count - 1;
        let (pre_first, pre_other) = if !is_last_subexpr {
            ("├─", "│ ")
        } else {
            ("└─", "  ")
        };
        let expr_tree = format_tree(expr);
        let expr_tree_lines: Vec<&str> = expr_tree.lines().collect();
        for (line_idx, &line) in expr_tree_lines.iter().enumerate() {
            res.push_str(if line_idx == 0 { pre_first } else { pre_other });
            res.push_str(line);
            if !is_last_subexpr || line_idx < expr_tree_lines.len() - 1 {
                res.push('\n');
            }
        }
    }
    res
}

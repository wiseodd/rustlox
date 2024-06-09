use crate::{expr::Expr, token::Literal};

pub fn print(expr: Expr) -> String {
    visit_expr(&expr)
}

fn visit_expr(expr: &Expr) -> String {
    match expr {
        // Base case
        Expr::Literal { value } => match value {
            Literal::None => "nil".to_string(),
            Literal::String(val) => val.to_string(),
            Literal::Boolean(val) => val.to_string(),
            Literal::Number(val) => val.to_string(),
        },
        // Recursion
        Expr::Binary {
            left,
            operator,
            right,
        } => parenthesize(&operator.lexeme, &[left, right]),
        Expr::Grouping { expression } => parenthesize("group", &[expression]),
        Expr::Unary { operator, right } => parenthesize(&operator.lexeme, &[right]),
        _ => "".to_string(),
    }
}

fn parenthesize(name: &str, exprs: &[&Expr]) -> String {
    let mut res = String::new();

    res.push('(');
    res.push_str(name);

    for expr in exprs {
        res.push(' ');
        res.push_str(&visit_expr(expr));
    }

    res.push(')');

    res
}

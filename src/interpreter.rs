use crate::{
    expr::Expr,
    token::{Literal, TokenType},
};

fn visit_expr(expr: &Expr) -> Literal {
    match expr {
        Expr::Literal { value } => value.clone(),
        Expr::Grouping { expression } => visit_expr(expression),
        Expr::Unary { operator, right } => {
            let right: Literal = visit_expr(right);

            match operator.token_type {
                TokenType::Bang => match right {
                    Literal::Boolean(value) => Literal::Boolean(!value),
                    _ => unreachable!(),
                },
                TokenType::Minus => match right {
                    Literal::Number(value) => Literal::Number(-value),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    };

    unreachable!()
}

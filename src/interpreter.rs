use core::panic;

use crate::{
    expr::Expr,
    token::{Literal, TokenType},
};

fn visit_expr(expr: &Expr) -> Literal {
    match expr {
        Expr::Literal { value } => value.clone(),
        Expr::Grouping { expression } => visit_expr(expression),
        Expr::Unary { operator, right } => {
            // Recursion to get the leaf (always a literal)
            let right: Literal = visit_expr(right);

            // Apply the unary operator
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
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            // DFS
            let left: Literal = visit_expr(left);
            let right: Literal = visit_expr(right);

            match operator.token_type {
                TokenType::Minus => match (left, right) {
                    (Literal::Number(val1), Literal::Number(val2)) => Literal::Number(val1 - val2),
                    _ => panic!(),
                },
                TokenType::Slash => match (left, right) {
                    (Literal::Number(val1), Literal::Number(val2)) => Literal::Number(val1 / val2),
                    _ => panic!(),
                },
                TokenType::Plus => match (left, right) {
                    (Literal::Number(val1), Literal::Number(val2)) => Literal::Number(val1 + val2),
                    (Literal::String(val1), Literal::String(val2)) => {
                        let mut res: String = val1.clone();
                        res.push_str(&val2);
                        Literal::String(res)
                    }
                    _ => panic!(),
                },
                TokenType::Greater => match (left, right) {
                    (Literal::Number(val1), Literal::Number(val2)) => Literal::Boolean(val1 > val2),
                    _ => panic!(),
                },
                TokenType::GreaterEqual => match (left, right) {
                    (Literal::Number(val1), Literal::Number(val2)) => {
                        Literal::Boolean(val1 >= val2)
                    }
                    _ => panic!(),
                },
                TokenType::Less => match (left, right) {
                    (Literal::Number(val1), Literal::Number(val2)) => Literal::Boolean(val1 < val2),
                    _ => panic!(),
                },
                TokenType::LessEqual => match (left, right) {
                    (Literal::Number(val1), Literal::Number(val2)) => {
                        Literal::Boolean(val1 <= val2)
                    }
                    _ => panic!(),
                },
                TokenType::BangEqual => Literal::Boolean(!is_equal(left, right)),
                TokenType::EqualEqual => Literal::Boolean(is_equal(left, right)),
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    };

    unreachable!()
}

fn is_equal(a: Literal, b: Literal) -> bool {
    match (a, b) {
        (Literal::None, Literal::None) => true,
        (Literal::None, _) => false,
        (_, Literal::None) => false,
        (Literal::Number(val1), Literal::Number(val2)) => val1 == val2,
        (Literal::String(val1), Literal::String(val2)) => val1 == val2,
        _ => false,
    }
}

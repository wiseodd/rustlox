use core::panic;
use std::string::ParseError;

use anyhow::{anyhow, Result};

use crate::{
    expr::Expr,
    lox::Lox,
    token::{Literal, Token, TokenType},
};

pub enum LoxError {
    ParseError,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    // expression -> equality ;
    fn expression(&mut self) -> Expr {
        self.equality()
    }

    // comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Expr {
        let mut expr: Expr = self.comparison();

        while self.is_match_advance(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator: Token = match self.previous() {
                Some(token) => token.clone(),
                None => panic!(),
            };
            let right: Expr = self.comparison();

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    // term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Expr {
        let mut expr: Expr = self.term();

        while self.is_match_advance(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator: Token = match self.previous() {
                Some(token) => token.clone(),
                None => panic!(),
            };
            let right: Expr = self.term();

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    // factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Expr {
        let mut expr: Expr = self.factor();

        while self.is_match_advance(&[TokenType::Minus, TokenType::Plus]) {
            let operator: Token = match self.previous() {
                Some(token) => token.clone(),
                None => panic!(),
            };
            let right: Expr = self.factor();

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    // unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Expr {
        let mut expr: Expr = self.unary();

        while self.is_match_advance(&[TokenType::Slash, TokenType::Star]) {
            let operator: Token = match self.previous() {
                Some(token) => token.clone(),
                None => panic!(),
            };
            let right: Expr = self.unary();

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    // ( ( "!" | "-" ) unary ) | primary ;
    fn unary(&mut self) -> Expr {
        if self.is_match_advance(&[TokenType::Bang, TokenType::Minus]) {
            let operator: Token = match self.previous() {
                Some(token) => token.clone(),
                None => panic!(),
            };
            let expr: Expr = self.unary();

            return Expr::Unary {
                operator,
                right: Box::new(expr),
            };
        }

        self.primary()
    }

    // primary -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
    fn primary(&mut self) -> Expr {
        if self.is_match_advance(&[TokenType::Number, TokenType::String]) {
            return Expr::Literal {
                value: match self.previous() {
                    Some(token) => token.literal.clone(),
                    None => panic!(),
                },
            };
        }

        if self.is_match_advance(&[TokenType::True]) {
            return Expr::Literal {
                value: Literal::Boolean(true),
            };
        }

        if self.is_match_advance(&[TokenType::False]) {
            return Expr::Literal {
                value: Literal::Boolean(false),
            };
        }

        if self.is_match_advance(&[TokenType::Nil]) {
            return Expr::Literal {
                value: Literal::None,
            };
        }

        if self.is_match_advance(&[TokenType::LeftParen]) {
            let expr: Expr = self.expression();
            self.consume(TokenType::RightParen, "Expect ')' after expression.");
            return Expr::Grouping {
                expression: Box::new(expr),
            };
        }

        unreachable!()
    }

    // ------------------------ Utility functions ---------------------------
    // ----------------------------------------------------------------------

    fn is_match_advance(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        match self.peek() {
            Some(token) => token.token_type == *token_type,
            None => false,
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        match self.peek() {
            Some(token) => token.token_type == TokenType::Eof,
            None => true,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.current - 1)
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, LoxError> {
        if self.check(&token_type) {
            match self.advance() {
                Some(token) => Ok(token.clone()),
                None => Err(LoxError::ParseError),
            };
        }

        return Self::error(
            match self.peek() {
                Some(token) => token,
                None => return Err(LoxError::ParseError),
            },
            message,
        );
    }

    fn error(token: &Token, message: &str) -> Result<Token, LoxError> {
        Lox::parse_error(token, message);
        Err(LoxError::ParseError)
    }
}

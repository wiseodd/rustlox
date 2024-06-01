use anyhow::Result;

use crate::{
    error::ParseError,
    expr::Expr,
    lox::Lox,
    stmt::Stmt,
    token::{Literal, Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    // program -> statement* EOF ;
    pub fn parse(&mut self) -> Vec<Option<Stmt>> {
        let mut statements: Vec<Option<Stmt>> = vec![];

        while !self.is_at_end() {
            statements.push(self.declaration());
        }

        statements
    }

    // declaration -> varDecl | statement ;
    fn declaration(&mut self) -> Option<Stmt> {
        if self.is_match_advance(&[TokenType::Var]) {
            return match self.var_declaration() {
                Ok(stmt) => Some(stmt),
                Err(_) => {
                    self.synchronize();
                    None
                }
            };
        }

        match self.statement() {
            Ok(some_stmt) => some_stmt,
            Err(_) => {
                self.synchronize();
                None
            }
        }
    }

    // varDecl -> "var" IDENTIFIER ( "=" expression )? ";" ;
    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name: Token = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let initializer: Option<Expr> = if self.is_match_advance(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Stmt::Var { name, initializer })
    }

    // statement -> exprStmt | ifStmt | printStmt | block ;
    fn statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        if self.is_match_advance(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.is_match_advance(&[TokenType::Print]) {
            return self.print_statement();
        }

        if self.is_match_advance(&[TokenType::LeftBrace]) {
            return Ok(Some(Stmt::Block {
                statements: Box::new(self.block()?),
            }));
        }

        self.expression_statement()
    }

    // exprStmt -> expression ";" ;
    fn expression_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        let expr: Expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Some(Stmt::Expression { expression: expr }))
    }

    // ifStmt -> "if" "(" expression ")" statement
    //           ( "else" statement )? ;
    fn if_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        let _ = self.consume(TokenType::LeftParen, "Expect '(' after 'if' .");
        let condition: Expr = self.expression()?;
        let _ = self.consume(TokenType::RightParen, "Expect ')' after if condition.");

        let then_branch: Stmt = self.statement()?.unwrap();
        let else_branch: Option<Stmt> = match self.is_match_advance(&[TokenType::Else]) {
            true => self.statement()?,
            false => None,
        };

        Ok(Some(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        }))
    }

    // printStmt -> "print" expression ";" ;
    fn print_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        let expr: Expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Some(Stmt::Print { expression: expr }))
    }

    // block -> "{" declaration* "}" ;
    fn block(&mut self) -> Result<Vec<Option<Stmt>>, ParseError> {
        let mut statements: Vec<Option<Stmt>> = vec![];

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration());
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    // expression -> assignment ;
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    // assignment -> IDENTIFIER "=" assignment | equality ;
    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr: Expr = self.equality()?;

        if self.is_match_advance(&[TokenType::Equal]) {
            let equals: Token = self.previous().to_owned();
            let value: Expr = self.assignment()?;

            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            };

            return Err(Self::error(&equals, "Invalid assignment target."));
        }

        Ok(expr)
    }

    // comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr: Expr = self.comparison()?;

        while self.is_match_advance(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.comparison()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr: Expr = self.term()?;

        while self.is_match_advance(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.term()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr: Expr = self.factor()?;

        while self.is_match_advance(&[TokenType::Minus, TokenType::Plus]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.factor()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr: Expr = self.unary()?;

        while self.is_match_advance(&[TokenType::Slash, TokenType::Star]) {
            let operator: Token = self.previous().clone();
            let right: Expr = self.unary()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // ( ( "!" | "-" ) unary ) | primary ;
    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.is_match_advance(&[TokenType::Bang, TokenType::Minus]) {
            let operator: Token = self.previous().clone();
            let expr: Expr = self.unary()?;

            return Ok(Expr::Unary {
                operator,
                right: Box::new(expr),
            });
        }

        self.primary()
    }

    // primary -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.is_match_advance(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal {
                value: self.previous().literal.clone(),
            });
        }

        if self.is_match_advance(&[TokenType::True]) {
            return Ok(Expr::Literal {
                value: Literal::Boolean(true),
            });
        }

        if self.is_match_advance(&[TokenType::False]) {
            return Ok(Expr::Literal {
                value: Literal::Boolean(false),
            });
        }

        if self.is_match_advance(&[TokenType::Nil]) {
            return Ok(Expr::Literal {
                value: Literal::None,
            });
        }

        if self.is_match_advance(&[TokenType::LeftParen]) {
            let expr: Expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        if self.is_match_advance(&[TokenType::Identifier]) {
            return Ok(Expr::Variable {
                name: self.previous().to_owned(),
            });
        }

        Err(Self::error(self.peek(), "Expect expression."))
    }

    // ------------------------------ Utility functions --------------------------------
    // ---------------------------------------------------------------------------------

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

        self.peek().token_type == *token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(&token_type) {
            return Ok(self.advance().clone());
        }

        Err(Self::error(self.peek(), message))
    }

    fn error(token: &Token, message: &str) -> ParseError {
        Lox::parse_error(token, message);
        ParseError {}
    }

    fn synchronize(&mut self) {
        // Consume everything until the end of the statement.
        // At the end, `self.current` is at the beginning of a new statement,
        // and we can continue parsing.
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::For
                | TokenType::Fn
                | TokenType::If
                | TokenType::Print
                | TokenType::Return
                | TokenType::Var
                | TokenType::While => return,
                _ => (),
            }

            self.advance();
        }
    }
}

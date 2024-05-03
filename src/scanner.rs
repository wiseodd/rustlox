use crate::{
    lox::Lox,
    token::{Literal, Token, TokenType},
};

pub struct Scanner {
    source: String,
    pub tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_single_token();
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            "".to_string(),
            Literal::None,
            self.line,
        ));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn add_token_no_lit(&mut self, token_type: TokenType) {
        self.add_token(token_type, Literal::None)
    }

    fn add_token(&mut self, token_type: TokenType, literal: Literal) {
        let lexeme: &str = &self.source[self.start..self.current];
        self.tokens.push(Token::new(
            token_type,
            lexeme.to_string(),
            literal,
            self.line,
        ))
    }

    fn scan_single_token(&mut self) {
        let next_char: char = self.advance();

        match next_char {
            '(' => self.add_token_no_lit(TokenType::LeftParen),
            ')' => self.add_token_no_lit(TokenType::RightParen),
            '{' => self.add_token_no_lit(TokenType::LeftBrace),
            '}' => self.add_token_no_lit(TokenType::RightBrace),
            ',' => self.add_token_no_lit(TokenType::Comma),
            '.' => self.add_token_no_lit(TokenType::Dot),
            '-' => self.add_token_no_lit(TokenType::Minus),
            '+' => self.add_token_no_lit(TokenType::Plus),
            ';' => self.add_token_no_lit(TokenType::Semicolon),
            '*' => self.add_token_no_lit(TokenType::Star),
            '!' => match self.matches('=') {
                true => self.add_token_no_lit(TokenType::BangEqual),
                false => self.add_token_no_lit(TokenType::Bang),
            },
            '=' => match self.matches('=') {
                true => self.add_token_no_lit(TokenType::EqualEqual),
                false => self.add_token_no_lit(TokenType::Equal),
            },
            '>' => match self.matches('=') {
                true => self.add_token_no_lit(TokenType::GreaterEqual),
                false => self.add_token_no_lit(TokenType::Greater),
            },
            '<' => match self.matches('=') {
                true => self.add_token_no_lit(TokenType::LessEqual),
                false => self.add_token_no_lit(TokenType::Less),
            },
            '/' => match self.matches('/') {
                // Read the whole comment line
                true => {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                }
                false => self.add_token_no_lit(TokenType::Slash),
            },
            ' ' | '\r' | '\t' => (), // Do nothing
            '\n' => {
                self.line += 1;
            }
            '"' => self.add_string(),
            _ => {
                if next_char.is_ascii_digit() {
                    self.add_number();
                } else {
                    println!("Unexpected character in line {}", self.line);
                }
            }
        };
    }

    fn advance(&mut self) -> char {
        let next_char: char = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        next_char
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.current >= self.source.len() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        match self.is_at_end() {
            true => '\0',
            false => self.source.chars().nth(self.current).unwrap(),
        }
    }

    fn peek_next(&self) -> char {
        match self.current + 1 >= self.source.len() {
            true => '\0',
            false => self.source.chars().nth(self.current + 1).unwrap(),
        }
    }

    fn add_string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            Lox::error(self.line, "Unterminated");
            return;
        }

        self.advance(); // Move cursor to the closing "

        // Trim the quotes, get the string itself
        let lit_val: &str = &self.source[(self.start + 1)..(self.current - 1)];
        self.add_token(TokenType::String, Literal::String(lit_val.to_string()));
    }

    fn add_number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let parsing_res = (&self.source[self.start..self.current]).parse::<f64>();
        match parsing_res {
            Ok(val) => self.add_token(TokenType::Number, Literal::Number(val)),
            Err(err) => println!("{err:?}"),
        }
    }
}

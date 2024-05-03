use crate::token::{Token, TokenType};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
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

            let token_type: Option<TokenType> = self.scan_single_token();

            if token_type.is_none() {
                continue;
            }

            let token_type: TokenType = token_type.unwrap();
            let literal: Option<String> = match token_type {
                TokenType::String => self.get_string().clone(),
                _ => None,
            };
            let text: &str = &self.source[self.start..self.current];

            self.tokens
                .push(Token::new(token_type, text.to_string(), literal, self.line));
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            "".to_string(),
            Option::None,
            self.line,
        ));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_single_token(&mut self) -> Option<TokenType> {
        let next_char: char = self.advance();

        let token_type: Option<TokenType> = match next_char {
            '(' => Some(TokenType::LeftParen),
            ')' => Some(TokenType::RightParen),
            '{' => Some(TokenType::LeftBrace),
            '}' => Some(TokenType::RightBrace),
            ',' => Some(TokenType::Comma),
            '.' => Some(TokenType::Dot),
            '-' => Some(TokenType::Minus),
            '+' => Some(TokenType::Plus),
            ';' => Some(TokenType::Semicolon),
            '*' => Some(TokenType::Star),
            '!' => match self.matches('=') {
                true => Some(TokenType::BangEqual),
                false => Some(TokenType::Bang),
            },
            '=' => match self.matches('=') {
                true => Some(TokenType::EqualEqual),
                false => Some(TokenType::Equal),
            },
            '>' => match self.matches('=') {
                true => Some(TokenType::GreaterEqual),
                false => Some(TokenType::Greater),
            },
            '<' => match self.matches('=') {
                true => Some(TokenType::LessEqual),
                false => Some(TokenType::Less),
            },
            '/' => match self.matches('/') {
                // Read the whole comment line
                true => {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    None
                }
                false => Some(TokenType::Slash),
            },
            ' ' | '\r' | '\t' => None,
            '\n' => {
                self.line += 1;
                None
            }
            '"' => Some(TokenType::String),
            _ => unreachable!(),
        };

        token_type
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

    fn get_string(&mut self) -> Option<String> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            println!("Unterminated string.");
            return None;
        }

        self.advance(); // Move cursor to the closing "

        // Trim the quotes, get the string itself
        let value: &str = &self.source[(self.start + 1)..(self.current - 1)];
        Some(value.to_string())
    }
}

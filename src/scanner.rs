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
        while self.current >= self.source.len() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            "".to_string(),
            Option::None,
            self.line,
        ));
    }

    fn scan_token(&mut self) {
        let c: char = self.advance();

        self.add_token(
            match c {
                '(' => TokenType::LeftParen,
                ')' => TokenType::RightParen,
                '{' => TokenType::LeftBrace,
                '}' => TokenType::RightBrace,
                ',' => TokenType::Comma,
                '.' => TokenType::Dot,
                '-' => TokenType::Minus,
                '+' => TokenType::Plus,
                ';' => TokenType::Semicolon,
                '*' => TokenType::Star,
                _ => unreachable!(),
            },
            Option::None,
        )
    }

    fn advance(&mut self) -> char {
        let next_char: char = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        next_char
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<String>) {
        let text: &str = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(token_type, text.to_string(), literal, self.line));
    }
}

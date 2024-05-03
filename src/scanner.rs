use crate::{
    lox::Lox,
    token::{Literal, Token, TokenType},
};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    in_comment_block: bool,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            in_comment_block: false,
        }
    }

    pub fn scan_tokens(&mut self) -> Option<&Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;

            if self.in_comment_block {
                // Consume block (possibly multi-line) comment
                while !self.is_at_end() {
                    let c = self.advance();

                    if c == '\n' {
                        self.line += 1;
                    } else if c == '*' && self.peek() == '/' {
                        self.in_comment_block = false;
                        break;
                    }
                }

                if self.in_comment_block {
                    // If after consuming everything above, we haven't found the closing "*/"
                    // Then we throw an error.
                    Lox::error(self.line, "Block comment never closed.");
                    return None;
                } else {
                    // The above iter stopped at the closing '*'.
                    // So, we consume the closing '\'.
                    self.advance();
                }
            }

            self.scan_single_token();
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            "".to_string(),
            Literal::None,
            self.line,
        ));

        Some(&self.tokens)
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
            '*' => {
                dbg!(next_char, self.current);
                if self.current == 1 && self.peek_prev() == '/' {
                    // Handle edge case where a comment block is at the
                    // very start of the file
                    self.in_comment_block = true;
                } else {
                    self.add_token_no_lit(TokenType::Star);
                }
            }
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
            '/' => {
                if self.peek() == '*' {
                    self.in_comment_block = true;
                } else if self.matches('/') {
                    // Consume the whole comment line
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token_no_lit(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => (), // Do nothing
            '\n' => {
                self.line += 1;
            }
            '"' => self.add_string(),
            'o' => {
                if self.matches('r') {
                    self.add_token_no_lit(TokenType::Or)
                }
            }
            _ => {
                if next_char.is_ascii_digit() {
                    self.add_number();
                } else if Scanner::is_alpha(next_char) {
                    self.add_identifier();
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
        match !self.is_at_end() {
            true => self.source.chars().nth(self.current).unwrap(),
            false => '\0',
        }
    }

    fn peek_next(&self) -> char {
        match self.current + 1 < self.source.len() {
            true => self.source.chars().nth(self.current + 1).unwrap(),
            false => '\0',
        }
    }

    fn peek_prev(&self) -> char {
        self.source.chars().nth(self.current - 1).unwrap()
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

        match (&self.source[self.start..self.current]).parse::<f64>() {
            Ok(val) => self.add_token(TokenType::Number, Literal::Number(val)),
            Err(err) => println!("{err:?}"),
        }
    }

    fn add_identifier(&mut self) {
        while Scanner::is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text: &str = &self.source[self.start..self.current];
        let token_type: TokenType = Scanner::text2token(text);

        self.add_token_no_lit(token_type);
    }

    fn is_alpha(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_alphanumeric(c: char) -> bool {
        c.is_ascii_digit() || Scanner::is_alpha(c)
    }

    fn text2token(text: &str) -> TokenType {
        match text {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fn" => TokenType::Fn,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        }
    }
}

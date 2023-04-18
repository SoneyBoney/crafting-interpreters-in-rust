use std::collections::HashMap;
use std::fmt;

use crate::lox;


pub struct Scanner<'a> {
    lox: &'a lox::Lox,
    source: Vec<char>,
    source_len: usize,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u32,
    keywords: HashMap<String, TokenType>,
}

impl<'a> Scanner<'a> {
    pub fn new(lox: &'a lox::Lox, source: &'a str) -> Self {
        Self {
            lox,
            source: source.chars().collect(),
            source_len: source.chars().count().try_into().unwrap(),
            tokens: Vec::<Token>::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords: HashMap::from(
                [
                    ("and", TokenType::AND),
                    ("class", TokenType::CLASS),
                    ("else", TokenType::ELSE),
                    ("false", TokenType::FALSE),
                    ("for", TokenType::FOR),
                    ("fun", TokenType::FUN),
                    ("if", TokenType::IF),
                    ("nil", TokenType::NIL),
                    ("or", TokenType::OR),
                    ("print", TokenType::PRINT),
                    ("return", TokenType::RETURN),
                    ("super", TokenType::SUPER),
                    ("this", TokenType::THIS),
                    ("true", TokenType::TRUE),
                    ("var", TokenType::VAR),
                    ("while", TokenType::WHILE),
                ]
                .map(|(k, v)| (String::from(k), v)),
            ),
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token {
            token_type: TokenType::EOF,
            lexeme: "".to_string(),
            literal: None,
            line: self.line,
        });
        &self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source_len
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LEFT_PAREN),
            ')' => self.add_token(TokenType::RIGHT_PAREN),
            '{' => self.add_token(TokenType::LEFT_BRACE),
            '}' => self.add_token(TokenType::RIGHT_BRACE),
            ',' => self.add_token(TokenType::COMMA),
            '.' => self.add_token(TokenType::DOT),
            '-' => self.add_token(TokenType::MINUS),
            '+' => self.add_token(TokenType::PLUS),
            ';' => self.add_token(TokenType::SEMICOLON),
            '*' => self.add_token(TokenType::STAR),
            '!' => {
                let token_type = if self.matches('=') {
                    TokenType::BANG_EQUAL
                } else {
                    TokenType::BANG
                };
                self.add_token(token_type)
            }
            '=' => {
                let token_type = if self.matches('=') {
                    TokenType::EQUAL_EQUAL
                } else {
                    TokenType::EQUAL
                };
                self.add_token(token_type)
            }
            '<' => {
                let token_type = if self.matches('=') {
                    TokenType::LESS_EQUAL
                } else {
                    TokenType::LESS
                };
                self.add_token(token_type)
            }
            '>' => {
                let token_type = if self.matches('=') {
                    TokenType::GREATER_EQUAL
                } else {
                    TokenType::GREATER
                };
                self.add_token(token_type)
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::SLASH);
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '"' => self.string(),
            _ => {
                if c.is_ascii_digit() {
                    self.number();
                } else if Scanner::is_alpha(c) {
                    self.identifier();
                } else {
                    self.lox.error(self.line, "Unexpected character.");
                }
            }
        }
    }

    fn advance(&mut self) -> char {
        let current_char = self.source[self.current];
        self.current += 1;
        current_char
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_literal(token_type, None);
    }

    fn add_token_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token {
            token_type: token_type,
            lexeme: text.into_iter().collect(),
            literal: literal,
            line: self.line,
        });
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() || (self.source[self.current] != expected) {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source_len {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            self.lox.error(self.line, "Unterminated string.");
        }
        self.advance(); // closing "
        let value = String::from_iter(&self.source[self.start + 1..self.current - 1]);
        self.add_token_literal(TokenType::STRING, Some(Literal::Str(value)));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let num: f64 = String::from_iter(&self.source[self.start..self.current])
            .parse()
            .unwrap();
        self.add_token_literal(TokenType::NUMBER, Some(Literal::Number(num)));
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() {
            self.advance();
        }

        let text: &String = &self.source[self.start..self.current].into_iter().collect();
        let token_type = *self.keywords.get(text).unwrap_or(&TokenType::IDENTIFIER);
        self.add_token(token_type);
    }

    fn is_alpha(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    fn is_alphanumeric(c: char) -> bool {
        Scanner::is_alpha(c) || c.is_ascii_digit()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(non_camel_case_types)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,

    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Str(String),
    Number(f64),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: u32,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} {} {:?}",
            self.token_type, self.lexeme, self.literal
        )
    }
}

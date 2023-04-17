use crate::expr;
use crate::scanner;
use crate::lox;

#[derive(Debug)]
enum ParseError {
    Error,
}

pub struct Parser<'a> {
    pub lox: &'a lox::Lox,
    pub tokens: &'a Vec<scanner::Token>,
    current: usize
}

impl<'a> Parser<'a> {
    pub fn new(lox: &'a lox::Lox, tokens: &'a Vec<scanner::Token>) -> Self {
        Self { lox, tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<expr::Expr> {
        match self.expression() {
            Ok(e) => Some(e),
            Err(_) => None
        }
    }

    fn matches(&mut self, token_types: &[scanner::TokenType]) -> bool {
        for token_type in token_types {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: scanner::TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> &scanner::Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == scanner::TokenType::EOF
    }

    fn peek(&self) -> &scanner::Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &scanner::Token {
        &self.tokens[self.current - 1]
    }

    fn consume(
        &mut self,
        token_type: scanner::TokenType,
        msg: &str,
    ) -> Result<&scanner::Token, ParseError> {
        if self.check(token_type) {
            return Ok(self.advance())
        }
        let tok = self.peek().clone();
        Err(self.error(tok, msg))
    }

    fn error(&mut self, token: scanner::Token, msg: &str) -> ParseError {
        self.lox.parse_error(token.clone(), msg);
        ParseError::Error
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == scanner::TokenType::SEMICOLON { return }
            match self.peek().token_type {
                scanner::TokenType::CLASS | 
                scanner::TokenType::FUN |
                scanner::TokenType::VAR |
                scanner::TokenType::FOR |
                scanner::TokenType::IF |
                scanner::TokenType::WHILE |
                scanner::TokenType::PRINT |
                scanner::TokenType::RETURN => return,
                _ => {}
            }
            self.advance();
        }
    }

    fn token_op_to_expr_binops(token_type: scanner::TokenType) -> expr::BinaryOp {
        match token_type {
            scanner::TokenType::BANG_EQUAL => expr::BinaryOp::BangEqual,
            scanner::TokenType::EQUAL_EQUAL => expr::BinaryOp::EqualEqual,
            scanner::TokenType::GREATER => expr::BinaryOp::Greater,
            scanner::TokenType::GREATER_EQUAL => expr::BinaryOp::GreaterEqual,
            scanner::TokenType::LESS => expr::BinaryOp::Less,
            scanner::TokenType::LESS_EQUAL => expr::BinaryOp::LessEqual,
            scanner::TokenType::SLASH => expr::BinaryOp::Slash,
            scanner::TokenType::STAR => expr::BinaryOp::Star,
            scanner::TokenType::MINUS => expr::BinaryOp::Minus,
            scanner::TokenType::PLUS => expr::BinaryOp::Plus,
            _ => panic!("wtf binary"),
        }
    }

    fn token_op_to_expr_unops(token_type: scanner::TokenType) -> expr::UnaryOp {
        match token_type {
            scanner::TokenType::BANG => expr::UnaryOp::Bang,
            scanner::TokenType::MINUS => expr::UnaryOp::Minus,
            _ => panic!("wtf unary"),
        }
    }

    fn expression(&mut self) -> Result<expr::Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<expr::Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.matches(&[
            scanner::TokenType::BANG_EQUAL,
            scanner::TokenType::EQUAL_EQUAL,
        ]) {
            let operator_token = self.previous();
            let operator = Parser::token_op_to_expr_binops(operator_token.token_type);
            let right = Box::new(self.comparison()?);
            expr = expr::Expr::Binary(Box::new(expr), operator, right);
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<expr::Expr, ParseError> {
        let mut expr = self.term()?;
        while self.matches(&[
            scanner::TokenType::GREATER,
            scanner::TokenType::GREATER_EQUAL,
            scanner::TokenType::LESS,
            scanner::TokenType::LESS_EQUAL,
        ]) {
            let operator_token = self.previous();
            let operator = Parser::token_op_to_expr_binops(operator_token.token_type);
            let right = Box::new(self.comparison()?);
            expr = expr::Expr::Binary(Box::new(expr), operator, right);
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<expr::Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.matches(&[scanner::TokenType::PLUS, scanner::TokenType::MINUS]) {
            let operator_token = self.previous();
            let operator = Parser::token_op_to_expr_binops(operator_token.token_type);
            let right = Box::new(self.factor()?);
            expr = expr::Expr::Binary(Box::new(expr), operator, right);
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<expr::Expr, ParseError> {
        let mut expr = self.unary()?;
        while self.matches(&[scanner::TokenType::SLASH, scanner::TokenType::STAR]) {
            let operator_token = self.previous();
            let operator = Parser::token_op_to_expr_binops(operator_token.token_type);
            let right = Box::new(self.unary()?);
            let left = Box::new(expr);
            expr = expr::Expr::Binary(left, operator, right);
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<expr::Expr, ParseError> {
        if self.matches(&[scanner::TokenType::BANG, scanner::TokenType::MINUS]) {
            let operator_token = self.previous();
            let operator = Parser::token_op_to_expr_unops(operator_token.token_type);
            let right = Box::new(self.unary()?);
            return Ok(expr::Expr::Unary(operator, right));
        }
        Ok(self.primary()?)
    }

    fn primary(&mut self) -> Result<expr::Expr, ParseError> {
        if self.matches(&[scanner::TokenType::FALSE]) {
            return Ok(expr::Expr::Literal(expr::Literal::False));
        } else if self.matches(&[scanner::TokenType::TRUE]) {
            return Ok(expr::Expr::Literal(expr::Literal::True));
        } else if self.matches(&[scanner::TokenType::NIL]) {
            return Ok(expr::Expr::Literal(expr::Literal::Nil));
        }

        if self.matches(&[scanner::TokenType::STRING]) {
            if let Some(scanner::Literal::Str(lit_string)) = &self.previous().literal {
                return Ok(expr::Expr::Literal(expr::Literal::String(
                    lit_string.clone(),
                )));
            } else {
                return Err(ParseError::Error);
            }
        } else if self.matches(&[scanner::TokenType::NUMBER]) {
            if let Some(scanner::Literal::Number(num)) = &self.previous().literal {
                return Ok(expr::Expr::Literal(expr::Literal::Number(*num)));
            } else {
                return Err(ParseError::Error);
            }
        }

        if self.matches(&[scanner::TokenType::LEFT_PAREN]) {
            let expr = Box::new(self.expression()?);
            self.consume(
                scanner::TokenType::RIGHT_PAREN,
                "Expect ')' after expression.",
            )?;
            return Ok(expr::Expr::Grouping(expr));
        }
        Err(self.error(self.peek().clone(), "Expect expression."))
    }
}

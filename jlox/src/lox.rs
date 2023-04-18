use std::env;
use std::fs;
use std::io;
use std::process;

use crate::scanner;
use crate::parser;

pub struct Lox {
}

impl Lox {
    pub fn new() -> Self {
        Self { }
    }

    pub fn main(&mut self) {
        let args: Vec<String> = env::args().collect::<Vec<String>>()[1..].to_vec();
        if args.len() > 1 {
            println!("Usage: jlox [script]");
            process::exit(64)
        } else if args.len() == 1 {
            self.run_file(&args[0]);
        } else {
            self.run_prompt();
        }
    }

    pub fn parse_error(&self, token: scanner::Token, msg: &str) {
        if token.token_type == scanner::TokenType::EOF {
            self.report(token.line, " at end", msg);
        } else {
            self.report(token.line, &format!(" at '{}'", token.lexeme), msg);
        }
    }

    pub fn error(&self, line: u32, message: &str) {
        self.report(line, "", message);
    }

    fn report(&self, line: u32, arg_where: &str, message: &str) {
        println!("[line {} ] Error{}: {}", line, arg_where, message);
    }

    fn run_prompt(&mut self) {
        println!("run prompt");
        loop {
            let mut buffer = String::new();
            let bytes = io::stdin().read_line(&mut buffer).unwrap();
            if bytes == 0 {
                break;
            }
            self.run(&buffer.trim_end().to_string());
        }
    }

    fn run_file(&mut self, source_file: &str) {
        println!("run file: {}", source_file);
        let contents = fs::read_to_string(source_file).expect("Cannot open file");
        self.run(&contents);
    }

    fn run(&mut self, source: &str) {
        println!("{}", source);
        let mut scanner = scanner::Scanner::new(&self, source);
        let tokens = scanner.scan_tokens();
        println!("{:#?}", tokens);
        let mut parser = parser::Parser::new(&self, &tokens);
        let expr = parser.parse().unwrap();
        
        println!("{:#?}", expr);
    }
}
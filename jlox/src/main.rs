

mod expr;
mod parser;
mod scanner;
mod lox;




fn main() {
    let mut lox = lox::Lox::new();
    lox.main();
}

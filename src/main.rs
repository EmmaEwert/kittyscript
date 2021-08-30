#![allow(unused_braces)]

extern crate inkwell;
extern crate plex;

use std::io::Read;

mod ast;
mod lexer;
mod parser;
mod compiler;

fn main() -> Result<(), i32> {
	let mut source = String::new();
	std::io::stdin().read_to_string(&mut source).unwrap();
	let lexer = lexer::Lexer { source: &source };
	let ast = match parser::parse(lexer) {
		Err((None, message)) => panic!("parser: end of file: {}", message),
		Err((Some(token), message)) => panic!("parser: unexpected token: {:?}, {}", token.0, message),
		Ok(expressions) => {
			expressions.iter().for_each(|expression| println!("parser: {:?}", expression));
			expressions
		}
	};
	match compiler::compile(ast) {
		Err(message) => { eprintln!("compiler: {}", message); Err(1) }
		Ok(_) => { println!("compiler: OK"); Ok(()) }
	}
}
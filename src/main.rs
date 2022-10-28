use std::env;
use std::fs;

mod parser;
mod utils;
use parser::parser::Token;

const DEFAULT_TEST_PATH: &str = "test-single.txt";

fn main() {
	let args: Vec<String> = env::args().collect();
	let (path, p) = match &args[..] {
		[_, path] => (&path[..], false),
		[_, path, print] => (&path[..], print == "print"),
		_ => (DEFAULT_TEST_PATH, false),
	};

	let rune = b'$';
	let content = fs::read_to_string(path).unwrap();
	let parsed = Token::parse(&rune, &content).unwrap();
	let print = format!("{}", parsed);

	println!("len: {}", parsed.tokens.len());
	if p {
		println!("tree\n-------------------\n{:#?}\n", parsed);
		println!("print\n-------------------\n{}", print);
	}
}

use std::env;
use std::fs;

mod parser;
mod utils;
use parser::Token;

const DEFAULT_TEST_PATH: &str = "test-single.txt";

fn main() {
	let string_args: Vec<String> = env::args().collect();
	let args: Vec<&str> = string_args.iter().map(|a| a.as_str()).collect();

	let (path, p, o) = match &args[..] {
		[_, path] => (&path[..], false, None),
		[_, path, print] => (&path[..], print == &"print", None),
		[_, path, "print", output] => match &output[..] {
			"tree" => (&path[..], true, Some("tree")),
			"html" => (&path[..], true, Some("html")),
			_ => (&path[..], true, None),
		},
		_ => (DEFAULT_TEST_PATH, false, None),
	};

	let content: String = fs::read_to_string(path).unwrap();
	let tree: Token = Token::new(&content);
	let html: String = format!("{}", tree);

	println!("len: {}", tree.tokens.len());
	println!("parsed len: {}", html.len());
	if p {
		match o {
			Some("tree") => println!("\ntree\n-------------------\n{:#?}\n", tree),
			Some("html") => println!("\nprint\n-------------------\n{}", html),
			None => println!("tree\n-------------------\n{:#?}\n", tree),
			_ => (),
		}
	}
}

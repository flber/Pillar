/*
use std::io;

mod parser;
mod utils;
use parser::Token;

fn main() {
	let handle = io::stdin().lock();
	match io::read_to_string(handle) {
		Ok(mut s) => {
			let tree: Token = Token::new(&mut s);
			let html: String = format!("{}", tree);

			println!("{}", html);
		}
		Err(e) => println!("failed to read from stdin: {}", e),
	}
}
*/

use std::env;
use std::fs;

mod parser;
mod utils;
use parser::Token;

const DEFAULT_TEST_PATH: &str = "test-100.txt";

fn main() {
	coz::thread_init();

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

	if p {
		match o {
			Some("tree") => println!("{:#?}\n", tree),
			Some("html") => println!("{}", html),
			None => println!("{:#?}\n", tree),
			_ => (),
		}
	} else {
		println!("len: {}", html.len());
	}
}

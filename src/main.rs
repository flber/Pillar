use std::io;

mod parser;
mod utils;
use parser::Token;

fn main() {
	let handle = io::stdin().lock();
	match io::read_to_string(handle) {
		Ok(s) => {
			let tree: Token = Token::new(&s);
			let html: String = format!("{}", tree);

			println!("{}", html);
		}
		Err(e) => println!("failed to read from stdin: {}", e),
	}
}

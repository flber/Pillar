pub mod parser {
	use std::ops::Range;

	// sorta a weird way of doing things, but to check if a byte is in RUNES use:
	// RUNES.bytes()
	pub const _RUNES: &str = "~!@#$%&*+=?>";

	/*
	pub struct Attr {
		name: String,
		val: String ,
	}
	*/

	/// Interface for a node on the abstract token tree to build a page.
	///
	/// attributes:
	/// 	rune: str in the RUNES vec denoting the associated element
	/// 	contents: format string to encode the non-token part of a token
	/// 	tokens: vec of child tokens
	#[derive(Debug, Default)]
	pub struct Token {
		pub rune: u8,
		pub contents: String,
		// pub attrs: Option<Vec<Attr>>,
		pub tokens: Vec<Token>,
	}

	impl Token {
		// pub fn new(s: &String) -> Token {}

		/// Helper function to parse a string and rune into a token tree.
		///
		/// Args:
		/// 	r: rune to convey the outer context
		/// 	s: interior of passed rune to be parsed into token
		///
		/// Returns:
		/// 	optional token representation of the inputed rune and string
		pub fn parse(_rune: &u8, content: &str) -> Option<Token> {
			let bytes = content.bytes();
			let mut t = Token::default();
			t.rune = *_rune;
			let mut t_bytes: Vec<u8> = vec![];

			let rune_char: u8 = b'$';
			let mut num_brack = 0;
			let mut range = Range { start: 0, end: 0 };

			for (i, c) in bytes.enumerate() {
				match c {
					b'{' => {
						if num_brack < 1 {
							t_bytes.push(c);
							range.start = i;
						}
						num_brack += 1;
					}
					b'}' => {
						num_brack -= 1;
						if num_brack < 1 {
							t_bytes.push(c);
							range.end = i;
							let new_token =
								Self::parse(&rune_char, &content[range.start + 1..range.end]);
							if let Some(nt) = new_token {
								t.tokens.push(nt);
							}
						}
					}
					_ => {
						if num_brack < 1 {
							t_bytes.push(c);
						}
					}
				}
			}

			t.contents = String::from_utf8_lossy(&t_bytes).to_string();

			Some(t)
		}
	}

	pub struct _Page {
		metadata: String,
		tokens: Token,
	}
}

/*
pub mod preproc {
	use crate::parser::page::*;

	fn parse(input: &String) -> Page {}
}
*/

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_basic() {
		let rune = b'$';
		let content = String::from("the {quick {brown}} fox {jumps}");
		let parsed = parser::Token::parse(&rune, &content).unwrap();
		println!("{}", content);
		println!("{:#?}", parsed);

		assert_eq!(2, parsed.tokens.len());
		assert_eq!(1, parsed.tokens[0].tokens.len());
	}

	#[test]
	fn test_parse_unicode() {
		let rune = b'$';
		let content = String::from("🚚🜗🦇 {🪟🗐💒🐞🎉 {🤳🢰🞿🤈🎽🖥🌁}} 🌖🵟🥖 {🧣👜🯹🖺🌗🯶🶰}");
		let parsed = parser::Token::parse(&rune, &content).unwrap();
		println!("{}", content);
		println!("{:#?}", parsed);

		assert_eq!(2, parsed.tokens.len());
		assert_eq!("🚚🜗🦇 {} 🌖🵟🥖 {}", parsed.contents);
		assert_eq!("🪟🗐💒🐞🎉 {}", parsed.tokens[0].contents);
		assert_eq!("🧣👜🯹🖺🌗🯶🶰", parsed.tokens[1].contents);
	}
}

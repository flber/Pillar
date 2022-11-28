pub mod parser {
	use crate::utils::*;
	use std::fmt;
	use std::ops::Range;

	// sorta a weird way of doing things, but to check if a byte is in RUNES use:
	// RUNES.bytes()
	pub const RUNES: &str = "~-!@#$%&*+=?>";

	/*
	pub struct Attr {
		name: String,
		val: String ,
	}
	*/

	/// Interface for a node on the abstract token tree to build a page.
	///
	/// attributes:
	///   rune: str in the RUNES vec denoting the associated element
	///   contents: format string to encode the non-token part of a token
	///   tokens: vec of child tokens
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
		///   r: rune to convey the outer context
		///   s: interior of passed rune to be parsed into token
		///
		/// Returns:
		///   optional token representation of the inputed rune and string
		pub fn parse(rune: &u8, content: &str) -> Option<Token> {
			let bytes: &[u8] = content.as_bytes();
			let mut t = Token::default();
			t.rune = *rune;
			let mut t_bytes: Vec<u8> = vec![];

			let mut rune_char: u8 = b'$';
			let mut num_brack = 0;
			let mut range = Range { start: 0, end: 0 };

			for (i, c) in bytes.iter().enumerate() {
				match c {
					b'{' => {
						if num_brack < 1 {
							if i > 0 && RUNES.as_bytes().contains(&bytes[i - 1]) {
								rune_char = bytes[i - 1];
								t_bytes.pop();
							} else {
								rune_char = b'$';
							}

							t_bytes.push(*c);
							range.start = i;
						}
						num_brack += 1;
					}
					b'|' => {
						if i > 0 && bytes[i - 1] == b'{' {
						} else {
							t_bytes.push(*c);
						}
					}
					b'}' => {
						num_brack -= 1;
						if num_brack < 1 {
							t_bytes.push(*c);
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
							t_bytes.push(*c);
						}
					}
				}
			}

			t.contents = String::from_utf8_lossy(&t_bytes).to_string();

			Some(t)
		}
	}

	impl fmt::Display for Token {
		fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
			let elem = match self.rune {
				b'~' => "ul",
				b'-' => "li",
				b'!' => "img",
				b'@' => "a",
				b'#' => "h1",
				b'$' => "",
				b'&' => "video",
				b'*' => "i",
				b'+' => "summary",
				b'=' => "pre",
				b'?' => "",
				b'>' => "blockquote",
				_ => "",
			};

			if self.tokens.is_empty() {
				// this is where the element formatting will go
				write!(f, "<{}>{}</{}>", elem, self.contents.clone(), elem)?;
			} else {
				let mut parsed_tokens: Vec<String> = vec![];
				for t in &self.tokens {
					parsed_tokens.push(t.to_string().to_owned());
				}
				let parsed_tokens = parsed_tokens.iter().map(|t| &t[..]).collect::<Vec<&str>>();
				let contents = self.contents.split("{}").collect::<Vec<&str>>();
				write!(
					f,
					"<{}>{}</{}>",
					elem,
					format::fast_zip(contents, parsed_tokens),
					elem
				)?;
			}
			Ok(())
		}
	}

	pub struct _Page {
		metadata: String,
		tokens: Token,
	}
}

#[cfg(test)]
mod test {
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

	#[test]
	fn test_display() {
		let rune = b'$';
		let content = String::from("the *{quick @{brown}} fox ={jumps}");
		let parsed = parser::Token::parse(&rune, &content).unwrap();
		let display = format!("{}", parsed);

		assert_eq!(
			"<>the <i>quick <a>brown</a></i> fox <pre>jumps</pre></>",
			display
		);
	}

	#[test]
	fn test_display_empty() {
		let rune = b'$';
		let content = String::from("the {quick {brown}} fox {jumps}");
		let parsed = parser::Token::parse(&rune, &content).unwrap();
		let display = format!("{}", parsed);

		assert_eq!("<>the <>quick <>brown</></> fox <>jumps</></>", display);
	}
}

use crate::murmur3;
use crate::utils::*;

use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

const RUNES: &str = ".~-!@#$%&*+=?>";
const RUNE_DEFAULT: u8 = b';';
const RUNE_EMPTY: u8 = b'?';

#[derive(Debug, Default)]
struct Token {
	pub id: u32,
	pub parent: Option<u32>,
	pub rune: u8,
	pub contents: String,
	pub children: Vec<u32>,
}

pub struct TokenMap(HashMap<u32, Token>);

impl TokenMap {
	pub fn from(s: &str) -> (u32, Self) {
		let mut map = TokenMap(HashMap::new());
		let (parsed, children) = TokenMap::parse(s);
		let hash: u32 = murmur3::hash(parsed.as_bytes());
		let mut token = Token {
			id: hash,
			parent: None,
			rune: RUNE_DEFAULT,
			contents: parsed,
			children: vec![],
		};

		for child in children {
			token.children.push(map.build(Some(hash), child.0, child.1));
		}

		map.insert(hash, token);
		(hash, map)
	}

	fn insert(&mut self, key: u32, value: Token) {
		self.0.insert(key, value);
	}

	pub fn get(&mut self, key: u32) -> &Token {
		self.0.get(&key).unwrap()
	}

	fn build(&mut self, parent: Option<u32>, rune: u8, s: &str) -> u32 {
		let (parsed, children) = TokenMap::parse(s);
		let hash: u32 = murmur3::hash(parsed.as_bytes());
		let mut token = Token {
			id: hash,
			parent,
			rune,
			contents: parsed,
			children: vec![],
		};

		for child in children {
			token
				.children
				.push(self.build(Some(hash), child.0, child.1));
		}

		self.insert(hash, token);
		hash
	}

	fn parse(content: &str) -> (String, Vec<(u8, &str)>) {
		let bytes: &[u8] = content.as_bytes();
		let mut parsed: Vec<u8> = vec![];

		let mut children: Vec<(u8, &str)> = vec![];

		let mut rune_char: u8 = RUNE_DEFAULT;
		let mut num_brack = 0;
		let mut range = Range { start: 0, end: 0 };

		for (i, c) in bytes.iter().enumerate() {
			match c {
				b'{' => {
					if num_brack < 1 {
						if i > 0 && RUNES.as_bytes().contains(&bytes[i - 1]) {
							rune_char = bytes[i - 1];
							parsed.pop();
						} else {
							rune_char = RUNE_DEFAULT;
						}

						parsed.push(*c);
						range.start = i;
					}
					num_brack += 1;
				}
				b'|' => {
					if i > 0 && bytes[i - 1] == b'{' {
						// attribute parsing will go here
					} else {
						parsed.push(*c);
					}
				}
				b'}' => {
					num_brack -= 1;
					if num_brack < 1 {
						parsed.push(*c);
						range.end = i;
						let child_token = (rune_char, &content[range.start + 1..range.end]);
						children.push(child_token);
					}
				}
				_ => {
					if num_brack < 1 {
						parsed.push(*c);
					}
				}
			}
		}
		(String::from_utf8_lossy(&parsed).to_string(), children)
	}

	pub fn print(&self, root: u32) -> String {
		let token = self.get(root);
		let mut output = String::new();
		
		let elem = match token.rune {
			b'.' => "p",
			b'~' => "ul",
			b'-' => "li",
			b'!' => "img",
			b'@' => "a",
			b'#' => "h1",
			b'&' => "video",
			b'*' => "i",
			b'+' => "summary",
			b'=' => "pre",
			b'>' => "blockquote",
			b'?' => "PARSE_ERROR", // evauluates to remove tag syntax when printed
			b';' => "",            // default, "contextual" rune
			b'$' => "",            // reserved "scripting" rune
			_ => "",
		};
		
		if token.children.is_empty() {
			// this is where the element formatting will go
			if token.rune != RUNE_EMPTY {
				output += &format!("<{}>{}</{}>", elem, token.contents.clone(), elem);
			}
		} else {
			let mut parsed_tokens: Vec<String> = vec![];
			for t in &token.children {
				parsed_tokens.push(self.print(*t));
			}
			let parsed_tokens = parsed_tokens.iter().map(|t| &t[..]).collect::<Vec<&str>>();
			let contents = token.contents.split("{}").collect::<Vec<&str>>();
			if token.rune != RUNE_EMPTY {
				output += &format!("<{}>{}</{}>",
					elem,
					format::fast_zip(contents, parsed_tokens),
					elem
				);
			} else {
				output += &format!("{}", format::fast_zip(contents, parsed_tokens));
			}
		}
		unimplemented!()
	}
}

#[cfg(test)]
mod test {
	use crate::parserhashed::*;

	#[test]
	fn test_parse_basic() {
		let content = String::from("the {quick {brown}} fox {jumps}");
		let (parsed, children) = TokenMap::parse(&content);

		assert_eq!(2, children.len());
		assert_eq!("the {} fox {}", parsed);
		assert_eq!("quick {brown}", children[0].1);
		assert_eq!("jumps", children[1].1);
	}

	#[test]
	fn test_build() {
		let content = String::from("the {quick {brown}} fox {jumps}");
		let (hash, mut map) = TokenMap::from(&content);
		let root = map.get(&hash);

		assert_eq!(root.contents, "the {} fox {}");

		let mut tree = root.contents.clone();
		tree.push_str("\n");
		let mut checking: Vec<u32> = vec![];
		for child in &root.children[..] {
			checking.push(*child);
		}

		for i in 0..checking.len() {
			let token = map.get(checking[i]);
			tree.push_str(&token.contents);
			tree.push_str("\n");
			checking.push(token.id);
		}

		assert_eq!("the {} fox {}\nquick {}\njumps\n", tree);
	}
}

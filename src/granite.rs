use crate::progress::*;
use crate::utils::text::*;
use std::fmt;
use std::str::FromStr;

pub struct Metadata {
	pub name: String,
	pub value: String,
}

#[derive(Debug)]
pub struct PageParseError;

impl fmt::Display for PageParseError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.write_str("invalid page format")
	}
}

pub struct Page {
	pub meta: Vec<Metadata>,
	pub content: String,
}

impl FromStr for Page {
	type Err = PageParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let raw = s.to_string();
		Ok(parse(&raw))
	}
}

impl Page {
	// returns a new Page struct using the parse_granite function
	// -> parse_granite(s: &String, debug: bool)
	pub fn new(s: &str) -> Self {
		let raw = s.to_string();
		parse(&raw)
	}
}

impl fmt::Display for Page {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.write_str(&self.content)
	}
}

/*
goes through the given String lines
figures out if it's in the meta heading
if it is, it starts generating a list of name: value pairs
if it isn't, it just adds the line to the output
it then returns a vec of Strings (the post), and a vec of Metadata (the name: value pairs)
*/
pub fn parse_header(l: &str) -> Page {
	// converts input string into Vec::<String>
	let split_content = l.lines();
	let str_lines: Vec<&str> = split_content.collect();
	let mut lines = Vec::<String>::new();
	for str_line in str_lines {
		let mut line = String::new();
		line.push_str(str_line);
		lines.push(line);
	}

	let mut meta = Vec::<Metadata>::new();
	let mut output = String::new();

	let mut in_reserved = false;
	for line in lines {
		// first returns (first non-whitespace character, index of that character)
		let first = first(&line).1;
		// anything longer than 6 characters isn't a "!meta!" tag anyway, so we check for that first
		if len(&line) >= first + 6 {
			// just some basic flag get/setting to tell if we need to start processing meta variables
			if slice(&line, first..first + 6) == "!meta!" && !in_reserved {
				in_reserved = true;
			} else if slice(&line, first..first + 6) == "!meta!" && in_reserved {
				in_reserved = false;
			} else if in_reserved {
				// split the (metadata variable) line by ":" and add the metadata pair to our meta vec
				if let Some(c_index) = line.find(':') {
					let mut name = slice(&line, 0..c_index);
					name = trim(&name, 0, 0).0;
					let mut value = slice(&line, c_index + 1..len(&line));
					value = trim(&value, 0, 0).0;
					meta.push(Metadata { name, value });
				}
			// if the line is >= 6 characters but isn't in a meta header, just push it to output
			} else {
				output.push_str(&line);
				output.push('\n')
			}
		// if the line is longer than 6 characters we don't need to deal with it, so it's just pushed to the output
		} else {
			output.push_str(&line);
			output.push('\n');
		}
	}
	// returns a metadata vec and the output (which is equal to the input, minus any metadata header)
	Page {
		meta,
		content: output,
	}
	// -> main.rs
}

// Preprocessing layer !!Not in use!!
/*
this is a paragraph
[ ul
  stuff 1
  stuff 1
]
*/

// Parsing layer
/*
[ p | this is a paragraph ]
[ ul |
  [ li | stuff 1 ]
  [ li | stuff 1 ]
]
*/

pub fn parse(s: &str) -> Page {
	// uses parse_header to return meta and content without header
	let header_parsed = parse_header(s);
	let meta = header_parsed.meta;
	let text = header_parsed.content;

	let post_process = text; // = pre_process(&text);

	// parses content
	// -> parse(s: &String, mut debug: bool)
	let content = parse_granite(&post_process);

	Page { meta, content }
}

// not implemented right now, still deciding on its usefulness
// fn _pre_process(s: &str) -> String {
// 	let t = s;
// 	let mut lines = t.lines();
// 	let mut output = Vec::<String>::new();
// 
// 	for _ in 0..lines.clone().count() {
// 		let line = lines.next().unwrap_or("");
// 		output.push(line);
// 	}
// 
// 	for i in 0..output.len() {
// 		// add `mut` if doing preprocessing
// 		let line = output[i].clone();
// 		/*
// 		if line != "" {
// 		  let first = first(&line.to_string()).0;
// 
// 		  if first != "[".to_string() && first != "]".to_string(){
// 			line = ["[ p |", &line, " ]"].concat();
// 		  }
// 		}
// 		*/
// 		output[i] = insert(&line, len(&line), "\n");
// 	}
// 
// 	output.concat()
// }

fn parse_granite(s: &String) -> String {
	let mut t = s.clone();
	let mut elems = Vec::<String>::new();
	let mut in_quotes = false;
	let mut in_content = false;
	let mut invalid_blocks = 0;

	// info for print bar
	let width = terminal_size().unwrap_or((100, 100)).1 as usize;
	let left_pad = width / 10;
	let bar_width = width / 2;
	let bar = Bar {
		left_pad,
		bar_width,
		max: len(s),
	};

	let mut i = 0;
	// let mut now = Instant::now();
	// goes through content string t character by character
	while i < len(&t) {
		// 4th test
		// small overhead to slice function
		let char = &slice(&t, i..i + 1)[..];
		// let char = "a";

		bar.print(i);

		// this... uh... sets the in_quotes and in_content variables?
		// 2nd test
		// /*
		match char {
			"\"" => {
				if in_quotes {
					in_quotes = false;
					// a bit scuffed, but it prevents mark [A] from deleting the character before the closing quote.
					i += 1;
					// this is a bad way of doing it, but otherwise if there's a quote just before a close bracket, it'll skip the close bracket
					let new_char = &slice(&t, i..i + 1)[..];
					if new_char == "]" {
						t = remove(&t, i, 1);
						let elem = match elems.pop() {
							Some(e) => e,
							None => String::from(""),
						};
						let end_tag = &format!("</{}>", elem);
						t = insert(&t, i, end_tag);
					}
				} else if !in_quotes {
					in_quotes = true;
				}
			}
			"[" => {
				if !in_quotes {
					// checks if an open bracket ends with a | or a ]. If the latter, the block is invalid and should not be parsed
					let mut j = i;
					let valid = loop {
						if j > len(&t) {
							break false;
						}
						let test_char = &slice(&t, j..j + 1)[..];
						match test_char {
							"|" => {
								break true;
							}
							"]" => {
								break false;
							}
							_ => (),
						}
						j += 1;
					};
					if valid {
						in_content = false;
					} else {
						invalid_blocks += 1;
					}
				}
			}
			"]" => {
				// replaces ] with proper tag, or ignores if it's an invalid block
				if !in_quotes && invalid_blocks < 1 {
					t = remove(&t, i, 1);
					let elem = match elems.pop() {
						Some(e) => e,
						None => String::from(""),
					};
					let end_tag = &format!("</{}>", elem);
					t = insert(&t, i, end_tag);
				}
				// allows for nesting of invalid blocks (i.e. `[ hi [parser]]`)
				if invalid_blocks > 0 {
					invalid_blocks -= 1;
				}
			}
			_ => (),
		}
		// this is where the sane formatting happens, once everything has been cleared by the above section
		if !in_quotes && !in_content {
			match char {
				"[" => {
					t = remove(&t, i, 1);
					t = insert(&t, i, "<");

					let next = first_from(&t, i + 1).1;
					t = remove(&t, i + 1, next);
					let mut j = i;
					// find the current element and adds it to the list for later closing
					let elem = slice(
						&t,
						i + 1..loop {
							let check = slice(&t, j..j + 1);
							if check == "," || check == " " || check == "\n" || check == "|" {
								break j;
							}
							j += 1;
						},
					);
					elems.push(elem);
				}
				"]" => {
					t = remove(&t, i, 1);
					let elem = match elems.pop() {
						Some(e) => e,
						None => String::from(""),
					};
					let end_tag = &format!("</{}>", elem);
					t = insert(&t, i, end_tag);
				}
				_ => {
					// lookahead code, mostly for pretty formatting (removing spaces, : to =, etc)
					let mut j = i;
					let next = loop {
						if j > len(&t) {
							break j;
						}
						let test_char = slice(&t, j..j + 1);
						if test_char != " " && test_char != "\t" {
							break j;
						}
						j += 1;
					};
					match &slice(&t, next..next + 1)[..] {
						"|" => {
							t = remove(&t, i + 1, next - i);
							t = insert(&t, i + 1, ">");
							t = remove(&t, i, 1);
							in_content = true;
						}
						":" => {
							t = remove(&t, i + 1, next - i);
							t = remove(&t, i, 1);
							t = insert(&t, i, "=");
							t = remove(&t, i + 1, next - i);
						}
						"," => {
							t = remove(&t, next, 1);
						}
						// gets rid of the previous space, but not the greatest way of doing things
						"\"" => {
							// [A]
							t = remove(&t, next - 1, 1);
							// counteracts the skipping effect of deleting the current char
							i -= 1;
						}
						_ => (),
					}
				}
			}
		}
		if i > len(&t) {
			break;
		}
		i += 1;
		// now = Instant::now();
	}
	t
}

pub mod progress {

	use std::io::{self, Write};

	pub struct Bar {
		pub left_pad: usize,
		pub bar_width: usize,
		pub max: usize,
	}

	impl Bar {
		pub fn print(&self, i: usize) {
			let inverse = 1.0 / (i as f32 / self.max as f32);
			let progress = (self.bar_width as f32 / inverse) as usize;
			let percent = if (100.0 / inverse).ceil() > 100.0 {
				100.0
			} else {
				(100.0 / inverse).ceil()
			};

			if self.bar_width >= progress {
				print!(
					"\r{:#left$}{} [{:=>mid$}{:->right$}",
					percent,
					" ",
					">",
					"]",
					left = self.left_pad,
					mid = progress,
					right = self.bar_width - progress
				);
			}
			io::stdout().flush().unwrap();
		}
	}

	pub fn terminal_size() -> Option<(u16, u16)> {
		use std::process::Command;
		use std::process::Stdio;

		let output = Command::new("stty")
			.arg("size")
			.arg("-F")
			.arg("/dev/stderr")
			.stderr(Stdio::inherit())
			.output()
			.unwrap();

		let stdout = String::from_utf8(output.stdout).unwrap();
		if !output.status.success() {
			return None;
		}

		// stdout is "rows cols"
		let mut data = stdout.split_whitespace();
		let rows = u16::from_str_radix(data.next().unwrap(), 10).unwrap();
		let cols = u16::from_str_radix(data.next().unwrap(), 10).unwrap();
		Some((rows, cols))
	}
}

pub mod granite {

	use crate::utils::progress::*;
	use crate::utils::text::*;
	// use std::cmp::Ordering;
	use std::fmt;
	use std::str::FromStr;
	// uncomment for debug output
	use std::io::stdin;
	// uncomment for delay in auto debug
	// use std::process::Command;
	// use regex::Regex;
	// use std::time::Instant;

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
			Ok(parse(&raw, false))
		}
	}

	impl Page {
		// returns a new Page struct using the parse_granite function
		// -> parse_granite(s: &String, debug: bool)
		pub fn new(s: &str, debug: bool) -> Self {
			let raw = s.to_string();
			parse(&raw, debug)
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

	pub fn parse(s: &str, debug: bool) -> Page {
		// uses parse_header to return meta and content without header
		let header_parsed = parse_header(s);
		let meta = header_parsed.meta;
		let text = header_parsed.content;

		let post_process = text; // = pre_process(&text);

		// parses content
		// -> parse(s: &String, mut debug: bool)
		let content = parse_granite(&post_process, debug);

		Page { meta, content }
	}

	// not implemented right now, still deciding on its usefulness
	fn _pre_process(s: &str) -> String {
		let t = s;
		let mut lines = t.lines();
		let mut output = Vec::<String>::new();

		for _ in 0..lines.clone().count() {
			let line = match lines.next() {
				Some(val) => val,
				None => "",
			}
			.to_string();
			output.push(line);
		}

		for i in 0..output.len() {
			// add `mut` if doing preprocessing
			let line = output[i].clone();
			/*
			if line != "" {
			  let first = first(&line.to_string()).0;

			  if first != "[".to_string() && first != "]".to_string(){
				line = ["[ p |", &line, " ]"].concat();
			  }
			}
			*/
			output[i] = insert(&line, len(&line), "\n");
		}

		output.concat()
	}

	fn debug_input(bar: &Bar, i: usize, mut debug: bool, mut auto: bool) -> (bool, bool) {
		let mut input_string = String::new();
		if !auto {
			loop {
				stdin()
					.read_line(&mut input_string)
					.expect("Failed to read line");
				if input_string == "\n" {
					break;
				} else if input_string == "next\n" {
					debug = false;
					break;
				} else if input_string == "auto\n" {
					auto = true;
					break;
				}
				bar.print(i);
			}
		}
		// let mut child = Command::new("sleep").arg("0.05").spawn().unwrap();
		// let _result = child.wait().unwrap();

		(debug, auto)
	}

	// prints debug information to terminal
	// t, elems, in_quotes, in_content, invalid_blocks, i, char, bar, now, debug, auto
	fn debug_output(
		t: &String,
		elems: Vec<String>,
		in_quotes: bool,
		in_content: bool,
		invalid_blocks: usize,
		i: usize,
		char: &str,
		bar: &Bar,
		/*now: Instant,*/
		mut debug: bool,
		mut auto: bool,
	) -> (bool, bool) {
		// Debugging output
		// clears screen for new output
		print!("{esc}c", esc = 27 as char);

		// organizes and prints current status of string
		let mut start = 0;
		let view = 500;
		if i > view {
			start = i - view;
		}
		let mut end = len(t);
		if len(t) >= view && i < len(t) - view {
			end = i + view;
		}
		println!(
			"...{}\x1b[31;1m@\x1b[0m{}...",
			slice(t, start..i),
			slice(t, i + 1..end)
		);
		// variable output
		println!("\n\n#################\n");
		println!("enter to continue, \"auto\" to speed up, \"next\" to skip");
		println!("elems: {:#?}", elems);
		println!("in_quotes: {}", in_quotes);
		println!("in_content: {}", in_content);
		println!("invalid_blocks: {}", invalid_blocks);
		if char != "\n" {
			println!("char: {}", char);
		// println!("Elapsed: {:.2?}", now.elapsed());
		} else {
			println!("char:");
		}

		// this just waits for user input
		let de_tuple: (bool, bool) = debug_input(bar, i, debug, auto);
		debug = de_tuple.0;
		auto = de_tuple.1;

		(debug, auto)
	}

	fn parse_granite(s: &String, mut debug: bool) -> String {
		let mut t = s.clone();
		let mut elems = Vec::<String>::new();
		let mut in_quotes = false;
		let mut in_content = false;
		let mut invalid_blocks = 0;

		// info for debug screen
		let width = terminal_size().unwrap_or((100, 100)).1 as usize;
		let left_pad = width / 10;
		let bar_width = width / 2;
		let bar = Bar {
			left_pad,
			bar_width,
			max: len(s),
		};

		let mut auto = false;
		let mut i = 0;
		// let mut now = Instant::now();
		// goes through content string t character by character
		while i < len(&t) {
			// 4th test
			// small overhead to slice function
			let char = &slice(&t, i..i + 1)[..];
			// let char = "a";

			if debug {
				match debug_output(
					&t,
					elems.clone(),
					in_quotes,
					in_content,
					invalid_blocks,
					i,
					char,
					&bar,
					/*now,*/
					debug,
					auto,
				) {
					(d, a) => {
						debug = d;
						auto = a;
					}
				}
			} else {
				bar.print(i);
			}

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
						()
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
}

pub mod text {

	use core::ops::Range;

	/*
	removes whitespace around given string from start and end indices
	*/
	pub fn trim(l: &String, start: usize, end: usize) -> (String, usize, usize) {
		let mut line = l.clone();
		let mut first: usize = 0;
		let mut last: usize = 0;
		let mut hit_text = false;
		for i in (0..len(&line) - end).rev() {
			let next = slice(&line, i..i + 1);
			if !hit_text && (next == " " || next == "\t") {
				line = remove(&line, i, 1);
			} else {
				first = i;
				hit_text = true;
			}
		}
		hit_text = false;
		let mut i = start;
		while i < len(&line) - end {
			let next = slice(&line, i..i + 1);
			if !hit_text && (next == " " || next == "\t") {
				line = remove(&line, i, 1);
			} else {
				hit_text = true;
				last = i;
				i += 1;
			}
		}
		(line, first, last)
	}

	/*
	replaces all target str in String with insert str
	*/
	pub fn replace(s: &String, target: &str, insert: &str) -> String {
		let mut source = s.clone();
		while let Some(i) = source.find(target) {
			source.replace_range(i..i + len(&target.to_string()), insert);
		}
		source
	}

	/*
	removes from String from index with length, preserving graphemes
	*/
	pub fn remove(s: &String, i: usize, l: usize) -> String {
		assert!(i <= len(s), "the index was larger than the target slice");

		let first = slice(s, 0..i);
		let second = slice(s, i + l..len(s));

		[first, second].concat()
	}

	/*
	returns the first character in a string, as well as the index of that string
	*/
	pub fn first(s: &String) -> (String, usize) {
		let line = s.clone();
		let mut num = 0;
		for i in 0..len(&line) {
			let char = slice(&line, i..i + 1);
			if char == " " || char == "\t" {
				num += 1;
			} else {
				return (char, num);
			}
		}
		(String::from(""), num)
	}

	/*
	returns the first character in a string from an index, as well as the index of that character
	*/
	pub fn first_from(s: &String, i: usize) -> (String, usize) {
		let line = s.clone();
		first(&slice(&line, i..len(&line)))
	}

	/*
	returns the length of a String, taking graphemes into account
	*/
	pub fn len(s: &String) -> usize {
		// let graphemes = UnicodeSegmentation::graphemes(&s[..], true).collect::<Vec<&str>>();
		// graphemes.len()
		s.chars().count()
	}

	/*
	inserts str into string, preserving graphemes
	*/
	pub fn insert(s: &String, idx: usize, ins: &str) -> String {
		assert!(idx <= len(&s), "the index was larger than the target slice");

		let mut r = String::with_capacity(s.len() + ins.len());
		let split_point = s.char_indices().nth(idx).map(|(i, _)| i).unwrap_or(s.len());

		let first_half = &s[..split_point];
		let second_half = &s[split_point..];
		r.push_str(first_half);
		r.push_str(ins);
		r.push_str(second_half);
		r
	}

	/*
	returns a slice of a string from a range, utf-8 compliant
	*/
	pub fn slice(s: &String, r: Range<usize>) -> String {
		let begin = s.char_indices().nth(r.start).map(|(i, _)| i).unwrap_or(0);
		let end = s
			.char_indices()
			.nth(r.end)
			.map(|(i, _)| i)
			.unwrap_or(s.len());
		s[begin..end].to_string()
	}
}

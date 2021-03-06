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
		
			if self.bar_width >= progress { print!("\r{:#left$}{} [{:=>mid$}{:->right$}", 
				(100.0/inverse).ceil(),
				" ", ">", "]", 
				left = self.left_pad, 
				mid = progress,
				right = self.bar_width - progress
			);}
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

    use crate::utils::text::*;
    use crate::utils::progress::*;
    // use std::cmp::Ordering;
    use std::fmt;
    use std::str::FromStr;
    // uncomment for debug output
    use std::io::stdin;
    // uncomment for delay in auto debug
    // use std::process::Command;

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
            Ok(parse_granite(&raw, false))
        }
    }

    impl Page {
    	pub fn new(s: &str, debug: bool) -> Self {
    		let raw = s.to_string();
    		parse_granite(&raw, debug)
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
    pub fn parse_header(l: &String) -> Page {
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
        for line_str in lines {
            let line = line_str.to_string();
            let first = first(&line).1;
            if len(&line) >= first + 6 {
                if slice(&line, first..first + 6) == "!meta!" && !in_reserved {
                    in_reserved = true;
                } else if slice(&line, first..first + 6) == "!meta!" && in_reserved {
                    in_reserved = false;
                } else if in_reserved {
                    if let Some(c_index) = line.find(':') {
                        let mut name = slice(&line, 0..c_index);
                        name = trim(&name, 0, 0).0;
                        let mut value = slice(&line, c_index + 1..len(&line));
                        value = trim(&value, 0, 0).0;
                        meta.push(Metadata { name, value });
                    }
                } else {
                    output.push_str(&line);
                    output.push('\n')
                }
            } else {
                output.push_str(&line);
                output.push('\n');
            }
        }
        Page {
            meta,
            content: output,
        }
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

    pub fn parse_granite(s: &String, debug: bool) -> Page {
        let meta = parse_header(&s).meta;
        let text = parse_header(&s).content;

        let post_process = pre_process(&text);

        let content = if debug { parse(&post_process, true) }
        else { parse(&post_process, false) };

        Page {
            meta,
            content,
        }
    }

	// not implemented right not, still deciding on its usefulness
    fn pre_process(s: &String) -> String {
    	let t = &s[..];
    	let mut lines = t.lines();
    	let mut output = Vec::<String>::new();

		for _ in 0..lines.clone().count() {
			let line = match lines.next() {
				Some(val) => val,
				None => "",
			}.to_string();
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
		
    	output.concat().to_string()
    }

    fn debug_input(bar: &Bar, i: usize, mut debug: bool, mut auto: bool) -> (bool, bool) {
	    let mut input_string = String::new();
	    if !auto { loop {
		    stdin().read_line(&mut input_string)
		    	.ok()
		        .expect("Failed to read line");
			if input_string == "\n" {
				break
			} else if input_string == "next\n" {
				debug = false;
				break
			} else if input_string == "auto\n" {
				auto = true;
				break
			}
		    bar.print(i);
	    }}
	    // let mut child = Command::new("sleep").arg("0.05").spawn().unwrap();
	    // let _result = child.wait().unwrap();

	    return (debug, auto)
    }

    fn parse(s: &String, mut debug: bool) -> String {		
    	let mut t = s.clone();
    	let mut elems = Vec::<String>::new();
    	let mut in_quotes = false;
    	let mut in_content = false;
    	let mut invalid_blocks = 0;

		let width = match terminal_size() {
			Some(s) => s,
			None => (100, 100),
		}.1 as usize;
		let left_pad = width / 10;
		let bar_width = width / 2;
    	
    	let bar = Bar { 
			left_pad, 
			bar_width, 
			max: len(&s), 
		};

		let mut auto = false;
		let mut i = 0;
    	while i < len(&t) {
    		let char = &slice(&t, i..i+1)[..];
    		if debug {
	    		// Debugging output
	    		// clears screen for new output
	    		print!("{esc}c", esc = 27 as char);

				// organizes and prints current status of string
	    		let mut start = 0;
	    		let view = 500;
	    		if i > view { start = i - view; }
	    		let mut end = len(&t);
	    		if i < len(&t)-view { end = i + view; }
	    		println!("...{}\x1b[31;1m@\x1b[0m{}...", slice(&t, start..i), slice(&t, i+1..end));
				// misc variable output
	    		println!("#################");
	    		println!("enter to continue, \"auto\" to speed up, \"next\" to skip");
	    		println!("elems: {:#?}", elems);
	    		println!("in_quotes: {}", in_quotes);
	    		println!("in_content: {}", in_content);
	    		println!("invalid_blocks: {}", invalid_blocks);
	    		if char != "\n" { println!("char: {}", char); }
	    		else { println!("char:"); }

				// this just waits for user input
				let de_tuple: (bool, bool) = debug_input(&bar, i, debug, auto);
				debug = de_tuple.0;
				auto = de_tuple.1;
		    } else {
				bar.print(i);
			}
			
			match char {
				"\"" => {
	  				if in_quotes {
	  					in_quotes = false;
	  					// a bit scuffed, but it prevents mark [A] from deleting the character before the closing quote.
	  					i += 1;
	  					// this is a bad way of doing it, but otherwise if there's a quote just before a close bracket, it'll skip the close bracket
	  					let new_char = &slice(&t, i..i+1)[..];
	  					match new_char {
							"]" => {
			    				t = remove(&t, i, 1);
			    				let elem = match elems.pop() {
			    					Some(e) => e,
			    					None => String::from(""),
			    				};
			    				let end_tag = &format!("</{}>", elem);
			    				t = insert(&t, i, end_tag);
							},
							_ => (),
	  					}
	  				} else if !in_quotes {
	  					in_quotes = true;
	  				}
				},
				"[" => {
					if !in_quotes{
						// checks if an open bracket ends with a | or a ]. If the latter, the block is invalid and should not be parsed
						let mut j = i;
						let valid = loop {
							if j > len(&t) { break false; }
							let test_char = &slice(&t, j..j+1)[..];
							match test_char {
								"|" => { break true; }
								"]" => { break false; }
								_ => (),
							}
							j += 1;
						};
						if valid { in_content = false; }
						else { invalid_blocks += 1; }
					}
				},
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
    				if invalid_blocks > 0 { invalid_blocks -= 1;}
				},
				_ => (),
			}

    		// this is where the sane formatting happens, once everything has been cleared by the above section
    		if !in_quotes && !in_content { match char {
    			"[" => {
    				t = remove(&t, i, 1);
    				t = insert(&t, i, "<");
    				
    				let next = first_from(&t, i+1).1;
    				t = remove(&t, i+1, next);
    				
    				let mut j = i;
    				// find the current element and adds it to the list for later closing
    				let elem = slice(&t, i+1..loop {
    					let check = slice(&t, j..j+1);
    					if check == "," || check == " " || check == "\n" || check == "|" {
    						break j;
    					}
    					j += 1;
    				});
    				elems.push(elem);
    			},
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
    					if j > len(&t) { break j; }
    					let test_char = slice(&t, j..j+1);
    					if test_char != " " && test_char != "\t" {
    						break j;
    					}
    					j += 1;
    				};
    				match &slice(&t, next..next+1)[..] {
    					"|" => {
    						t = remove(&t, i+1, next-i);
    						t = insert(&t, i+1, ">");
    						t = remove(&t, i, 1);
    						in_content = true;
    					},
    					":" => {
    						t = remove(&t, i+1, next-i);
    						t = remove(&t, i, 1);
    						t = insert(&t, i, "=");
    						t = remove(&t, i+1, next-i);
    					},
    					"," => {
    						t = remove(&t, next, 1);
    					},
    					// gets rid of the previous space, but not the greatest way of doing things
    					"\"" => {
    						// [A]
    						t = remove(&t, next-1, 1);
    						// counteracts the skipping effect of deleting the current char
    						i -= 1;
    					}
    					_ => (),
    				}
    			},
    		}}
    		if i > len(&t) {
    			break
    		}
    		i += 1;
    	}
    	return t;
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
    inserts str into string, preserving graphemes
    */
    pub fn insert(s: &String, idx: usize, ins: &str) -> String {
        assert!(idx <= len(&s), "the index was larger than the target slice");
        let ins_len = len(&ins.to_string());
        let fin_len = len(&s) + ins_len;
        let mut r = String::with_capacity(fin_len);
        for i in 0..fin_len {
            if i < idx {
                r.push_str(&slice(&s, i..i + 1));
            } else if i < idx + ins_len {
                let i_ins = i - idx;
                r.push_str(&slice(&ins.to_string(), i_ins..i_ins + 1));
            } else {
                let a_ins = i - ins_len;
                r.push_str(&slice(&s, a_ins..a_ins + 1));
            }
        }
        r
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
        assert!(i <= len(&s), "the index was larger than the target slice");

        let first = slice(&s, 0..i);
        let second = slice(&s, i + l..len(&s));

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
    returns a slice of a string from a range, utf-8 compliant
    */
    pub fn slice(s: &String, r: Range<usize>) -> String {
        let mut sub_string = Vec::<String>::new();
        for (i, c) in s.chars().enumerate() {
            if r.contains(&i) {
                sub_string.push(c.to_string());
            }
        }
        sub_string.join("")
    }
}

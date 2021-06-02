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

pub mod marble {

    use crate::utils::text::*;
    use crate::utils::progress::*;
    // use std::cmp::Ordering;
	use std::fmt;
	use std::str::FromStr;
	// uncomment for debug output
	use std::io::stdin;

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
            Ok(parse_marble(&raw, false))
        }
    }

    impl Page {
    	pub fn new(s: &str, debug: bool) -> Self {
    		let raw = s.to_string();
    		parse_marble(&raw, debug)
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

    pub fn parse_marble(s: &String, debug: bool) -> Page {
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

    fn parse(s: &String, debug: bool) -> String {		
    	let mut t = s.clone();
    	let mut elems = Vec::<String>::new();
    	let mut in_quotes = false;
    	let mut in_content = false;

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
		
		let mut i = 0;
    	while i < len(&t) {
    		if debug {
	    		// Debugging output
	    		print!("{esc}c", esc = 27 as char);
	    		println!("{}@{}", slice(&t, 0..i), slice(&t, i+1..len(&t)));
	    		println!("#################\nelems: {:#?}", elems);
	    		println!("in_quotes: {}", in_quotes);
	    		println!("in_content: {}", in_content);
			    let mut input_string = String::new();

			    loop {
			    	println!("press enter to continue");
				    stdin().read_line(&mut input_string)
				    	.ok()
				        .expect("Failed to read line");
					if input_string == "\n" {
						break
					} 		    	
			    }
		    } else {
				bar.print(i);
			}
			
    		let char = &slice(&t, i..i+1)[..];

			match char {
				"\"" => {
	  				if in_quotes {
	  					in_quotes = false;
	  					// a bit scuffed, but it prevents mark [A] from deleting the character before the closing quote.
	  					i += 1;
	  				} else if !in_quotes {
	  					in_quotes = true;
	  				}
				},
				"[" | "]" => {
					in_content = false;
				},
				// "|" => {
					// if !in_content { in_content = true; }
				// },
				_ => (),
			}
    		
    		if !in_quotes && !in_content { match char {
    			"[" => {
    				t = remove(&t, i, 1);
    				t = insert(&t, i, "<");
    				
    				let next = first_from(&t, i+1).1;
    				t = remove(&t, i+1, next);
    				
    				let mut j = i;
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

/*
pub mod time {

    use crate::utils::text::insert;

    // returns a day, month, and year, from a given epoch number. pretty scuffed.
    pub fn calc_date(s: String) -> (String, String, String) {
        let mut _seconds = s.parse::<i32>().unwrap();
        let mut _minutes = _seconds / 60;
        _seconds -= _minutes * 60;

        let mut _hours = _minutes / 60;
        _minutes -= _hours * 60;

        let mut days = _hours / 24;
        _hours -= days * 24;

        // Unix time starts in 1970 on a Thursday
        let mut year = 1970;
        let mut month = 0;
        let mut _day_of_week = 4;

        loop {
            let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
            let days_in_year = if leap_year { 366 } else { 365 };
            if days >= days_in_year {
                _day_of_week += if leap_year { 2 } else { 1 };
                days -= days_in_year;
                if _day_of_week >= 7 {
                    _day_of_week -= 7;
                }
                year += 1;
            } else {
                _day_of_week += days;
                _day_of_week %= 7;

                // calculate the month and day
                let days_in_month = vec![31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
                for m in 0..12 {
                    let mut dim = days_in_month[m];

                    // add a day to February if this is a leap year
                    if m == 1 && leap_year {
                        dim += 1;
                    }

                    if days >= dim {
                        days -= dim;
                    } else {
                        month = m;
                        break;
                    }
                }
                break;
            }
        }

        days += 1;
        month += 1;

        let mut f_days: String;
        let mut f_month: String;
        let mut f_year: String;

        if days < 10 {
            f_days = days.to_string();
            f_days = insert(&f_days, 0, "0");
        } else {
            f_days = days.to_string();
        }

        if month < 10 {
            f_month = month.to_string();
            f_month = insert(&f_month, 0, "0");
        } else {
            f_month = month.to_string();
        }

        if year < 10 {
            f_year = year.to_string();
            f_year = insert(&f_year, 0, "0");
        } else {
            f_year = year.to_string();
        }

        (f_days, f_month, f_year)
    }
}
*/

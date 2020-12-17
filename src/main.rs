use std::env;
use std::fs;

const HELP_MENU: &str = "Builds static site from marble files \n\
						 \n\
						 USAGE: \n\
						 \tpillar [OPTIONS] [COMMAND] \n\
						 \n\
						 OPTIONS: \n\
						 \t-h\tprints this information \n\
						 \t-V\tprints current version \n\
						 \n\
						 COMMANDS: \n\
						 \tbuild\tbuilds html from marble \n\
						 \tclean\tclears html directory \n";

const TEMPLATE_PATH: &str = "templates/";
const MARBLE_PATH: &str = "pages/";
const HTML_PATH: &str = "docs/";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", HELP_MENU);
        return ()
    }

    let instruction = args[1].as_str();

    match instruction {
    	"-V" => println!("Pillar version {}", env!("CARGO_PKG_VERSION")),
    	"build" => {
    		// gets the marble files
    		match fs::read_dir(MARBLE_PATH) {
    			// unpacks the entries result
    			Ok(entries) => {
    				// goes through the entry objects
    				println!("Parsing pages...");
    				for entry in entries {
    					// unpacks the entry result
    					match entry {
    						Ok(entry) => {
    							// parses the file
    							let path = format!("{:?}", entry.path());
    							let path_str = &path[1..path.len()-1];
    							
    							let contents = fs::read_to_string(path_str)
    								.expect(format!("Something went wrong reading {}", path_str).as_str());

    							let split_contents = contents.lines();
   							    let str_lines: Vec<&str> = split_contents.collect();
   							    let mut lines = Vec::<String>::new();
   							
   							    for str_line in str_lines {
   							        let mut line = String::new();
   							        line.push_str(str_line);
   							        lines.push(line);
   							    }
   							
   							    let parsed = parse_marble(lines).join("");

								let default_template_path = [TEMPLATE_PATH, "default.html"].concat();
   							    let template_contents = fs::read_to_string(default_template_path)
   							    	.expect("couldn't load default template");

   							    let page = replace(&template_contents, "{{content}}", &parsed);

								let target = [HTML_PATH, &path[MARBLE_PATH.len()+1..&path.len()-3], "html"].concat();
								println!("+ {}", target);
   							    match fs::write(&target, &page) {
							    	Ok(_) => (),
							    	Err(e) => println!("failed to write to {}: {}", &target, e),
							    };
   							
    						},
    						Err(e) => {
    							println!("Failed to open entry with error {}", e);
    						}
    					}
    				}
    			},
    			Err(e) => {
    				println!("Failed to open directory {} with error {}", MARBLE_PATH, e);
    				return ()
    			}
    		}
    	},
    	"clean" => {
		 	match fs::read_dir(HTML_PATH) {
    			// unpacks the entries result
    			Ok(entries) => {
    				// goes through the entry objects
    				println!("Deleting html...");
    				for entry in entries {
    					// unpacks the entry result
    					match entry {
    						Ok(entry) => {
    							// parses the file
    							let path = format!("{:?}", entry.path());
    							let path_str = &path[1..path.len()-1];
    							println!("- {}", path_str);
    							if &path_str[path_str.len()-4..] == "html" {
	    							match fs::remove_file(path_str) {
	    								Ok(_) => (),
	    								Err(e) => println!("failed to delete file {} with error {}", path_str, e)
	    							}
    							}
    						}
    						Err(e) => {
						    	println!("Failed to open entry with error {}", e);
						    }
						}
					}
				},
   				Err(e) => {
    				println!("Failed to open directory {} with error {}", HTML_PATH, e);
    				return ()
    			}
    		}
    	},
    	_ => println!("{}", HELP_MENU)
    }
}

fn parse_marble(lines: Vec<String>) -> Vec<String> {
    let mut output = Vec::<String>::new();

	// sets lines which shouldn't be parsed (code and page variables)
    let mut reserved = Vec::<String>::new();
	let mut in_reserved = false;
    for i in 0..lines.len() {
    	let mut line = lines[i].clone();
    	if line.len() > 5 {
	    	let first = first(&line).1;
	    	if &line[first..first+6] == "!code!" && !in_reserved {
	    		in_reserved = true;
	    		line = String::from("<pre><code>");
	    	} else if &line[first..first+6] == "!code!" && in_reserved {
	    		line = String::from("</code></pre>");
	    		in_reserved = false;
	    	} else if in_reserved {
	    		line.push_str("\n");
	    		reserved.push(line);
	    		line = String::from("!reserved!");
	    	}
    	}
    	output.push(line);
    }

    // single line formatting goes in here
   for i in 0..output.len() {
        if output[i].len() > 0 {
            output[i] = h(&output[i]);
            output[i] = em(&output[i]);
            output[i] = img(&output[i]);
            output[i] = a(&output[i]);
        } else {
            // output[i] = String::from("<br>");
        }
    }
    // multi-line formatting goes out here
    output = ul(&output);
    output = ol(&output);
    output = blockquote(&output);

	output = p(&output);
    output = nl(&output);

	let mut reserved_index = 0;
    for i in 0..output.len() {
    	if output[i].contains("!reserved!") {
    		output[i] = reserved[reserved_index].clone();
    		reserved_index += 1;
    	}
    }

    return output;
}

fn nl(l: &Vec<String>) -> Vec<String> {
	let mut output = Vec::<String>::new();
	for i in 0..l.len() {
		let mut line = l[i].clone();
		line.push_str("\n");
		output.push(line.to_string());
	}
	return output
}

fn p(l: &Vec<String>) -> Vec<String> {
	let mut output = Vec::<String>::new();
	for i in 0..l.len() {
		let mut line = l[i].clone();
		if line.len() > 4 {
			let i_first = first(&line).1;
			let four = &line[i_first..i_first+4];
			if four == "<em>" || four == "<a h" || four == "<img" || first(&line).0 != "<" {
				line.insert_str(0, "<p>");
				line.push_str("</p>");
			}
		}
		output.push(line.to_string());
	}
	return output
}

fn blockquote(l: &Vec<String>) -> Vec<String> {
    let mut output = Vec::<String>::new();
    let mut i = 0;
    while i < l.len() {
        let mut line = l[i].clone();
        let char = first(&line).0;

		let mut is_blockquote = false;
		if char == ">" {
			output.push(String::from("<blockquote>"));
			line = remove(&line, first(&line).1, 1);
			line = remove(&line, 0, first(&line).1);
			is_blockquote = true;
		}
        
        output.push(line.to_string());
        if is_blockquote {
			output.push(String::from("</blockquote>"));	
        }
        i += 1;
    }

    return output;
}

fn ol(l: &Vec<String>) -> Vec<String> {
    let mut output = Vec::<String>::new();

    let mut in_list = false;
    let mut level = 0;

    let mut i = 0;
    while i < l.len() {
        let mut line = l[i].clone();
        let char = first(&line).0;
        let space = first(&line).1;

        if char == "~" {
            if !in_list {
                in_list = true;
                level = space;
                output.push(String::from("<ol>"));
            }
            if space > level {
                for _j in 0..space - level {
                    output.push(String::from("<ol>"));
                }
                line = remove(&line, 0, first(&line).1);
                line.remove(0);
                line = remove(&line, 0, first(&line).1);
                line.insert_str(0, "<li>");
                line.insert_str(line.len(), "</li>");
            } else if space < level {
                for _j in 0..level - space {
                    output.push(String::from("</ol>"));
                }
                line = remove(&line, 0, first(&line).1);
                line.remove(0);
                line = remove(&line, 0, first(&line).1);
                line.insert_str(0, "<li>");
                line.insert_str(line.len(), "</li>");
            } else {
                line = remove(&line, 0, first(&line).1);
                line.remove(0);
                line = remove(&line, 0, first(&line).1);
                line.insert_str(0, "<li>");
                line.insert_str(line.len(), "</li>");
            }
            level = space;
        } else if char != "-" && in_list {
            in_list = false;
            for _j in 0..level + 1 {
                output.push(String::from("</ol>"));
            }
            level = 0;
        }
        output.push(line.to_string());
        i += 1;
    }

    return output;
}

fn ul(l: &Vec<String>) -> Vec<String> {
    let mut output = Vec::<String>::new();

    let mut in_list = false;
    let mut level = 0;

    let mut i = 0;
    while i < l.len() {
        let mut line = l[i].clone();
        let char = first(&line).0;
        let space = first(&line).1;

        if char == "-" {
            if !in_list {
                in_list = true;
                level = space;
                output.push(String::from("<ul>"));
            }
            if space > level {
                for _j in 0..space - level {
                    output.push(String::from("<ul>"));
                }
                line = remove(&line, 0, first(&line).1);
                line.remove(0);
                line = remove(&line, 0, first(&line).1);
                line.insert_str(0, "<li>");
                line.insert_str(line.len(), "</li>");
            } else if space < level {
                for _j in 0..level - space {
                    output.push(String::from("</ul>"));
                }
                line = remove(&line, 0, first(&line).1);
                line.remove(0);
                line = remove(&line, 0, first(&line).1);
                line.insert_str(0, "<li>");
                line.insert_str(line.len(), "</li>");
            } else {
                line = remove(&line, 0, first(&line).1);
                line.remove(0);
                line = remove(&line, 0, first(&line).1);
                line.insert_str(0, "<li>");
                line.insert_str(line.len(), "</li>");
            }
            level = space;
        } else if char != "-" && in_list {
            in_list = false;
            for _j in 0..level + 1 {
                output.push(String::from("</ul>"));
            }
            level = 0;
        }
        output.push(line.to_string());
        i += 1;
    }

    return output;
}

fn img(s: &String) -> String {
    let mut line = s.clone();
    let mut in_bracket = false;
    let mut i_bracket: usize = 0;
    let mut in_paren = false;
    let mut i_paren: usize = 0;

    let mut i = 1;
    while i < line.len() {
        let char = &line[i..i + 1];
        let prev_char = &line[i - 1..i];
        if &line[i..i+1] == "[" && prev_char == "!" && !in_bracket {
            in_bracket = true;
            i_bracket = i;
        }
        if in_bracket && prev_char == "]" && char == "(" {
            in_bracket = false;
            in_paren = true;
            i_paren = i;
        }
        if in_paren && char == ")" {
            in_paren = false;
            // grabs link
            let temp_line = line.clone();
            let link = &temp_line[i_paren + 1..i];
            // adds the close img
            line.insert_str(i + 1, "\"/>");
            // removes the closing bracket and link portion
            line = remove(&line, i_paren - 1, i - i_paren + 2);
            // removes opening bracket and !
            line = remove(&line, i_bracket-1, 2);
            // inserts img and alt
            line.insert_str(i_bracket-1, "<img alt=\"");
            // adds src to img
            line.insert_str(i_bracket+4, "src=\"");
            // adds image link to src
            line.insert_str(i_bracket+9, &link);
            // closes src
            line.insert_str(i_bracket+9 + &link.len(), "\" ");
        }
        i += 1;
    }
    // line.push_str("");
    return line;
}

fn a(s: &String) -> String {
    let mut line = s.clone();
    let mut in_bracket = false;
    let mut i_bracket: usize = 0;
    let mut in_paren = false;
    let mut i_paren: usize = 0;

    let mut i = 0;
    while i < line.len() {
        let char = &line[i..i + 1];
        if char == "[" && !in_bracket {
            in_bracket = true;
            i_bracket = i;
        }
        if i > 0 {
            let prev_char = &line[i - 1..i];
            if in_bracket && prev_char == "]" && char == "(" {
                in_bracket = false;
                in_paren = true;
                i_paren = i;
            }
            if in_paren && char == ")" {
                in_paren = false;
                // grabs link
                let temp_line = line.clone();
                let link = &temp_line[i_paren + 1..i];
                // adds the close line
                line.insert_str(i + 1, "</a>");
                // removes the closing bracket and link portion
                line = remove(&line, i_paren - 1, i - i_paren + 2);
                // removes opening bracket
                line = remove(&line, i_bracket, 1);
                // inserts start of html
                line.insert_str(i_bracket, "<a href=\"");
                // adds link to html
                line.insert_str(i_bracket + 9, &link);
                // closes initial link html
                line.insert_str(i_bracket + 9 + &link.len(), "\">");
            }
        }
        i += 1;
    }
    return line;
}

fn em(s: &String) -> String {
    let mut line = s.clone();
    let mut in_italics = false;
    for i in (0..line.len()).rev() {
        let char = &line[i..i + 1];
        if char == "*" && !in_italics {
            line = remove(&line, i, 1);
            line.insert_str(i, "</em>");
            in_italics = true;
        } else if char == "*" && in_italics {
            line = remove(&line, i, 1);
            line.insert_str(i, "<em>");
            in_italics = false;
        }
    }
    return line;
}

// change this, not written in same way as others
fn h(s: &String) -> String {
    let mut line = String::new();
    let mut is_header = false;

    if &s[..3] == "###" {
        line = remove(&s, 0, 3);
        line.insert_str(0, "<h3>");
        line.insert_str(line.len(), "</h3>");
        is_header = true;
    } else if &s[..2] == "##" {
        line = remove(&s, 0, 2);
        line.insert_str(0, "<h2>");
        line.insert_str(line.len(), "</h2>");
        is_header = true;
    } else if &s[..1] == "#" {
        line = remove(&s, 0, 1);
        line.insert_str(0, "<h1>");
        line.insert_str(line.len(), "</h1>");
        is_header = true;
    }

    if is_header {
        let mut hit_text = false;
        for i in (0..line.len() - 6).rev() {
            if !hit_text && &line[i..i + 1] == " " {
                line = remove(&line, i, 1);
            } else {
                hit_text = true;
            }
        }
        hit_text = false;
        let mut i = 4;
        while i < line.len() - 5 {
            if !hit_text && &line[i..i + 1] == " " {
                line = remove(&line, i, 1);
            } else {
                hit_text = true;
                i += 1;
            }
        }
    } else {
        line = s.clone();
        // line = insert(&s, 0, "<p>");
        // line.insert_str(line.len(), "</p>");
    }

    return line;
}

/*
fn insert(s: &String, idx: usize, ins: &str) -> String {
    assert!(idx <= s.len(), "the index was larger than the target slice");

    let fin_len = s.len() + ins.len();
    let mut r = String::with_capacity(fin_len);

    for i in 0..fin_len {
        if i < idx {
            r.push_str(&s[i..i + 1]);
        } else if i < idx + ins.len() {
            let i_ins = i - idx;
            r.push_str(&ins[i_ins..i_ins + 1]);
        } else {
            let a_ins = i - ins.len();
            r.push_str(&s[a_ins..a_ins + 1])
        }
    }
    return r;
}
*/

fn replace(s: &String, target: &str, insert: &str) -> String {
	let mut source = s.clone();
	let i = source.find(target).unwrap();
	source.replace_range(i..i+target.len(), insert);
	return source.to_string()
}

fn remove(s: &String, idx: usize, len: usize) -> String {
    assert!(idx <= s.len(), "the index was larger than the target slice");

    let first = &s[..idx];
    let second = &s[idx + len..];

    return [first, second].concat();
}

fn first(s: &String) -> (String, usize) {
    let line = s.clone();
    let mut num = 0;
    for i in 0..line.len() {
        let char = &line[i..i + 1];
        if char == " " || char == "\t" {
            num += 1;
        } else {
            return (char.to_string(), num);
        }
    }
    return (String::from(""), num);
}


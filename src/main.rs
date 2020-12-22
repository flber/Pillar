use std::env;
use std::fs;
use core::ops::Range;
use std::os::unix::fs::MetadataExt;
use unicode_segmentation::UnicodeSegmentation;

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
const MUSIC_PATH: &str = "/home/benh/Music/";

const LATEST_LENGTH: usize = 15;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", HELP_MENU);
        return ();
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
                                let path_str = slice(&path, 1..len(&path) - 1);

                                let mut contents = fs::read_to_string(&path_str).expect(
                                    format!("Something went wrong reading {}", path_str).as_str(),
                                );

                                let metadata = fs::metadata(&path_str).expect(
                                    format!("couldn't read metadata from {}", path_str).as_str(),
                                );
                                
                                let last_modified = &metadata.mtime().to_string();
                                let date = calc_date(last_modified.to_string());
                                let short_date = format!("{}{}{}", date.0, date.1, &date.2[2..]);

								if contents.contains("{{music}}") {
									let mut music = String::new();
	                                match fs::read_dir(MUSIC_PATH) {
										Ok(albums) => {
											for album in albums {
												match album {
													Ok(a) => {
														music.push_str("- ");
														music.push_str(&format!("{:?}", a.path()));
														music.push_str("\n");
													},
													Err(e) => println!("Failed to open an album with error {}", e),
												}
											}
										},
										Err(e) => println!("Failed to open {} with error {}", MUSIC_PATH, e),
	                                }
	                                music = replace(&music, MUSIC_PATH, "");
	                                music = replace(&music, "\"", "");
	                                contents = replace(&contents, "{{music}}", &music);
                                }

								if contents.contains("{{latest}}") {
									let mut posts = Vec::<(String, String, String)>::new();
	                                match fs::read_dir(MARBLE_PATH) {
										Ok(pages) => {
											for page in pages {
												match page {
													Ok(p) => {
														let page_path = format!("{:?}", p.path());
														let page_path_str = slice(&page_path, 1..len(&page_path)-1);
						                                let page_metadata = fs::metadata(&page_path_str).expect(
						                                    format!("couldn't read metadata from {}", page_path_str).as_str(),
						                                );
						                                let page_last_modified = &page_metadata.mtime().to_string();
						                                let page_date = calc_date(page_last_modified.to_string());
						                                let page_short_date = format!("{}{}{}", page_date.0, page_date.1, &page_date.2[2..]);
						                                
														posts.push(
															(page_path_str, 
															page_short_date, 
															page_last_modified.to_string())
														);
													},
													Err(e) => println!("Failed to open a page with error {}", e),
												}
											}
										},
										Err(e) => println!("Failed to open {} with error {}", MARBLE_PATH, e),
	                                }
	                                posts.sort_by(|a, b| a.2.cmp(&b.2));
	                                posts.reverse();
	                                let mut posts_list = String::new();
	                                for i in 0..posts.len() {
	                                	if i < LATEST_LENGTH {
											posts_list.push_str("- ");
											posts_list.push_str(posts[i].1.as_str());
											posts_list.push_str(" [{");

											let lines = file_to_lines(&posts[i].0);
			                                let mut title = String::from("");
											let header_meta = parse_header(lines).1;
											for header_var in header_meta {
												if header_var.name == "title" {
													title = header_var.value;
												}
											}
											
											if title == "" { 
												let mut title = replace(&posts[i].0, MARBLE_PATH, "");
												title = replace(&title, ".mr", "");
												posts_list.push_str(&title);
											} else {
												posts_list.push_str(&title);
											}
											posts_list.push_str("}](");
											let mut relative_path = replace(&posts[i].0, MARBLE_PATH, "");
											relative_path = replace(&relative_path, ".mr", ".html");
											posts_list.push_str(&relative_path);
											posts_list.push_str(")");
											posts_list.push_str("\n");
	                                	} else {
	                                		break
	                                	}
	                                }
	                                contents = replace(&contents, "{{latest}}", &posts_list);
                                }
                                
                                let split_contents = contents.lines();
                                let str_lines: Vec<&str> = split_contents.collect();
                                let mut lines = Vec::<String>::new();

                                for str_line in str_lines {
                                    let mut line = String::new();
                                    line.push_str(str_line);
                                    lines.push(line);
                                }

								let mut template_file = String::from("default.html");
								let header_meta = parse_header(lines.clone()).1;
								for header_var in header_meta {
									if header_var.name == "template" {
										template_file = header_var.value;
										template_file.push_str(".html");
									}
								}
                                let mut template_path =
                                    [TEMPLATE_PATH, &template_file].concat();
                                    
                                let template_contents = match fs::read_to_string(&template_path) {
                                	Ok(c) => c,
                                	Err(_) => {
                                		template_path = vec![TEMPLATE_PATH, "default.html"].concat();
                                		fs::read_to_string(&template_path)
                                			.expect("couldn't load default template")
                                	}
                                };

								let template_lines = file_to_lines(&template_path);
                                
								let mut whitespace = String::new();
								for line in template_lines {
									if line.contains("{{content}}") {
										whitespace = slice(&line, 0..first(&line).1);
									}
								}

								let wo_header = parse_header(lines).0;
                                let parsed = parse_marble(wo_header, &whitespace).join("");
								let mut con_w_space = String::from("{{content}}");
								con_w_space = insert(&con_w_space, 0, &whitespace);				

                                let page = replace(&template_contents, &con_w_space, &parsed);
                                let page = replace(&page, "{{date}}", &short_date);

                                let target = [
                                    HTML_PATH,
                                    &slice(&path, len(&MARBLE_PATH.to_string()) + 1..len(&path) - 3),
                                    "html",
                                ].concat();
                                println!("+ {}", target);
                                match fs::write(&target, &page) {
                                    Ok(_) => (),
                                    Err(e) => println!("failed to write to {}: {}", &target, e),
                                };
                            }
                            Err(e) => {
                                println!("Failed to open entry with error {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to open directory {} with error {}", MARBLE_PATH, e);
                    return ();
                }
            }
        }
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
                                let path_str = &slice(&path, 1..len(&path) - 1);
                                println!("- {}", path_str);
                                if slice(&path_str, len(&path_str) - 4..len(&path_str)) == "html" {
                                    match fs::remove_file(path_str) {
                                        Ok(_) => (),
                                        Err(e) => println!(
                                            "failed to delete file {} with error {}",
                                            path_str, e
                                        ),
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Failed to open entry with error {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to open directory {} with error {}", HTML_PATH, e);
                    return ();
                }
            }
        }
        _ => println!("{}", HELP_MENU),
    }
}

fn file_to_lines(path: &str) -> Vec<String> {
	let contents = fs::read_to_string(path).expect(
	    format!("Something went wrong reading {}", path).as_str(),
	);
	let split_contents = contents.lines();
	let str_lines: Vec<&str> = split_contents.collect();
	let mut lines = Vec::<String>::new();
	for str_line in str_lines {
	    let mut line = String::new();
	    line.push_str(str_line);
	    lines.push(line);
	}
	return lines
}

struct Metadata {
	name: String,
	value: String
}

fn parse_header(lines: Vec<String>) -> (Vec<String>, Vec<Metadata>) {
	let mut output = Vec::<String>::new();
	let mut meta = Vec::<Metadata>::new();

    let mut in_reserved = false;
    for i in 0..lines.len() {
        let line = lines[i].clone();
        let first = first(&line).1;
        if len(&line) >= first + 6 {
            if slice(&line, first..first + 6) == "!meta!" && !in_reserved {
                in_reserved = true;
            } else if slice(&line, first..first + 6) == "!meta!" && in_reserved {
                in_reserved = false;
            } else if in_reserved {
                match line.find(":") {
                	Some(c_index) => {
		                let mut name = slice(&line, 0..c_index);
		                name = trim(&name, 0, 0);
		                let mut value = slice(&line, c_index+1..len(&line));
		                value = trim(&value, 0, 0);
		                meta.push(Metadata {name, value});
                	},
                	None => ()
                };
            } else {
		        output.push(line);
            }
        }
    }
    return (output, meta);
}

fn parse_marble(lines: Vec<String>, whitespace: &String) -> Vec<String> {
    let mut output = Vec::<String>::new();

    // sets lines which shouldn't be parsed (code and page variables)
    let mut reserved = Vec::<String>::new();
    let mut in_reserved = false;
    for i in 0..lines.len() {
        let mut line = lines[i].clone();
        let first = first(&line).1;
        if len(&line) >= first + 6 {
            if slice(&line, first..first + 6) == "!code!" && !in_reserved {
                in_reserved = true;
                line = String::from("<pre><code>");
            } else if slice(&line, first..first + 6) == "!code!" && in_reserved {
                line = String::from("</code></pre>");
                in_reserved = false;
            } else if in_reserved {
                reserved.push(line);
                line = String::from("!reserved!");
            }
        }
        output.push(line);
    }

    // single line formatting goes in here
    for i in 0..output.len() {
        if len(&output[i]) > 0 {
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

	// adds back lines which were reserved
    let mut reserved_index = 0;
    for i in 0..output.len() {
        if output[i].contains("!reserved!") {
            output[i] = reserved[reserved_index].clone();
            reserved_index += 1;
        }
    }

    output = nl(&output, &whitespace);
    
    return output;
}

fn nl(l: &Vec<String>, whitespace: &String) -> Vec<String> {
    let mut output = Vec::<String>::new();
    for i in 0..l.len() {
        let mut line = l[i].clone();
        line = insert(&line, 0, &whitespace);
        if i != l.len()-1 {
	        line.push_str("\n");
        }
        output.push(line.to_string());
    }
    return output;
}

fn p(l: &Vec<String>) -> Vec<String> {
    let mut output = Vec::<String>::new();
    for i in 0..l.len() {
        let mut line = l[i].clone();
        let i_first = first(&line).1;
        if len(&line) >= i_first + 4 {
            let four = slice(&line, i_first..i_first + 4);
            if four == "<em>" || four == "<a h" || four == "<img" || first(&line).0 != "<" {
                line.insert_str(0, "<p>");
                line.push_str("</p>");
            }
        }
        output.push(line.to_string());
    }
    return output;
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
                line = insert(&line, 0, "<li>");
                line = insert(&line, len(&line), "</li>");
            } else if space < level {
                for _j in 0..level - space {
                    output.push(String::from("</ol>"));
                }
                line = remove(&line, 0, first(&line).1);
                line.remove(0);
                line = remove(&line, 0, first(&line).1);
                line = insert(&line, 0, "<li>");
                line = insert(&line, len(&line), "</li>");
            } else {
                line = remove(&line, 0, first(&line).1);
                line.remove(0);
                line = remove(&line, 0, first(&line).1);
                line = insert(&line, 0, "<li>");
                line = insert(&line, len(&line), "</li>");
            }
            level = space;
        } else if char != "~" && in_list {
            in_list = false;
            for _j in 0..level + 1 {
                output.push(String::from("</ol>"));
            }
            level = 0;
        }
        
        output.push(line.to_string());

        if i == l.len()-1 && in_list {
            in_list = false;
            for _j in 0..level + 1 {
                output.push(String::from("</ol>"));
            }
            level = 0;
        }
        
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
                line = insert(&line, 0, "<li>");
                line = insert(&line, len(&line), "</li>");
            } else if space < level {
                for _j in 0..level - space {
                    output.push(String::from("</ul>"));
                }
                line = remove(&line, 0, first(&line).1);
                line.remove(0);
                line = remove(&line, 0, first(&line).1);
                line = insert(&line, 0, "<li>");
                line = insert(&line, len(&line), "</li>");
            } else {
                line = remove(&line, 0, first(&line).1);
                line.remove(0);
                line = remove(&line, 0, first(&line).1);
                line = insert(&line, 0, "<li>");
                line = insert(&line, len(&line), "</li>");
            }
            level = space;
        } else if (char != "-" || i == l.len()-1) && in_list {
            in_list = false;
            for _j in 0..level + 1 {
                output.push(String::from("</ul>"));
            }
            level = 0;
        }
        output.push(line.to_string());

        if i == l.len()-1 && in_list {
            in_list = false;
            for _j in 0..level + 1 {
                output.push(String::from("</ul>"));
            }
            level = 0;
        }
        
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
    while i < len(&line) {
        let char = slice(&line, i..i + 1);
        let prev_char = slice(&line, i - 1..i);
        if slice(&line, i..i + 1) == "[" && prev_char == "!" && !in_bracket {
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
            let link = slice(&temp_line, i_paren + 1..i);
            // adds the close img
            line = insert(&line, i+1, "\"/>");
            // removes the closing bracket and link portion
            line = remove(&line, i_paren - 1, i - i_paren + 2);
            // removes opening bracket and !
            line = remove(&line, i_bracket - 1, 2);
            // inserts img and alt
            line = insert(&line, i_bracket-1, "<img alt=\"");
            // adds src to img
            line = insert(&line, i_bracket+4, "src=\"");
            // adds image link to src
            line = insert(&line, i_bracket+9, &link);
            // closes src
            line = insert(&line, i_bracket+9+len(&link), "\" ");
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
    while i < len(&line) {
        let char = slice(&line, i..i + 1);
        if char == "[" && !in_bracket {
            in_bracket = true;
            i_bracket = i;
        }
        if i > 0 {
            let prev_char = slice(&line, i - 1..i);
            if in_bracket && prev_char == "]" && char == "(" {
                in_bracket = false;
                in_paren = true;
                i_paren = i;
            }
            if in_paren && char == ")" {
                in_paren = false;
                // grabs link
                let temp_line = line.clone();
                let link = slice(&temp_line, i_paren + 1..i);
                // adds the close line
                line = insert(&line, i + 1, "</a>");
                // removes the closing bracket and link portion
                line = remove(&line, i_paren - 1, i - i_paren + 2);
                // removes opening bracket
                line = remove(&line, i_bracket, 1);
                // inserts start of html
                line = insert(&line, i_bracket, "<a href=\"");
                // closes initial link html
                line = insert(&line, i_bracket + 9, "\">");
                // adds link to html
                line = insert(&line, i_bracket + 9, &link);
            }
        }
        i += 1;
    }
    return line;
}

fn em(s: &String) -> String {
    let mut line = s.clone();
    let mut astrices = 0;

	for i in 0..len(&line) {
		let char = slice(&line, i..i+1);
		if char == "*" {
			astrices += 1;
		}
	}

	if astrices % 2 == 1 { astrices -= 1; }
	if astrices < 2 { return line }

	loop {
		match line.find("*") {
			Some(i) => {
				if astrices % 2 == 0 {
					line.replace_range(i..i+1, "<em>");
				} else {
					line.replace_range(i..i+1, "</em>");
				}
				astrices -= 1;
			},
			None => break,
		}
	}

    return line;
}

// change this, not written in same way as others
fn h(s: &String) -> String {
    let mut line = String::new();
    let mut is_header = false;

	let first = first(&s).1;
    if len(&s) > first+2 && slice(&s, first..first+3) == "###" {
        line = remove(&s, 0, first+3);
        line = insert(&line, 0, "<h3>");
        line = insert(&line, len(&line), "</h3>");
        is_header = true;
    } else if len(&s) > first+1 && slice(&s, first..first+2) == "##" {
        line = remove(&s, 0, first+2);
        line = insert(&line, 0, "<h2>");
        line = insert(&line, len(&line), "</h2>");
        is_header = true;
    } else if len(&s) > first && slice(&s, first..first+1) == "#" {
        line = remove(&s, 0, first+1);
        line = insert(&line, 0, "<h1>");
        line = insert(&line, len(&line), "</h1>");
        is_header = true;
    }

    if is_header {
        line = trim(&line, 4, 6);
    } else {
        line = s.clone();
        // line = insert(&s, 0, "<p>");
        // line.insert_str(len(&line), "</p>");
    }

    return line;
}

fn trim(l: &String, start: usize, end: usize) -> String {
	let mut line = l.clone();
	let mut hit_text = false;
	for i in (0..len(&line) - end).rev() {
		let next = slice(&line, i..i + 1);
	    if !hit_text && (next == " " || next == "\t") {
	        line = remove(&line, i, 1);
	    } else {
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
	        i += 1;
	    }
	}
	return line
}

fn insert(s: &String, idx: usize, ins: &str) -> String {
    assert!(idx <= len(&s), "the index was larger than the target slice");
    let ins_len = len(&ins.to_string());
    let fin_len = len(&s) + ins_len;
    let mut r = String::with_capacity(fin_len);
    for i in 0..fin_len {
        if i < idx {
            r.push_str(&slice(&s, i..i+1));
        } else if i < idx + ins_len {
            let i_ins = i - idx;
            r.push_str(&slice(&ins.to_string(), i_ins..i_ins + 1));
        } else {
            let a_ins = i - ins_len;
            r.push_str(&slice(&s, a_ins..a_ins + 1));
        }
    }
    return r
}

fn replace(s: &String, target: &str, insert: &str) -> String {
    let mut source = s.clone();
	loop {
		match source.find(target) {
			Some(i) => source.replace_range(i..i + len(&target.to_string()), insert),
			None => break,
		}
	}
    return source.to_string();
}

fn remove(s: &String, idx: usize, l: usize) -> String {
    assert!(idx <= len(&s), "the index was larger than the target slice");

    let first = slice(&s, 0..idx);
    let second = slice(&s, idx+l..len(&s));

    return [first, second].concat();
}

fn first(s: &String) -> (String, usize) {
    let line = s.clone();
    let mut num = 0;
    for i in 0..len(&line) {
        let char = slice(&line, i..i + 1);
        if char == " " || char == "\t" {
            num += 1;
        } else {
            return (char.to_string(), num);
        }
    }
    return (String::from(""), num);
}

fn len(s: &String) -> usize {
	let graphemes = UnicodeSegmentation::graphemes(&s[..], true)
		.collect::<Vec<&str>>();
	return graphemes.len()
}

fn slice(s: &String, r: Range<usize>) -> String {
	let graphemes = UnicodeSegmentation::graphemes(&s[..], true)
		.collect::<Vec<&str>>();
	let mut sub_graph = Vec::<&str>::new();
	for i in r {
		sub_graph.push(graphemes[i]);
	}
	return sub_graph.join("")
}

fn calc_date(s: String) -> (String, String, String) {
	let mut _seconds = s.clone().parse::<i32>().unwrap();
    let mut _minutes = _seconds / 60;
    _seconds -= _minutes * 60;

    let mut _hours = _minutes / 60;
    _minutes -= _hours * 60;

    let mut days = _hours / 24;
    _hours -= days * 24;

    /* Unix time starts in 1970 on a Thursday */
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

            /* calculate the month and day */
            let days_in_month = vec![31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            for m in 0..12 {
                let mut dim = days_in_month[m];

                /* add a day to February if this is a leap year */
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
    return (days.to_string(), month.to_string(), year.to_string());
}

/*  ^(;,;)^   */

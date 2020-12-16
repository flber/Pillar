use std::env;
use std::fs;

const HELP_MENU: &str = "\nConverts from marble to html\n";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 0 {
        println!("{}", HELP_MENU);
        return ();
    }
    let file_name = &args[1];
    let file_str = file_name.as_str();
    let target = [&file_str[..&file_str.len() - 2], "html"].concat();

    match &file_str[&file_str.len() - 2..] {
        "mr" => (),
        _ => {
            println!("Wrong type of file, please use .mr files");
            return ();
        }
    };

    let contents = fs::read_to_string(file_name).expect("Something went wrong reading the file");

    let split_contents = contents.lines();
    let str_lines: Vec<&str> = split_contents.collect();
    let mut lines = Vec::<String>::new();

    for str_line in str_lines {
        let mut line = String::new();
        line.push_str(str_line);
        lines.push(line);
    }

    let parsed = parse_marble(lines);
    
    match fs::write(&target, parsed) {
        Ok(_) => (),
        Err(e) => println!("failed to write to {}: {}", &target, e),
    };
}

fn parse_marble(lines: Vec<String>) -> String {
    let mut output = Vec::<String>::new();

    // single line formatting goes in here
    for mut line in lines {
        if line.len() > 0 {
            line = h(&line);
            line = em(&line);
            line = img(&line);
            line = a(&line);
        } else {
            // line = String::from("<br>");
        }
        output.push(line);
    }
    // multi-line formatting goes out here
    output = ul(&output);
    output = ol(&output);

	output = p(&output);
    output = nl(&output);

    return output.join("");
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
		    let four = &line[first(&line).1..4];

			if four != "<h1>" && four != "<h2>" && four != "<h3>" && four != "<ul>" && four != "<ol>" && four != "<li>" {
				line.insert_str(0, "<p>");
				line.push_str("</p>");
			}
		}
		output.push(line.to_string());
	}
	return output
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

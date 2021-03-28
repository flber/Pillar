pub mod marble {

    use crate::utils::text::*;
    use std::cmp::Ordering;
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
            Ok(parse_marble(&raw))
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
                        name = trim(&name, 0, 0);
                        let mut value = slice(&line, c_index + 1..len(&line));
                        value = trim(&value, 0, 0);
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

    /*
    starts by pulling out lines in `!code!` blocks
    replaces `!code!` markers with respective html elements
    puts them into a reserved vec
    replaces the line with a marker for later
    goes through each line and does single line formatting
    does multi line formating
    adds back reserved lines
    returns parsed lines
    */
    pub fn parse_marble(s: &String) -> Page {
        let meta = parse_header(&s).meta;
        let content = &parse_header(&s).content;
        let mut output = Vec::<String>::new();
    
		let split_content = content.lines();
		let str_lines: Vec<&str> = split_content.collect();
		let mut lines = Vec::<String>::new();

		for str_line in str_lines {
		    let mut line = String::new();
		    line.push_str(str_line);
		    lines.push(line);
		}
    
        // sets lines which shouldn't be parsed (code and page variables)
        let mut reserved = Vec::<String>::new();
        let mut in_reserved = false;
        for i in 0..lines.len() {
            let mut line = lines[i].clone();
            let first = first(&line).1;
            if len(&line) >= first + 6 {
                if slice(&line, first..first + 6) == "!code!" && !in_reserved {
                    in_reserved = true;
                    line = String::from("<pre>");
                } else if slice(&line, first..first + 6) == "!code!" && in_reserved {
                    line = String::from("</pre>");
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
                // println!("h");
                output[i] = em(&output[i]);
                // println!("em");
                output[i] = b(&output[i]);
                // println!("b");
                output[i] = img(&output[i]);
                // println!("img");
                output[i] = a(&output[i]);
                // println!("a\n");
            } else {
                // output[i] = String::from("<br>");
            }
        }
        
        // multi-line formatting goes out here
        output = list(&output, "-");
        output = list(&output, "~");
        output = blockquote(&output);

        output = p(&output);

        output = nl(&output);
        

        // adds back lines which were reserved
        let mut reserved_index = 0;
        for i in 0..output.len() {
            if output[i].contains("!reserved!") {
                output[i] = reserved[reserved_index].clone();
                if i != output.len() - 1 {
                    output[i].push('\n');
                }
                reserved_index += 1;
            }
        }

        let mut content = String::new();
        for row in output {
        	content.push_str(row.as_str());
        }

        Page {
            meta,
            content,
        }
    }

    /*
    adds given whitespace to start of each line
    adds a \n to the end of each line
    */
    fn nl(l: &[String]) -> Vec<String> {
        let mut output = Vec::<String>::new();
        for i in 0..l.len() {
            let mut line = l[i].clone();
            if i != l.len() - 1 {
                line.push('\n');
            }
            output.push(line.to_string());
        }
        output
    }

    /*
    if the line doesn't have any special formatting adds paragraph elements
    */
    fn p(l: &[String]) -> Vec<String> {
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
        output
    }

    /*
    adds blockquote elements
    */
    fn blockquote(l: &[String]) -> Vec<String> {
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

        output
    }

    /*
    decides which type of list it should be making based on the given type
    figures out when list starts and adds appropriate element
    if the "level" (whitespace) increases, it adds a new list element before adding the line
    if the "level" decreases, it adds a close list element after adding the line
    once the list ends, it adds the necessary number of close list elements
    */
    fn list(l: &[String], point: &str) -> Vec<String> {
        let mut output = Vec::<String>::new();

        let mut start = String::from("");
        let mut end = String::from("");
        if point == "-" {
            start = String::from("<ul>");
            end = String::from("</ul>");
        } else if point == "~" {
            start = String::from("<ol>");
            end = String::from("</ol>");
        }

        let mut in_list = false;
        let mut level = 0;

        let mut i = 0;
        while i < l.len() {
            let mut line = l[i].clone();
            let char = first(&line).0;
            let space = first(&line).1;

            if char == point {
                if !in_list {
                    in_list = true;
                    level = space;
                    output.push(String::from(&start));
                }
                match space.cmp(&level) {
                    Ordering::Greater => {
                        for _j in 0..space - level {
                            output.push(String::from(&start));
                        }
                        line = remove(&line, 0, first(&line).1);
                        line.remove(0);
                        line = remove(&line, 0, first(&line).1);
                        line = insert(&line, 0, "<li>");
                        line = insert(&line, len(&line), "</li>");
                    }
                    Ordering::Less => {
                        for _j in 0..level - space {
                            output.push(String::from(&end));
                        }
                        line = remove(&line, 0, first(&line).1);
                        line.remove(0);
                        line = remove(&line, 0, first(&line).1);
                        line = insert(&line, 0, "<li>");
                        line = insert(&line, len(&line), "</li>");
                    }
                    _ => {
                        line = remove(&line, 0, first(&line).1);
                        line.remove(0);
                        line = remove(&line, 0, first(&line).1);
                        line = insert(&line, 0, "<li>");
                        line = insert(&line, len(&line), "</li>");
                    }
                }
                level = space;
            } else if char != point && in_list {
                in_list = false;
                for _j in 0..level + 1 {
                    output.push(String::from(&end));
                }
                level = 0;
            }

            output.push(line.to_string());

            if i == l.len() - 1 && in_list {
                in_list = false;
                for _j in 0..level + 1 {
                    output.push(String::from(&end));
                }
                level = 0;
            }

            i += 1;
        }

        output
    }

    /*
    finds and formats images
    */
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
                line = insert(&line, i + 1, "\"/>");
                // removes the closing bracket and link portion
                line = remove(&line, i_paren - 1, i - i_paren + 2);
                // removes opening bracket and !
                line = remove(&line, i_bracket - 1, 2);
                // inserts img and alt
                line = insert(&line, i_bracket - 1, "<img alt=\"");
                // adds src to img
                line = insert(&line, i_bracket + 4, "src=\"");
                // adds image link to src
                line = insert(&line, i_bracket + 9, &link);
                // closes src
                line = insert(&line, i_bracket + 9 + len(&link), "\" ");
            }
            i += 1;
        }
        // line.push_str("");
        line
    }

    /*
    finds and formats links
    */
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
                    line = insert(&line, i_bracket + 9, "\" loading=\"lazy\">");
                    // adds link to html
                    line = insert(&line, i_bracket + 9, &link);
                }
            }
            i += 1;
        }
        line
    }

    /*
    goes through the line and replaces "*"'s with an even number of em elements
    leaves out the last one if the total number if odd
    */
    fn em(s: &String) -> String {
        let mut line = s.clone();
        let mut astrices = 0;

        for i in 0..len(&line) {
            let char = slice(&line, i..i + 1);
            if char == "*" {
                astrices += 1;
            }
        }

        if astrices % 2 == 1 {
            astrices -= 1;
        }
        if astrices < 2 {
            return line;
        }

        while let Some(i) = line.find('*') {
            if astrices % 2 == 0 {
                line.replace_range(i..i + 1, "<em>");
            } else {
                line.replace_range(i..i + 1, "</em>");
            }
            astrices -= 1;
        }

        line
    }

    fn b(s: &String) -> String {
        let mut line = s.clone();
        let mut astrices = 0;

        for i in 0..len(&line) {
            let char = slice(&line, i..i + 1);
            if char == "^" {
                astrices += 1;
            }
        }

        if astrices % 2 == 1 {
            astrices -= 1;
        }
        if astrices < 2 {
            return line;
        }

        while let Some(i) = line.find('^') {
            if astrices % 2 == 0 {
                line.replace_range(i..i + 1, "<b>");
            } else {
                line.replace_range(i..i + 1, "</b>");
            }
            astrices -= 1;
        }

        line
    }

    /*
    trims line
    replaces headers with elements
    */
    fn h(s: &String) -> String {
        let mut line = s.clone();
        line = trim(&line, 0, len(&line));
        let mut is_header = false;

        let f = first(&s).1;
        if len(&s) > f + 2 && slice(&s, f..f + 3) == "###" {
            line = remove(&s, 0, f + 3);
            line = remove(&line, 0, first(&line).1);
            line = insert(&line, 0, "<h3>");
            line = insert(&line, len(&line), "</h3>");
            is_header = true;
        } else if len(&s) > f + 1 && slice(&s, f..f + 2) == "##" {
            line = remove(&s, 0, f + 2);
            line = remove(&line, 0, first(&line).1);
            line = insert(&line, 0, "<h2>");
            line = insert(&line, len(&line), "</h2>");
            is_header = true;
        } else if len(&s) > f && slice(&s, f..f + 1) == "#" {
            line = remove(&s, 0, f + 1);
            line = remove(&line, 0, first(&line).1);
            line = insert(&line, 0, "<h1>");
            line = insert(&line, len(&line), "</h1>");
            is_header = true;
        }

        if !is_header {
            line = s.clone();
            // line = insert(&s, 0, "<p>");
            // line.insert_str(len(&line), "</p>");
        }

        line
    }
}

pub mod text {

    use core::ops::Range;

    /*
    removes whitespace around given string from start and end indices
    */
    pub fn trim(l: &String, start: usize, end: usize) -> String {
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
        line
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
    pub fn remove(s: &String, idx: usize, l: usize) -> String {
        assert!(idx <= len(&s), "the index was larger than the target slice");

        let first = slice(&s, 0..idx);
        let second = slice(&s, idx + l..len(&s));

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
        // let graphemes = UnicodeSegmentation::graphemes(&s[..], true).collect::<Vec<&str>>();
        // let mut sub_graph = Vec::<&str>::new();
        // for i in r {
        // sub_graph.push(graphemes[i]);
        // }
        // sub_graph.join("")
        let mut sub_string = Vec::<String>::new();
        for (i, c) in s.chars().enumerate() {
            if r.contains(&i) {
                sub_string.push(c.to_string());
            }
        }
        sub_string.join("")
    }
}

pub mod time {

    use crate::utils::text::insert;

    /*
    returns a day, month, and year, from a given epoch number
    */
    pub fn calc_date(s: String) -> (String, String, String) {
        let mut _seconds = s.parse::<i32>().unwrap();
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

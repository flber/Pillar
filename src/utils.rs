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
	pub fn replace(s: &str, target: &str, insert: &str) -> String {
		let mut out = s.to_string();
		while let Some(i) = s.find(target) {
			out.replace_range(i..i + len(target), insert);
		}
		out
	}

	/*
	removes from String from index with length, preserving graphemes
	*/
	pub fn remove(s: &str, i: usize, l: usize) -> String {
		assert!(i <= len(s), "the index was larger than the target slice");

		let first = slice(s, 0..i);
		let second = slice(s, i + l..len(s));

		[first, second].concat()
	}

	/*
	returns the first character in a string, as well as the index of that string
	*/
	pub fn first(s: &str) -> (String, usize) {
		let mut num = 0;
		for i in 0..len(s) {
			let char = slice(s, i..i + 1);
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
	pub fn first_from(s: &str, i: usize) -> (String, usize) {
		first(&slice(s, i..len(s)))
	}

	/*
	returns the length of a String, taking graphemes into account
	*/
	pub fn len(s: &str) -> usize {
		// let graphemes = UnicodeSegmentation::graphemes(&s[..], true).collect::<Vec<&str>>();
		// graphemes.len()
		s.chars().count()
	}

	/*
	inserts str into string, preserving graphemes
	*/
	pub fn insert(s: &str, idx: usize, ins: &str) -> String {
		assert!(idx <= len(s), "the index was larger than the target slice");

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
	pub fn slice(s: &str, r: Range<usize>) -> String {
		let begin = s.char_indices().nth(r.start).map(|(i, _)| i).unwrap_or(0);
		let end = s
			.char_indices()
			.nth(r.end)
			.map(|(i, _)| i)
			.unwrap_or(s.len());
		s[begin..end].to_string()
	}
}

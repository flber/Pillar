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
        match new_char {
          "]" => {
            t = remove(&t, i, 1);
            let elem = match elems.pop() {
              Some(e) => e,
              None => String::from(""),
            };
            let end_tag = &format!("</{}>", elem);
            t = insert(&t, i, end_tag);
          }
          _ => (),
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

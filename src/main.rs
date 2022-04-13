// use core::ops::Range;
// use std::cmp::Ordering;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::os::unix::fs::MetadataExt;
use std::process::{Command, Stdio};
use std::time::SystemTime;
use std::{fs::File, io::ErrorKind};
use toml::Value;
mod utils;
use utils::granite::*;
use utils::text::*;
// #[macro_use]
// extern crate lazy_static;

// basic help menu items to generate responses to unknown commands
const HELP_MENU: &str = "Builds static site from granite files \n\nUSAGE: \n\tpillar [OPTIONS] [COMMAND] \n\nOPTIONS: \n\t-h\tprints this information \n\t-V\tprints current version \n\nCOMMANDS: \n\tbuild\tbuilds html from granite \n\tclean\tclears html directory \n";

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

fn get_banner() -> std::string::String {
  format!(
    "{} Version {}
Convert granite to html\n",
    AUTHORS, VERSION,
  )
}

// prints generated help menu
fn usage() {
  println!("{}", get_banner());
  println!("{}", HELP_MENU);
}

fn main() -> std::io::Result<()> {
  // flags to define program behaviour
  let mut should_build = false;
  let mut debug_active = false;

  let args: Vec<String> = env::args().collect();
  match &args[..] {
    [_] => usage(),
    // single-command options: verion info, help page, and normal build
    [_, cmd] => match cmd.as_str() {
      "-V" | "--version" => println!("Version: {}", VERSION),
      "-h" | "--help" => usage(),
      "build" => {
        should_build = true;
      }
      // clean does nothing right now...
      "clean" => (),
      _ => println!("{}", HELP_MENU),
    },
    // command with option, allows for build command with debug flag
    [_, cmd, opt] => match cmd.as_str() {
      "-V" | "--version" => println!("Version: {}", VERSION),
      "-h" | "--help" => usage(),
      "build" => {
        should_build = true;
        match opt.as_str() {
          "--debug" => debug_active = true,
          _ => (),
        }
      }
      // this clean also does nothing right now...
      "clean" => (),
      _ => println!("{}", HELP_MENU),
    },
    _ => usage(),
  }

  if should_build {
    // a config struct has path information and a last run date
    let config = Config::new().unwrap();
    // uses config info to go through granite directory files and build them into html
    for e in fs::read_dir(&config.granite_path)? {
      let entry = e?;
      // normalizes path str
      let path = format!("{:?}", entry.path());
      let path_str = slice(&path, 1..len(&path) - 1);

      // gets and formats file metadata, modified and creation time
      let meta = fs::metadata(&path_str)?;
      let modified = meta.mtime() as u64;
      let created = meta
        .created()
        .unwrap()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

      // reads file to str and generates header variables from the `parse_header(&String)` function
      let mut static_build = false;
      let contents =
        fs::read_to_string(&path_str).expect("Something went wrong reading a granite file");
      // -> utils.rs:[parse_header(l: &String)]
      let page_vars = parse_header(&contents).meta;
      for pvar in page_vars {
        if pvar.name == "static" {
          static_build = true;
        }
      }

      // re-builds the file if it was modified after the last build, or if it's a static page
      if (modified > config.last_run) | static_build {
        // formats target string to look like html_path/file.html
        let target = [
          config.html_path.clone(),
          slice(
            &path,
            len(&config.granite_path.to_string()) + 1..len(&path) - 3,
          ),
          String::from("html"),
        ]
        .concat();
        // just changes what symbol pillar outputs when printing build status
        if created > config.last_run {
          println!("+ {}", target);
        } else {
          println!("~ {}", target);
        }

        // parses content into Page
        // -> utils.rs:[Page::new(s: &str, debug: bool)]
        let page = Page::new(&contents, debug_active);
        // makes progress bars on different lines
        println!();

        let mut templated_string = templated(&config, &page);
        //This is where plugins are run
        templated_string = run_plugins(&config, &path_str, &templated_string)?;
        // let completed = replace(&templated_string, "{{date}}", &short_date);
        match fs::write(&target, templated_string) {
          Ok(_) => (),
          Err(e) => println!("failed to write to {}: {}", &target, e),
        };
      }
    }
  }

  Config::update_time();
  Ok(())
}

fn run_plugins(config: &Config, path_str: &str, contents: &String) -> std::io::Result<String> {
  let mut output = [path_str, "\n", contents].concat().to_string();

  // this is sorta gross, but does a fine job of pulling plugin tags as long as the formatting is correct
  let mut to_run = Vec::<String>::new();
  for (i, chr) in contents.char_indices() {
    if chr == '{' && slice(&contents, i - 1..i) == "{" {
      let mut tag = slice(&contents, i..i + 1);
      let mut num_close = 0;
      for test_chr in slice(&contents, i..len(&contents)).chars() {
        if !tag.is_empty() && test_chr != '}' {
          tag.push(test_chr);
        }
        if !tag.is_empty() && test_chr == '}' {
          num_close += 1;
          if num_close > 1 {
            break;
          }
        }
      }
      if !tag.is_empty() {
        to_run.push(tag);
      }
    }
  }
  let plugins: Vec<String> = to_run.into_iter().map(|s| slice(&s, 2..len(&s))).collect();

  for plugin in plugins {
    // fixing path
    let script_str = format!("./{}{}", &config.plugin_path, plugin);

    // run script
    let process = match Command::new(script_str)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn()
    {
      Err(why) => panic!("couldn't spawn process: {}", why),
      Ok(process) => process,
    };

    match process.stdin.unwrap().write_all(output.as_bytes()) {
      Err(why) => panic!("couldn't write to script stdin: {}", why),
      Ok(_) => (),
    }

    let mut s = String::new();
    match process.stdout.unwrap().read_to_string(&mut s) {
      Err(why) => panic!("couldn't read script stdout: {}", why),
      Ok(_) => (),
    }

    output = s;
  }
  let mut final_string = String::new();
  for i in 0..len(&output) {
    let char = &slice(&output, i..i + 1);
    if char == "\n" {
      final_string = slice(&output, i + 1..len(&output));
      break;
    }
  }
  return Ok(final_string);
}

/*
Takes a config file (for the template path) and a page and outputs the html with everything
*/
fn templated(config: &Config, page: &Page) -> String {
  // starts with default template file
  let mut template_file = String::from("default.html");
  for header_var in &page.meta {
    if header_var.name == "template" {
      // if the granite meta header has a template value, sets `template_file` to that
      template_file = header_var.value.clone();
      template_file.push_str(".html");
    }
  }
  let template_path = [config.template_path.clone(), template_file].concat();

  // gets the contents of the given template file
  let template_contents = match fs::read_to_string(&template_path) {
    Ok(c) => c,
    Err(_) => {
      // if it can't be loaded, just load the default
      let mut temp_path = config.template_path.clone();
      temp_path.push_str("default.html");
      fs::read_to_string(&temp_path).expect("couldn't load default template")
    }
  };

  let template_lines: Vec<&str> = template_contents.as_str().lines().collect();

  // figures out how indented the content marker is (non-functional)
  let mut whitespace = String::new();
  for l in template_lines {
    let line = l.to_string();
    if line.contains("{{content}}") {
      whitespace = slice(&line, 0..first(&line).1);
    }
  }

  // replaces content in template
  let mut con_w_space = String::from("{{content}}");
  con_w_space = insert(&con_w_space, 0, &whitespace);
  replace(&template_contents, &con_w_space, &page.content)
}

struct Config {
  template_path: String,
  granite_path: String,
  html_path: String,
  plugin_path: String,
  // music_path: String,
  // latest_length: usize,
  last_run: u64,
}

impl Config {
  fn new() -> Option<Config> {
    File::open(".pillar.toml").unwrap_or_else(|error| {
      if error.kind() == ErrorKind::NotFound {
        File::create(".pillar.toml").unwrap_or_else(|create_error| {
          panic!("Problem creating the file: {:?}", create_error);
        });
        let default = "[paths]\n\
	                template_path = \"templates/\"\n\
	                granite_path = \"pages/\"\n\
	                html_path = \"docs/\"\n\
	                plugin_path = \"plugins/\"\n\
	                music_path = \"/home/user/Music/\"\n\
	                \n\
	                [values]\n\
	                latest_length = 15\n\
	                last_run = 0";
        fs::write(".pillar.toml", default).unwrap();
        File::open(".pillar.toml").unwrap()
      } else {
        panic!("Problem opening the file: {:?}", error);
      }
    });

    let config_string = fs::read_to_string(".pillar.toml").unwrap();
    let config = config_string.parse::<Value>().unwrap();

    let template_path = config["paths"]["template_path"].to_string();
    let granite_path = config["paths"]["granite_path"].to_string();
    let html_path = config["paths"]["html_path"].to_string();
    let plugin_path = config["paths"]["plugin_path"].to_string();
    // let music_path = config["paths"]["music_path"].to_string();
    // let latest_length = config["values"]["latest_length"]
    // .to_string()
    // .parse::<usize>()
    // .unwrap();
    let last_run = config["values"]["last_run"]
      .to_string()
      .parse::<u64>()
      .unwrap();

    Some(Config {
      template_path: slice(&template_path, 1..len(&template_path) - 1),
      granite_path: slice(&granite_path, 1..len(&granite_path) - 1),
      html_path: slice(&html_path, 1..len(&html_path) - 1),
      plugin_path: slice(&plugin_path, 1..len(&plugin_path) - 1),
      // music_path: slice(&music_path, 1..len(&music_path)-1),
      // latest_length,
      last_run,
    })
  }

  fn update_time() {
    let config_str = &fs::read_to_string(".pillar.toml").unwrap();
    let now = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_secs();

    let mut config_string = String::new();
    for line in config_str.lines() {
      if slice(&line.to_string(), 0..8) == "last_run" {
        config_string.push_str(&format!("last_run = {}\n", now));
      } else {
        config_string.push_str(&format!("{}\n", line));
      }
    }

    fs::write(".pillar.toml", config_string).unwrap();
  }
}

/*  ^(;,;)^   */

// use core::ops::Range;
// use std::cmp::Ordering;
use std::env;
use std::fs;
use std::{fs::File, io::ErrorKind};
use toml::Value;
mod utils;
use std::os::unix::fs::MetadataExt;
use utils::marble::*;
use utils::text::*;
use utils::time::*;

const HELP_MENU: &str = "Builds static site from marble files \n\nUSAGE: \n\tpillar [OPTIONS] [COMMAND] \n\nOPTIONS: \n\t-h\tprints this information \n\t-V\tprints current version \n\nCOMMANDS: \n\tbuild\tbuilds html from marble \n\tclean\tclears html directory \n";

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

fn get_banner() -> std::string::String {
    format!(
        "{} Version {}
		Convert marble to html\n",
        AUTHORS, VERSION,
    )
}

fn usage() {
    println!("{}", get_banner());
    println!("{}", HELP_MENU);
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    match &args[..] {
        [_] => usage(),
        [_, cmd] => match cmd.as_str() {
            "-V" | "--version" => println!("Version: {}", VERSION),
            "-h" | "--help" => usage(),
            "build" => {
                let config = Config::new().unwrap();
                for e in fs::read_dir(&config.marble_path)? {
                    let entry = e?;
                    // fixing path
                    let path = format!("{:?}", entry.path());
                    let path_str = slice(&path, 1..len(&path) - 1);

                    // gets contents of marble file
                    let mut contents = fs::read_to_string(&path_str)
                        .expect("Something went wrong reading a marble file");

                    let short_date = get_date_meta(&path_str);

                    // replaces music marker with an unordered list of folders in your music dir
                    if contents.contains("{{music}}") {
                        contents = replace_music(&contents, &config.music_path);
                    }

                    // replaces latest marker with the `LATEST_LENGTH` most recently modified mr pages
                    if contents.contains("{{latest}}") {
                        contents =
                            replace_latest(&contents, &config.marble_path, config.latest_length);
                    }

					// generates target string
                    let target = [
                        config.html_path.clone(),
                        slice(
                            &path,
                            len(&config.marble_path.to_string()) + 1..len(&path) - 3,
                        ),
                        String::from("html"),
                    ]
                    .concat();
                    
                    println!("+ {}", target);

                    // replaces content and date markers
                    contents = replace(&contents, "{{date}}", &short_date, true);
                    let page = contents.parse::<Page>().unwrap();
                    // this just adds a newline so the progress bars are on separate lines
                    println!();
                    
                    let templated_string = templated(&config, &page);
                    let completed = replace(&templated_string, "{{date}}", &short_date, true);
                    match fs::write(&target, completed) {
                        Ok(_) => (),
                        Err(e) => println!("failed to write to {}: {}", &target, e),
                    };
                }
            }
            "clean" => (),
            _ => println!("{}", HELP_MENU),
        },
        _ => usage(),
    }

    Ok(())
}

struct Config {
    template_path: String,
    marble_path: String,
    html_path: String,
    music_path: String,
    latest_length: usize,
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
                    marble_path = \"pages/\"\n\
                    html_path = \"docs/\"\n\
                    music_path = \"/home/user/Music/\"\n\
                    \n\
                    [values]\n\
                    latest_length = 15";
                fs::write(".pillar.toml", default).unwrap();
                File::open(".pillar.toml").unwrap()
            } else {
                panic!("Problem opening the file: {:?}", error);
            }
        });

        let config_string = fs::read_to_string(".pillar.toml").unwrap();
        let config = config_string.parse::<Value>().unwrap();

        let template_path = config["paths"]["template_path"].to_string();
        let marble_path = config["paths"]["marble_path"].to_string();
        let html_path = config["paths"]["html_path"].to_string();
        let music_path = config["paths"]["music_path"].to_string();
        let latest_length = config["values"]["latest_length"]
            .to_string()
            .parse::<usize>()
            .unwrap();

        Some(Config {
            template_path: slice(&template_path, 1..len(&template_path) - 1),
            marble_path: slice(&marble_path, 1..len(&marble_path) - 1),
            html_path: slice(&html_path, 1..len(&html_path) - 1),
            music_path: slice(&music_path, 1..len(&music_path) - 1),
            latest_length,
        })
    }
}

fn get_date_meta(path_str: &str) -> String {
    //get metadata of marble file
    let metadata = fs::metadata(path_str)
        .unwrap_or_else(|_| panic!("couldn't read metadata from {}", path_str));

    // calculates the date in DDMMYY format
    let last_modified = &metadata.mtime().to_string();
    let date = calc_date(last_modified.to_string());
    format!("{}{}{}", date.0, date.1, &date.2[2..])
}

fn replace_music(contents: &String, path: &str) -> String {
    let mut music = String::new();
    match fs::read_dir(path) {
        Ok(albums) => {
            for album in albums {
                match album {
                    Ok(a) => {
                        music.push_str("- ");
                        music.push_str(&format!("{:?}", a.path()));
                        music.push('\n');
                    }
                    Err(e) => println!("Failed to open an album with error {}", e),
                }
            }
        }
        Err(e) => println!("Failed to open {} with error {}", path, e),
    }
    music = replace(&music, path, "", true);
    music = replace(&music, "\"", "", true);
    replace(&contents, "{{music}}", &music, true)
}

fn replace_latest(contents: &String, path: &str, l: usize) -> String {
    // this is a vec of [path, formatted date, epoch date]
    let mut posts = Vec::<(String, String, String)>::new();
    match fs::read_dir(path) {
        Ok(pages) => {
            for page in pages {
                match page {
                    Ok(p) => {
                        let page_path = format!("{:?}", p.path());
                        let page_path_str = slice(&page_path, 1..len(&page_path) - 1);
                        let page_metadata = fs::metadata(&page_path_str).unwrap_or_else(|_| {
                            panic!("couldn't read metadata from {}", page_path_str)
                        });
                        let page_last_modified = &page_metadata.mtime().to_string();
                        let page_date = calc_date(page_last_modified.to_string());
                        let page_short_date =
                            format!("{}{}{}", page_date.0, page_date.1, &page_date.2[2..]);

                        posts.push((
                            page_path_str,
                            page_short_date,
                            page_last_modified.to_string(),
                        ));
                    }
                    Err(e) => println!("Failed to open file with error {}", e),
                }
            }
        }
        Err(e) => println!("Failed to open file with error {}", e),
    }
    // sorts the list by epoch data from earliest to latest
    posts.sort_by(|a, b| a.2.cmp(&b.2));
    posts.reverse();
    // an unordered list of the most recent posts
    let mut posts_list = String::new();
    for (i, post) in posts.iter().enumerate() {
        if i < l {
            posts_list.push_str("- ");
            posts_list.push_str(post.1.as_str());
            posts_list.push_str(" [");

            let mut title = String::from("");
            // gets the meta header from the post you're checking
            let temp_contents =
                fs::read_to_string(&post.0).expect("Something went wrong reading a marble file");
            let header_meta = parse_header(&temp_contents).meta;
            for header_var in header_meta {
                if header_var.name == "title" {
                    // gets the title of the post
                    title = header_var.value.clone();
                }
            }

            if title.is_empty() {
                let mut title = replace(&post.0, path, "", true);
                title = replace(&title, ".mr", "", true);
                posts_list.push_str(&title);
            } else {
                posts_list.push_str(&title);
            }
            posts_list.push_str("](");
            let mut relative_path = replace(&post.0, path, "", true);
            relative_path = replace(&relative_path, ".mr", ".html", true);
            posts_list.push_str(&relative_path);
            posts_list.push(')');
            posts_list.push('\n');
        } else {
            break;
        }
    }
    // this is in the format of `- DDMMYY [{Title}](path)` where the path is modified to be an html file in the html dir
    replace(&contents, "{{latest}}", &posts_list, true)
}

fn templated(config: &Config, page: &Page) -> String {
    // starts with default template file
    let mut template_file = String::from("default.html");
    for header_var in &page.meta {
        if header_var.name == "template" {
            // if the marble meta header has a template value, sets `template_file` to that
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

    // figures out how indented the content marker is
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
    replace(&template_contents, &con_w_space, &page.content, true)
}

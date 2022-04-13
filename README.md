# Pillar
A simple site generator with template support, written in plain (and now slightly more idiomatic) Rust, with support for extensions in whatever language you prefer.

Used in [[my site]](https://mineralexistence.com)

A little demo is available in the `examples/` folder of this repo, which shows off Pillar's features (and honestly is mostly a place for me to test and debug).

## Commands

Pillar has only a few commands. With no arguments, it will give you the help menu. That help menu will also be printed with the arguments `--help` or `-h`. Run `./pillar build` to generate your site, and `./pillar build --debug` to get the debug parser, where it will show you what it's doing step-by-step (note that this is *much* slower, so should be used only if you want to see how the parser works).
That's all of the command arguments, and anything else will just give you the help menu.

## Configuration

Pillar expects a `.pillar.toml` file to be in the same directory as your `pillar` executable, which controls the customization. It's currently quite a small file, the default being:
```toml
[paths]
template_path = "templates/"
granite_path = "pages/"
html_path = "docs/"
plugin_path = "plugins/"
music_path = "/home/user/Music/"
	
[values]
latest_length = 15
```
Pillar takes all .gn files in the `granite_path` directory and parses them into html, templating them with the templates in the `template_path` directory, and outputting the html to the `html_path` directory. Before the html is saved to the html directory, each page is passed through all the scripts in the `plugin_path` directory (see the [extensions](#Extensions) section).

The `latest_length` parameter is currently not used, but was originally intended to control the length of the list of latest pages (see the [extensions](#Extensions) section for more information on how extensions are currently handled).

## Granite

Pillar uses the Granite (.gn)  markup format, which is more similar HTML than it is to markdown. It also isn't too picky about whitespace.
Granite maps 1:1 with html, so the conversion process is relatively simple, and is done in a single pass. The syntax gets rid of close tags in favor of close square brackets, as so:

html: `<p> This is a paragraph </p>`

granite: `[p|This is a paragraph]`

The only other difference in syntax is the use of commas and semicolons in the elements' attribute sections. This is a small change, and the normal html syntax can also be used.

html: `<a href="link.com">A link</a>`

granite: `[a, href: "link.com"|A link]`

As you can see, the Granite syntax is very similar to html, but is in my mind a bit easier to read and write, mostly due to the lack of close tags and its small syntax footprint. The fact that it maps to html also means that any html element can be used; there are none of the restrictions that markdown has. For more examples, look to the `example` folder in this repository. If you like, just clone that directory to your system, add the pillar executable, and run it. This will give you a small, but working, example of a Pillar setup.

## Headers

- To define a metadata header for a page, put `!meta!` at the top and bottom of your variables
- variables are declared with the `name: value` pattern (it's not too particular about whitespace)
- the `title` variable is used to set the title (used for the `{{latest}}` substitution)
- the `template` variable sets the template for the page, defaulting to `default`, where the value is the file name (without extension) of a template in the given template directory
- if the `static` variable is set to a value of "true", the page will always be parsed when pillar is run. This useful for pages which you want to be updated by a script every time pillar is run, and not just when that page is changed.

Example:
```
!meta!
title: Example page
template: fancy_template
!meta!
```

## Extensions

Pillar supports extensions in the form of executable scripts in your defined plugins folder. These work by receiving each parsed page, in html, through stdin. The script should then return a modified form of the page in its stdout. There are several extensions in the example folder, which replace the following tags:
- `{{date}}` replaces with the date the content was last modified (the granite, not the html itself)
- `{{doc-gen}}` generate documentation from comments in python plugins in the plugin directory
- `{{feed}}` is a basic rss feed generator
- `{{files}}` gives a list of files in a `/files` directory
- `{{latest}}` replaces with a given number of the latest updated pages in an unordered list
- `{{music}}` replaces with an unordered list of your album directory names in a designated music path
- `{{stats}}` just gives a list of orphaned pages (the algorithm is not super complete, and may not be entirely accurate)
- `{{title}}` is mostly for the html templates to get the title of a page based on its first h2 tag
- `{{tree}}` is an in-development tree printout of page interconnectivity; non-functional

## Syntax Highlighting

If you use the [micro text editor](https://github.com/zyedidia/micro), I wrote a syntax highlighting file which works nicely with Granite files which you can find in the `examples` folder (or on my website here: `https://mineralexistence.com/files/micro/micro-granite.yaml`). Just put it into your `.config/micro/syntax/` directory, and you should get decent syntax highlighting.

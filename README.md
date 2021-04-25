r
# Pillar
A simple site generator with template support, written in plain (and now slightly more idiomatic) Rust.

## Now with unicode support!

Used in [[my site]](https://mineralexistence.com)

A nice little demo is available in the `examples/` folder of this repo, which shows off Pillar's features (and honestly is mostly a place for me to test and debug). The main page is currently in (probably totally meaningless) Japanese, to stress test the unicode support.

Pillar uses the marble markup format, which is quite similar to markdown but with a few changes:
- Like markdown, headers are defined with `#`. Marble has support for header levels 1 to 3 (because when are you really using anything smaller?)
- Italics are also defined with `*`
- Bold is defined by surrounding the text with `^`
- Images and links follow normal markdown syntax
- Unordered lists still use `-`, but it's far less picky about initial indentation than some markdown parsers (I'm looking at you cmark) and supports weird changes in indentation
- Ordered lists just use `~`, so no need for manually numbering your lists. Same indentation support as ordered lists.
- Blockquotes use the same syntax as markdown, a `>` with as much or as little whitespace before and after as you want.
- To define a code block use `!code!` at the top and bottom of your code block
- To define a metadata header, put `!meta!` at the top and bottom of your variables
	- variables are declared with the `name: value` pattern (it's not too particular about whitespace)
	- the `title` variable is used to set the title for the `{{latest}}` substitution
	- without this deceleration the `{{latest}}` substitution will just use trimmed filenames
	- the `template` variable sets the template for the page, defaulting to `default`, where the value is the file name (without extension) of a template in the given template directory
- `{{date}}` replaces with the date the content was last modified (the marble, not the html itself)
- `{{music}}` replaces with an unordered list of your album directory names in a designated music path
- `{{latest}}` replaces with a given number of the latest updated pages in an unordered list

I'm also planning on adding several more niche elements to the marble format, so that list above will grow over time.

Pillar also does some html substitutions to make templates more interesting
- `{{content}}` is replaced with the content of a give marble page, parsed into html (sort of necessary)
- `{{date}}` has the same behaviour as in marble

## Examples

```
### This is an h3
##		this is an h2
	# this is an h1
```
renders to
```
<h3>This is an h3</h3>
<h2>this is an h2</h2>
<h1>this is an h1</h1>
```
---

```
this is a normal paragraph
this is too, but with *italics*
```
renders to
```
<p>this is a normal paragraph</p>
<p>this is too, but with <em>italics</em></p>
```
---

```
- an unordered list
	- still unordered
			- unordered, but indented twice!
- unordered and back to normal 
~ this is an ordered list
~ it has the same abilities as the unordered list
```
renders to
```
<ul>
<li>an unordered list</li>
<ul>
<li>still unordered</li>
<ul>
<ul>
<li>unordered, but indented twice!</li>
</ul>
</ul>
</ul>
<li>unordered and back to normal </li>
</ul>
<ol>
<li>this is an ordered list</li>
<li>it has the same abilities as the unordered list</li>
</ol>
```
---

```
> This is a simple blockquote, nothing too fancy here
!code!
let mut this = "an actual code block";
this = ["this is", &this].concat();
	// here's some indentation
!code!
```
renders to
```
<blockquote>
<p>This is a simple blockquote, nothing too fancy here</p>
</blockquote>
<pre><code>
let mut this = "an actual code block";
this = ["this is", &this].concat();
	// here's some indentation
</code></pre>
```

Lot's of updates and improvements to come!


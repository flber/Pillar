# Pillar
A simple site generator with template support, written in plain Rust with no dependencies.

Soon to be used in [[my site]](https://mineralexistence.com)

A nice little demo is available in the `examples/` folder of this repo, which shows off Pillar's features (and honestly is mostly a place for me to test and debug)

Pillar uses the marble markup format, which is quite similar to markdown but with a few changes:
- Like markdown, headers are defined with `#`. Marble has support for header levels 1 to 3 (because when are you really using anything smaller?)
- Italics are also defined with `*` (it's not a perfect implementation, as it sometimes leaves `</em>` if a line only has one `*`, but that's a problem for a bit later)
- Images and links follow normal markdown syntax
- Unordered lists still use `-`, but it's far less picky about initial indentation than some markdown parsers (I'm looking at you cmark) and supports weird changes in indentation
- Ordered lists just use `~`, so no need for manually numbering your lists. Same indentation support as ordered lists.
- Blockquotes use the same syntax as markdown, a `>` with as much or as little whitespace before and after as you want.

I'm also planning on adding several more niche elements to the marble format, so that list above will grow over time.

Pillar also does some html substitutions to make templates more interesting
- {{content}} is replaced with the content of a give marble page, parsed into html (sort of necessary)

Lot's of updates and improvements to come!


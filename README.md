# Pillar
A simple site generator with template support, written in plain Rust with no extensions.

Soon to be used in [[my site]](https://mineralexistence.com)

Pillar uses the marble markup format, which is quite similar to markdown but with a few changes:
- Like markdown, headers are defined with `#`. Marble has support for header levels 1 to 3 (because when are you really using anything smaller?)
- Italics are also defined with `*` (it's not a perfect implementation, as it sometimes leaves `</em>` if a line only has one `*`, but that's a problem for a bit later)
- Images and links follow normal markdown syntax
- Unordered lists still use `-`, but it's far less picky about initial indentation than some markdown parsers (I'm looking at you cmark) and supports weird changes in indentation
- Ordered lists just use `~`, so no need for manually numbering your lists. Same indentation support as ordered lists.

I'm also planning on adding several more niche elements to the marble format, so that list will grow over time.

Right now it just parses marble files but isn't really a site generator, so the next couple of update will be implementing templates. After that I'll be alternating between extending marble and adding substitutions in the template html.

Lot's of updates and improvements to come!


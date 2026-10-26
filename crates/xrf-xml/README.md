# xrf-xml

Shared XML reading, source ranges, encoding detection, and output for X-Ray tools.

This crate owns XML mechanics. Translation languages, dialog schemas, texture descriptions, file access, and staged
writes belong to their consuming crates. The public API is exported from `xrf_xml`; parser dependencies stay internal.

## Choose a document

- `XmlDocument` owns a semantic tree of `XmlElement`s detached from the input. Use it to inspect elements, attributes,
  and text, as the texture-description and sound-reference readers do.
- `XmlSourceDocument` owns decoded source text together with `XmlElementSpan` ranges. Use it when the original
  spelling, comments, and whitespace must remain available to an editor, as the translation and dialog crates do.

Both expose case-sensitive local names and attributes in source order. Namespace identities are discarded. `text()`
concatenates direct text children, including CDATA and resolved entities; text inside child elements is excluded.
Neither tree is a complete representation for re-serializing an existing document.

Both refuse a document whose elements nest deeper than `XmlParseOptions::MAX_DEPTH` (32; shipped game data nests eight
at most). The bound is measured on the text before parsing, because the parser and both trees recurse once per level,
and a deeper document would otherwise overflow the stack and abort the process rather than fail to parse.

## Read XML

```rust
use xrf_xml::{XmlDocument, XmlParseOptions};

fn main() -> xrf_error::XrfResult {
  let document: XmlDocument = XmlDocument::parse(
    "<root><sound>weapons\\shot</sound><group><sound>ambient\\wind</sound></group></root>",
    XmlParseOptions::default(),
  )?;

  let references: Vec<&str> = document.elements_named("sound").map(|element| element.text()).collect();

  assert_eq!(references, ["weapons\\shot", "ambient\\wind"]);

  Ok(())
}
```

`elements_named` includes the root and walks descendants in document order. On an element, `children_named` visits
direct children; `descendants` and `descendants_named` exclude the element itself. DTD processing is disabled by default
and can be enabled through `XmlParseOptions::allow_dtd`. The nesting bound does not reach inside an entity a DTD
declares: elements an entity expands to are nested as the parser expands them.

`XmlDocument::parse_bytes` decodes by the document's declaration and defaults to UTF-8. `XmlEncoding::find_declared`
looks for the declaration where it may stand: after an optional UTF-8 byte order mark, blanks, and the comment banners
shipped mods write above it, so one quoted in a comment or named by another processing instruction never counts.
Declared labels support UTF-8 and Windows-1250/1251/1252, including `cp1250`/`cp1251`/`cp1252` aliases, which
`XmlEncoding::from_label` resolves for a label already in hand. The decoder handles and strips a BOM; this API does not
retain the original encoding or BOM for writing. Translation and dialog readers supply their own legacy fallback and
retain encoding metadata for edits.

## Retain source ranges

```rust
use xrf_xml::{XmlEscape, XmlSourceDocument};

fn main() -> xrf_error::XrfResult {
  let document: XmlSourceDocument =
    XmlSourceDocument::parse("<root><!-- keep this --><text>old &amp; new</text></root>".to_owned())?;

  let text = document.root().child_named("text").expect("text element");
  assert_eq!(text.text(), "old & new");

  let range = text.content_range().expect("paired tags").clone();
  assert_eq!(&document.source()[range.clone()], "old &amp; new");

  let mut edited: String = document.into_source();
  edited.replace_range(range, &XmlEscape::text("replacement & text"));
  assert_eq!(edited, "<root><!-- keep this --><text>replacement &amp; text</text></root>");

  Ok(())
}
```

Ranges address bytes in the owned UTF-8 string, not offsets in a legacy-encoded file. `element_range()` includes both
tags; `content_range()` includes everything between them, including comments and child markup. An empty paired element
has an empty content range, while a self-closing element has none. The start tag's end skips a `>` inside a quoted
attribute value, and the closing tag may carry a prefix or blanks before its `>`. Replacing content removes everything
in that range. Apply multiple edits from highest offset to lowest, then validate the result before encoding and
publishing it.

`XmlSourceDocument::parse` takes text already decoded; decode the file's bytes first, using `XmlEncoding::find_declared`
and the fallback your format needs. It always refuses a DTD, since an element an entity expands has its range inside
the declaration rather than where it reads. It tries strict parsing first. On failure it retries a same-length copy that
neutralizes comment banners (`--` inside a comment), bare ampersands and an XML declaration that does not open the
document, which shipped mods write below a banner. Ranges stay valid because no byte moves, and `source()` and
`into_source()` always return the original decoded text. A value whose source the repair changed is read again from
that original, so a bare `&` reads as written and CDATA keeps its dashes; every other value is the parser's own.

## Expand includes

The engine splices `#include "<name>"` lines into a document before parsing it, and shipped character descriptions take
most of their content that way. `XmlIncludes::expand` does the same over bytes, with a reader for each name: the engine
opens names under `$game_config$`, so a game-tree reader maps `gameplay\x.xml` to `configs\gameplay\x.xml`. Included
files lose their byte order mark and declaration, and the including document's declaration decodes the result.
`XmlIncludes::list` names a document's includes without reading them, which tells a file that only exists to be
included from one that stands alone.

```rust
use xrf_xml::XmlIncludes;

fn main() -> xrf_error::XrfResult {
  let expanded: Vec<u8> = XmlIncludes::expand(b"<root>\n#include \"part.xml\"\n</root>", &mut |name| {
    assert_eq!(name, "part.xml");

    Ok(b"<part/>\n".to_vec())
  })?;

  assert_eq!(expanded, b"<root>\n<part/>\n</root>");

  Ok(())
}
```

A document with no `#include` is returned as written. An included file that does not end in a newline is followed by
CRLF, as the engine writes every line it copies with a line break. A malformed directive (an unquoted name, or one
outside 1 to 1024 bytes), a name the document's encoding cannot decode, a missing include, or nesting past 128 levels is
an error, as each is fatal to the engine's loaders. `XmlIncludes::list` skips malformed directives instead.

## Generate XML

`XmlSerializer::serialize` renders a Serde model with two-space indentation, LF newlines, and expanded empty elements.
It adds no XML declaration and performs no byte encoding or file writes.

```rust
use serde::Serialize;
use xrf_xml::XmlSerializer;

#[derive(Serialize)]
#[serde(rename = "root")]
struct Root {
  value: String,
}

fn main() -> xrf_error::XrfResult {
  let output: String = XmlSerializer::serialize(&Root { value: String::new() })?;

  assert_eq!(output, "<root>\n  <value></value>\n</root>");

  Ok(())
}
```

`XmlEscape::text` escapes `&`, `<`, and the `>` of a `]]>`, leaving every other `>`, quote and apostrophe raw as
shipped data writes them. `XmlEscape::attribute` also escapes `"` for double-quoted attributes, and writes tabs and line
breaks as character references, which a parser would otherwise read back as spaces. Pass raw values: an existing entity
is escaped again. These helpers do not validate XML characters.

## Errors and checks

The API returns `XrfResult`. Malformed XML, nesting past the bound, and a refused DTD produce `XrfError::Parsing`;
unsupported encoding labels and bytes that do not decode produce `XrfError::Encoding`; Serde failures produce
`XrfError::Serialization`.

Run `cargo test -p xrf-xml --locked` from the workspace root; the README examples are included in the crate's doctests.

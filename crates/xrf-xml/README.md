# xrf-xml

Shared XML reading, source ranges, encoding detection, include expansion and output for X-Ray tools.

This crate owns XML mechanics only. Translation languages, dialog schemas, texture descriptions, file access and staged
writes belong to the crates that use it. The public API is exported from `xrf_xml`; parser dependencies stay internal.

## Choose a document

| Type                | Holds                                            | Use when                                                                               |
| ------------------- | ------------------------------------------------ | -------------------------------------------------------------------------------------- |
| `XmlDocument`       | A tree of `XmlElement`s, detached from the input | Reading elements, attributes and text (texture descriptions, sound references)         |
| `XmlSourceDocument` | Decoded source text plus `XmlElementSpan` ranges | An editor needs the original spelling, comments and whitespace (translations, dialogs) |

Both:

- expose case-sensitive local names and attributes in source order; namespace identities are discarded;
- return from `text()` the direct text children concatenated, including CDATA and resolved entities, but not text inside
  child elements;
- are not complete enough to re-serialize an existing document;
- refuse elements nested deeper than `XmlParseOptions::MAX_DEPTH` (32; shipped game data nests eight at most).

The depth is measured on the text before parsing, because the parser and both trees recurse once per level; a deeper
document would otherwise overflow the stack and abort the process instead of failing to parse.

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

Navigation:

- `elements_named` includes the root and walks descendants in document order.
- On an element, `children_named` visits direct children; `descendants` and `descendants_named` exclude the element
  itself.

DTDs are disabled by default; `XmlParseOptions::allow_dtd` enables them. The nesting limit does not apply inside an
entity a DTD declares: its elements nest as the parser expands them.

### Encodings

`XmlDocument::parse_bytes` decodes by the document's declaration, defaulting to UTF-8.

- `XmlEncoding::find_declared` looks for the declaration only where one may stand: after an optional UTF-8 byte order
  mark, blank space, and the comment banners shipped mods put above it. A declaration quoted in a comment, or named by
  another processing instruction, never counts.
- Supported labels: UTF-8 and Windows-1250, 1251 and 1252, including the `cp1250`, `cp1251` and `cp1252` aliases.
  `XmlEncoding::from_label` resolves a label already in hand.
- The decoder handles and strips a BOM. This API does not keep the original encoding or BOM for writing; the
  translation and dialog readers supply their own fallback and keep that metadata for edits.

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

Ranges are byte offsets into the owned UTF-8 string, not into a legacy-encoded file.

- `element_range()` includes both tags.
- `content_range()` includes everything between them, comments and child markup included. An empty paired element has
  an empty content range; a self-closing element has none.
- The start tag's end skips a `>` inside a quoted attribute value, and a closing tag may carry a prefix or blanks before
  its `>`.
- Replacing content removes everything in the range. Apply several edits from the highest offset down, then validate
  the result before encoding and publishing it.

### Parsing tolerantly

`XmlSourceDocument::parse` takes text already decoded: decode the file's bytes first, with
`XmlEncoding::find_declared` and the fallback your format needs.

- It always refuses a DTD, since an element expanded from an entity would have its range inside the declaration.
- It tries strict parsing first. If that fails, it retries on a same-length copy that neutralizes comment banners (`--`
  inside a comment), bare ampersands, and an XML declaration that does not open the document, all of which shipped mods
  write.
- No byte moves, so ranges stay valid, and `source()` and `into_source()` always return the original text.
- A value whose source the repair changed is read again from the original, so a bare `&` reads as written and CDATA
  keeps its dashes. Every other value is the parser's own.

## Expand includes

The engine splices `#include "<name>"` lines into a document before parsing it; shipped character descriptions get most
of their content that way. `XmlIncludes::expand` does the same over bytes, calling a reader for each name.

- The engine opens names under `$game_config$`, so a game-tree reader maps `gameplay\x.xml` to
  `configs\gameplay\x.xml`.
- Included files lose their byte order mark and declaration; the including document's declaration decodes the result.
- `XmlIncludes::list` names a document's includes without reading them, to tell a file that only exists to be included
  from one that stands alone.

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

- A document with no `#include` is returned as written.
- An included file not ending in a newline is followed by CRLF, as the engine writes every copied line with a break.
- These are errors, as each is fatal to the engine's loaders: a malformed directive (an unquoted name, or one outside 1
  to 1024 bytes), a name the document's encoding cannot decode, a missing include, or nesting past 128 levels.
- `XmlIncludes::list` skips malformed directives instead.

## Generate XML

`XmlSerializer::serialize` renders a Serde model with two-space indentation, LF newlines and expanded empty elements. It
adds no XML declaration and does no byte encoding or file writing.

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

Escaping:

- `XmlEscape::text` escapes `&`, `<`, and the `>` of a `]]>`. Every other `>`, quote and apostrophe stays raw, as
  shipped data writes them.
- `XmlEscape::attribute` also escapes `"` for double-quoted attributes, and writes tabs and line breaks as character
  references, which a parser would otherwise read back as spaces.
- Pass raw values: an existing entity is escaped again. These helpers do not validate XML characters.

## Errors

| Error                     | Cause                                                      |
| ------------------------- | ---------------------------------------------------------- |
| `XrfError::Parsing`       | Malformed XML, nesting past the limit, a refused DTD       |
| `XrfError::Encoding`      | An unsupported encoding label, or bytes that do not decode |
| `XrfError::Serialization` | A Serde failure                                            |

## Checks

Run `cargo test --locked -p xrf-xml` from the repository root.

# xrf-xml

Shared XML reading, source ranges, encoding detection, and output for X-Ray tools.

This crate owns XML mechanics. Translation languages, dialog schemas, texture descriptions, file access, and staged
writes belong to their consuming crates. The public API is exported from `xrf_xml`; parser dependencies stay internal.

## Choose a document

- `XmlDocument` owns a semantic tree detached from the input. Use it to inspect elements, attributes, and text.
- `XmlSourceDocument` owns decoded source text together with element ranges. Use it when the original spelling,
  comments, and whitespace must remain available to an editor.

Both expose case-sensitive local names and attributes in source order. Namespace identities are discarded. `text()`
concatenates direct text children, including CDATA and resolved entities; text inside child elements is excluded.
Neither tree is a complete representation for re-serializing an existing document.

## Read XML

```rust
use xrf_xml::{XmlDocument, XmlParseOptions};

# fn main() -> xrf_error::XrfResult {
    let document: XmlDocument = XmlDocument::parse(
        "<root><sound>weapons\\shot</sound><group><sound>ambient\\wind</sound></group></root>",
        XmlParseOptions::default(),
    )?;

    let references: Vec<&str> = document.elements_named("sound").map(|element| element.text()).collect();
    assert_eq!(references, ["weapons\\shot", "ambient\\wind"]);
    # Ok(())
    #
}
```

`elements_named` includes the root and walks descendants in document order. On an element, `children_named` visits
direct children; `descendants` and `descendants_named` exclude the element itself. DTD processing is disabled by default
and can be enabled through `XmlParseOptions::allow_dtd`.

`XmlDocument::parse_bytes` detects a declaration in the first 256 bytes and defaults to UTF-8. Declared labels support
UTF-8 and Windows-1250/1251/1252, including `cp1250`/`cp1251`/`cp1252` aliases. The underlying decoder handles and
strips a BOM; this API does not retain the original encoding or BOM for writing. `declared_xml_encoding` inspects the
declaration without decoding the body, and `encoding_from_label` resolves a label already in hand. Translation and
dialog readers supply their own legacy fallback and retain encoding metadata for edits.

## Retain source ranges

```rust
use xrf_xml::{XmlParseOptions, XmlSourceDocument, escape_xml_text};

# fn main() -> xrf_error::XrfResult {
    let document: XmlSourceDocument = XmlSourceDocument::parse(
        "<root><!-- keep this --><text>old &amp; new</text></root>".to_owned(),
        XmlParseOptions::default(),
    )?;

    let text = document.root().child_named("text").expect("text element");
    assert_eq!(text.text(), "old & new");
    let range = text.content_range().expect("paired tags").clone();
    assert_eq!(&document.source()[range.clone()], "old &amp; new");

    let mut edited: String = document.into_source();
    edited.replace_range(range, &escape_xml_text("replacement & text"));
    assert_eq!(edited, "<root><!-- keep this --><text>replacement &amp; text</text></root>");
    # Ok(())
    #
}
```

Ranges address bytes in the owned UTF-8 string, not offsets in a legacy-encoded file. `element_range()` includes both
tags; `content_range()` includes everything between them, including comments and child markup. An empty paired element
has an empty content range, while a self-closing element has none. Replacing content removes everything in that range.
Apply multiple edits from highest offset to lowest, then validate the result before encoding and publishing it.

The source reader tries strict parsing first. On failure it retries a same-length copy that neutralizes comment banners
and bare ampersands. `source()` and `into_source()` always return the original decoded text.

## Generate XML

`serialize_xml` renders a Serde model with two-space indentation, LF newlines, and expanded empty elements. It adds no
XML declaration and performs no byte encoding or file writes.

```rust
use serde::Serialize;
use xrf_xml::serialize_xml;

#[derive(Serialize)]
#[serde(rename = "root")]
struct Root {
    value: String,
}

# fn main() -> xrf_error::XrfResult {
    let output: String = serialize_xml(&Root { value: String::new() })?;
    assert_eq!(output, "<root>\n  <value></value>\n</root>");
    # Ok(())
    #
}
```

`escape_xml_text` escapes `&` and `<`. `escape_xml_attribute` also escapes `"` for double-quoted attributes. Pass raw
values: an existing entity is escaped again. These helpers do not validate XML characters.

## Errors and checks

The API returns `XrfResult`. Malformed XML produces `XrfError::Parsing`, unsupported encoding labels produce
`XrfError::Encoding`, and Serde failures produce `XrfError::Serialization`. Invalid encoded bytes currently propagate as
`XrfError::Io` from the shared decoding helper.

Run `cargo test -p xrf-xml --locked` from the workspace root; the README examples are included in the crate's doctests.

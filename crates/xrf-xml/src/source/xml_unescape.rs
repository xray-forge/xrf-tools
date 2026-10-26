/// Values read straight from the source text, for the nodes a repaired copy parsed differently.
///
/// A repaired copy replaces each bare ampersand with `~` so a strict parser accepts it, which is what keeps its ranges
/// addressing the original bytes. The values that parser resolves would carry those tildes, so a node whose source the
/// repair changed is read again from the original instead: references resolved, a bare ampersand kept as written, as
/// the engine's own reader keeps it.
pub(crate) struct XmlUnescape;

impl XmlUnescape {
  const CDATA_START: &'static str = "<![CDATA[";
  const CDATA_END: &'static str = "]]>";

  /// The text a run of character data stands for: references resolved, CDATA kept as written, line ends normalized.
  pub fn text(raw: &str) -> String {
    let normalized: String = Self::normalize_line_ends(raw);
    let mut text: String = String::with_capacity(normalized.len());
    let mut rest: &str = &normalized;

    while let Some(start) = rest.find(Self::CDATA_START) {
      text.push_str(&Self::resolve_references(&rest[..start]));

      let body: &str = &rest[start + Self::CDATA_START.len()..];

      match body.find(Self::CDATA_END) {
        Some(end) => {
          text.push_str(&body[..end]);
          rest = &body[end + Self::CDATA_END.len()..];
        }
        None => {
          text.push_str(body);
          rest = "";
        }
      }
    }

    text.push_str(&Self::resolve_references(rest));
    text
  }

  /// The value an attribute stands for: references resolved, and literal whitespace normalized to spaces as XML does.
  pub fn attribute(raw: &str) -> String {
    // Normalized before references resolve, so a character reference to a line break survives as one.
    let normalized: String = Self::normalize_line_ends(raw).replace(['\t', '\n'], " ");

    Self::resolve_references(&normalized)
  }

  /// `\r\n` and a lone `\r` read as `\n`, as an XML parser hands them on.
  fn normalize_line_ends(raw: &str) -> String {
    raw.replace("\r\n", "\n").replace('\r', "\n")
  }

  /// Resolves the five predefined entities and numeric references, keeping anything else, a bare `&` included, as is.
  fn resolve_references(raw: &str) -> String {
    let mut resolved: String = String::with_capacity(raw.len());
    let mut rest: &str = raw;

    while let Some(start) = rest.find('&') {
      resolved.push_str(&rest[..start]);
      rest = &rest[start..];

      match rest
        .find(';')
        .and_then(|end| Some((Self::resolve_reference(&rest[1..end])?, end)))
      {
        Some((character, end)) => {
          resolved.push(character);
          rest = &rest[end + 1..];
        }
        None => {
          resolved.push('&');
          rest = &rest[1..];
        }
      }
    }

    resolved.push_str(rest);
    resolved
  }

  /// The character a reference's name between `&` and `;` stands for, if it is one the reader would resolve.
  fn resolve_reference(name: &str) -> Option<char> {
    match name {
      "amp" => Some('&'),
      "lt" => Some('<'),
      "gt" => Some('>'),
      "quot" => Some('"'),
      "apos" => Some('\''),
      _ => {
        let code: u32 = match name.strip_prefix('#')? {
          hex if hex.starts_with(['x', 'X']) => u32::from_str_radix(&hex[1..], 16).ok()?,
          decimal => decimal.parse().ok()?,
        };

        char::from_u32(code)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::source::xml_unescape::XmlUnescape;

  #[test]
  fn keeps_a_bare_ampersand_and_resolves_references() {
    assert_eq!(
      XmlUnescape::text("Tom & Jerry &amp; co &#126; &#x41;"),
      "Tom & Jerry & co ~ A"
    );
  }

  #[test]
  fn keeps_an_unknown_entity_as_written() {
    assert_eq!(XmlUnescape::text("&nbsp; and &; and &#zz;"), "&nbsp; and &; and &#zz;");
  }

  #[test]
  fn keeps_cdata_as_written() {
    assert_eq!(XmlUnescape::text("a & <![CDATA[b &amp; c]]> &lt;"), "a & b &amp; c <");
  }

  #[test]
  fn normalizes_line_ends_in_text() {
    assert_eq!(XmlUnescape::text("a\r\nb\rc"), "a\nb\nc");
  }

  #[test]
  fn normalizes_attribute_whitespace_but_not_referenced_breaks() {
    assert_eq!(XmlUnescape::attribute("a\tb\r\nc &#10; d & e"), "a b c \n d & e");
  }
}

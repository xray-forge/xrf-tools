use quick_xml::se::Serializer;
use serde::Serialize;
use xrf_error::{XrfError, XrfResult};

/// Writes a Serde model as a new XML document in the layout the engine's own files use.
pub struct XmlSerializer;

impl XmlSerializer {
  /// Spaces each nesting level is indented by.
  const INDENT: usize = 2;

  /// Serialize a Serde model as indented XML with expanded empty elements.
  ///
  /// Empty elements are expanded rather than self-closed because the engine's own files write them
  /// that way, and generated output that matches what ships is easier to compare against it.
  ///
  /// # Errors
  ///
  /// Returns a serialization error when the model cannot be represented as XML.
  pub fn serialize<T>(value: &T) -> XrfResult<String>
  where
    T: Serialize,
  {
    let mut output: String = String::new();
    let mut serializer: Serializer<String> = Serializer::new(&mut output);

    serializer.expand_empty_elements(true);
    serializer.indent(' ', Self::INDENT);
    value
      .serialize(serializer)
      .map_err(|error| XrfError::new_serialization_error(error.to_string()))?;

    Ok(output)
  }
}

#[cfg(test)]
mod tests {
  use serde::Serialize;

  use crate::write::XmlSerializer;

  #[derive(Debug, Serialize)]
  #[serde(rename = "root")]
  struct Fixture {
    value: String,
  }

  #[test]
  fn serializes_an_indented_document() {
    let serialized: String = XmlSerializer::serialize(&Fixture {
      value: String::from("content"),
    })
    .unwrap();

    assert_eq!(serialized, "<root>\n  <value>content</value>\n</root>");
  }

  #[test]
  fn expands_an_empty_element_rather_than_self_closing_it() {
    let serialized: String = XmlSerializer::serialize(&Fixture { value: String::new() }).unwrap();

    assert_eq!(serialized, "<root>\n  <value></value>\n</root>");
  }
}

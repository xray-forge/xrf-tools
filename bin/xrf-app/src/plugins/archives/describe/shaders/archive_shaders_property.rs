use std::collections::HashMap;

use serde::Serialize;
use xrf_db::{ShaderBlenderProperty, ShaderBlenderPropertyKind, ShaderBlenderPropertyValue, ShaderBlenderToken};
use xrf_vfs::XrayAssetType;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;

/// The character an engine slot name begins with, against a texture name that addresses a file.
const SLOT_PREFIX: char = '$';

/// One value an author left in a blender's property grid.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveShadersProperty {
  pub name: String,
  /// The type the file tags the payload with, as `xrEngine/Properties.h` names it.
  pub kind: String,
  /// The value as a reader would compare it, empty for a marker, which carries none.
  pub value: String,
  /// The texture a texture property names, when it names a file rather than a slot the renderer binds.
  pub texture: Option<ArchiveReference>,
}

impl ArchiveShadersProperty {
  /// Every property of one blender, with texture names resolved through a lookup shared across the library.
  pub fn of_all(
    source: &ArchiveDescribeSource,
    properties: &[ShaderBlenderProperty],
    resolved: &mut HashMap<String, ArchiveReference>,
  ) -> Vec<Self> {
    properties
      .iter()
      .map(|property| Self {
        name: property.name.clone(),
        kind: Self::to_kind_label(property.value.kind()).to_owned(),
        value: Self::to_value(&property.value),
        texture: Self::to_texture(source, &property.value, resolved),
      })
      .collect()
  }

  /// The engine's own name for a payload type.
  const fn to_kind_label(kind: ShaderBlenderPropertyKind) -> &'static str {
    match kind {
      ShaderBlenderPropertyKind::Marker => "Marker",
      ShaderBlenderPropertyKind::Matrix => "Matrix",
      ShaderBlenderPropertyKind::Constant => "Constant",
      ShaderBlenderPropertyKind::Texture => "Texture",
      ShaderBlenderPropertyKind::Integer => "Integer",
      ShaderBlenderPropertyKind::Float => "Float",
      ShaderBlenderPropertyKind::Bool => "Bool",
      ShaderBlenderPropertyKind::Token => "Token",
      ShaderBlenderPropertyKind::ClassId => "Class id",
      ShaderBlenderPropertyKind::Object => "Object",
      ShaderBlenderPropertyKind::Text => "Text",
    }
  }

  /// A payload as one line of text.
  ///
  /// A bounded number keeps its range: the editor's clamp is what says whether an authored value sits at the end of
  /// what the class accepts, which the number alone does not.
  fn to_value(value: &ShaderBlenderPropertyValue) -> String {
    match value {
      ShaderBlenderPropertyValue::Marker => String::new(),
      ShaderBlenderPropertyValue::Matrix(text)
      | ShaderBlenderPropertyValue::Constant(text)
      | ShaderBlenderPropertyValue::Texture(text)
      | ShaderBlenderPropertyValue::Object(text)
      | ShaderBlenderPropertyValue::Text(text) => text.clone(),
      ShaderBlenderPropertyValue::Integer {
        value,
        minimum,
        maximum,
      } => format!("{value} ({minimum} to {maximum})"),
      ShaderBlenderPropertyValue::Float {
        value,
        minimum,
        maximum,
      } => format!("{value} ({minimum} to {maximum})"),
      ShaderBlenderPropertyValue::Bool(value) => String::from(if *value { "true" } else { "false" }),
      ShaderBlenderPropertyValue::Token { selected, items } => Self::to_token_value(*selected, items),
      ShaderBlenderPropertyValue::ClassId { selected, items } => {
        format!("{selected} of {} classes", items.len())
      }
    }
  }

  /// The label a token property selected, falling back to the number for a selection nothing in the list carries.
  fn to_token_value(selected: u32, items: &[ShaderBlenderToken]) -> String {
    items
      .iter()
      .find(|token| token.id == selected)
      .map_or_else(|| selected.to_string(), |token| token.label.clone())
  }

  /// The file a texture property names, or `None` for a property that is not one or names a slot.
  fn to_texture(
    source: &ArchiveDescribeSource,
    value: &ShaderBlenderPropertyValue,
    resolved: &mut HashMap<String, ArchiveReference>,
  ) -> Option<ArchiveReference> {
    let name: &String = match value {
      ShaderBlenderPropertyValue::Texture(name) => name,
      _ => return None,
    };

    if name.is_empty() || name.starts_with(SLOT_PREFIX) {
      return None;
    }

    Some(
      resolved
        .entry(name.clone())
        .or_insert_with(|| ArchiveReference::resolve(source, XrayAssetType::Dds, name))
        .clone(),
    )
  }
}

#[cfg(test)]
mod tests {
  use xrf_db::{ShaderBlenderPropertyValue, ShaderBlenderToken};

  use super::ArchiveShadersProperty;

  #[test]
  fn a_bounded_number_keeps_the_range_the_editor_clamps_it_to() {
    assert_eq!(
      ArchiveShadersProperty::to_value(&ShaderBlenderPropertyValue::Integer {
        value: 4,
        minimum: 0,
        maximum: 8,
      }),
      "4 (0 to 8)"
    );
  }

  #[test]
  fn a_token_reads_as_the_label_it_selected() {
    let value: ShaderBlenderPropertyValue = ShaderBlenderPropertyValue::Token {
      selected: 2,
      items: vec![
        ShaderBlenderToken {
          id: 1,
          label: String::from("Opaque"),
        },
        ShaderBlenderToken {
          id: 2,
          label: String::from("Blend"),
        },
      ],
    };

    assert_eq!(ArchiveShadersProperty::to_value(&value), "Blend");
  }

  #[test]
  fn a_selection_no_token_carries_reads_as_the_number_it_is() {
    let value: ShaderBlenderPropertyValue = ShaderBlenderPropertyValue::Token {
      selected: 9,
      items: Vec::new(),
    };

    assert_eq!(ArchiveShadersProperty::to_value(&value), "9");
  }

  #[test]
  fn a_marker_carries_no_value_at_all() {
    assert_eq!(
      ArchiveShadersProperty::to_value(&ShaderBlenderPropertyValue::Marker),
      ""
    );
  }
}

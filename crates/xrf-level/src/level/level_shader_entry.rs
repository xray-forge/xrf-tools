use serde::{Deserialize, Serialize};

/// Single entry of the level shader table.
///
/// The renderer stores each entry as `shader_name/texture,texture,...`, skips entries with an empty
/// name, and splits the remainder on the first `/`. Entries without the delimiter are represented
/// separately because the engine dereferences the result of `strchr` without a null check, so such
/// an entry is a crash rather than a resolvable reference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum LevelShaderEntry {
  /// Entry with an empty name, skipped by the renderer.
  Empty,
  /// Entry without the `/` delimiter, unusable by the renderer.
  Malformed(String),
  /// Resolvable shader reference with its texture list.
  Reference(LevelShaderReference),
}

/// Parsed `shader_name/texture,texture,...` level shader table entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelShaderReference {
  pub shader: String,
  pub textures: Vec<String>,
}

impl LevelShaderEntry {
  /// Parse raw level shader table entry as stored in the chunk.
  pub fn parse(raw: &str) -> Self {
    if raw.is_empty() {
      return Self::Empty;
    }

    match raw.split_once('/') {
      None => Self::Malformed(String::from(raw)),
      Some((shader, textures)) => Self::Reference(LevelShaderReference {
        shader: String::from(shader),
        textures: textures
          .split(',')
          .map(str::trim)
          .filter(|texture| !texture.is_empty())
          .map(String::from)
          .collect(),
      }),
    }
  }

  /// Render entry back into its raw chunk representation.
  pub fn to_raw(&self) -> String {
    match self {
      Self::Empty => String::new(),
      Self::Malformed(raw) => raw.clone(),
      Self::Reference(reference) => {
        format!("{}/{}", reference.shader, reference.textures.join(","))
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::level::level_shader_entry::{LevelShaderEntry, LevelShaderReference};

  #[test]
  fn parses_empty_entry_as_skipped_by_renderer() {
    assert_eq!(LevelShaderEntry::parse(""), LevelShaderEntry::Empty);
  }

  #[test]
  fn parses_entry_without_delimiter_as_malformed() {
    assert_eq!(
      LevelShaderEntry::parse("def_shaders"),
      LevelShaderEntry::Malformed(String::from("def_shaders"))
    );
  }

  #[test]
  fn parses_shader_and_texture_list() {
    assert_eq!(
      LevelShaderEntry::parse("def_shaders\\def_vertex/prop\\prop_fence,detail\\dirt"),
      LevelShaderEntry::Reference(LevelShaderReference {
        shader: String::from("def_shaders\\def_vertex"),
        textures: vec![String::from("prop\\prop_fence"), String::from("detail\\dirt"),],
      })
    );
  }

  #[test]
  fn parses_shader_without_textures() {
    assert_eq!(
      LevelShaderEntry::parse("def_shaders\\def_vertex/"),
      LevelShaderEntry::Reference(LevelShaderReference {
        shader: String::from("def_shaders\\def_vertex"),
        textures: Vec::new(),
      })
    );
  }

  #[test]
  fn renders_entries_back_into_raw_chunk_representation() {
    for raw in ["", "no_delimiter", "def_shaders\\def_vertex/prop\\fence,dirt"] {
      assert_eq!(LevelShaderEntry::parse(raw).to_raw(), raw);
    }
  }
}

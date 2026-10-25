use serde::Serialize;
use xrf_level::{LevelFile, LevelShaderEntry};

/// The shader table, counted by what each entry is rather than listed whole: every entry is a reference, empty or
/// malformed, so the three add up to the table.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelShadersReport {
  references: usize,
  lightmapped: usize,
  /// Entries the renderer skips because they name nothing.
  empty: usize,
  /// Entries without the `/` delimiter, which the renderer dereferences without a null check.
  malformed: usize,
}

impl LevelShadersReport {
  pub fn of(level: &LevelFile) -> Self {
    let entries: &[LevelShaderEntry] = match &level.shaders {
      Some(shaders) => &shaders.entries,
      None => &[],
    };

    Self {
      empty: entries
        .iter()
        .filter(|it| matches!(it, LevelShaderEntry::Empty))
        .count(),
      lightmapped: entries
        .iter()
        .filter(|it| matches!(it, LevelShaderEntry::Reference(reference) if reference.textures.len() > 1))
        .count(),
      malformed: entries
        .iter()
        .filter(|it| matches!(it, LevelShaderEntry::Malformed(_)))
        .count(),
      references: entries
        .iter()
        .filter(|it| matches!(it, LevelShaderEntry::Reference(_)))
        .count(),
    }
  }
}

use serde::Serialize;
use xrf_ltx::Section;

use crate::finding::EnvironmentRule;
use crate::section::EnvironmentSectionReader;

/// A `thunderbolt_collections.ltx` section, `SThunderboltCollection`: the bolts a keyframe strikes with, one of which
/// the engine picks at random each strike. Each line's key names a bolt; its value is not read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThunderboltCollection {
  pub name: String,
  /// The config, as a logical path.
  pub file: String,
  /// The bolts' `thunderbolts.ltx` sections, in the order written.
  pub thunderbolts: Vec<String>,
}

impl ThunderboltCollection {
  /// Reads one collection's bolt names.
  pub(crate) fn read(reader: &mut EnvironmentSectionReader, name: &str, section: &Section) -> Self {
    let thunderbolts: Vec<String> = section.iter().map(|(bolt, _)| bolt.to_owned()).collect();

    if thunderbolts.is_empty() {
      let message: String = format!(
        "{} names no thunderbolts, and the engine picks one at random for every strike",
        reader.describe(name)
      );

      reader.report(EnvironmentRule::Engine, name, None, message);
    }

    Self {
      file: reader.get_file().to_owned(),
      name: name.to_owned(),
      thunderbolts,
    }
  }
}

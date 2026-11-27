use std::collections::BTreeMap;

use xrf_game_config::ConfigReader;

/// How much a squad of one behaviour wants each kind of target (`misc\squad_behaviours.ltx`): the weights its target
/// priority multiplies a target's simulation props by, such as `resource = 2`, `lair = 0`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SquadBehaviour {
  pub weights: BTreeMap<String, f32>,
}

impl SquadBehaviour {
  /// The behaviour `section` holds; a weight that is no number is an issue and left out.
  pub fn read(section: &mut ConfigReader<'_, '_>) -> Self {
    Self {
      weights: section.read_all(&[]),
    }
  }
}

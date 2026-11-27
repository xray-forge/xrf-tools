use std::collections::BTreeMap;

use xrf_ltx::Section;

/// How much a squad of one behaviour wants each kind of target (`misc\squad_behaviours.ltx`): the weights its target
/// priority multiplies a target's simulation props by, such as `resource = 2`, `lair = 0`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SquadBehaviour {
  pub weights: BTreeMap<String, f32>,
}

impl SquadBehaviour {
  /// The behaviour `section` holds, or why a weight is no number.
  pub fn read(section: &Section) -> Result<Self, String> {
    let weights: BTreeMap<String, f32> = section
      .iter()
      .map(|(key, value)| {
        value
          .trim()
          .parse::<f32>()
          .map(|weight| (String::from(key), weight))
          .map_err(|_| format!("{key} = '{value}' is no number"))
      })
      .collect::<Result<_, _>>()?;

    Ok(Self { weights })
  }
}

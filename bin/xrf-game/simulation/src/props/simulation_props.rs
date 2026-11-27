use std::collections::BTreeMap;

use xrf_ltx::{Condlist, Section};

use crate::config::config_text::ConfigText;

/// What a simulation object offers squads (`misc\simulation_objects_props.ltx`, a section named for a smart terrain,
/// a squad section or the actor): how much of each kind of target it is, such as `resource = 2`, and whether it is a
/// target at all now (`sim_avail`, a condlist, `true` where it names none).
#[derive(Clone, Debug, PartialEq)]
pub struct SimulationProps {
  pub available: Condlist,
  pub values: BTreeMap<String, f32>,
}

impl SimulationProps {
  /// The props `section` holds, or why one is no number or `sim_avail` no condlist.
  pub fn read(section: &Section) -> Result<Self, String> {
    let mut values: BTreeMap<String, f32> = BTreeMap::new();
    let mut available: Condlist = ConfigText::parse_condlist("true")?;

    for (key, value) in section {
      if key == "sim_avail" {
        available = ConfigText::parse_condlist(value)?;
      } else {
        let number: f32 = value
          .trim()
          .parse()
          .map_err(|_| format!("{key} = '{value}' is no number"))?;

        values.insert(String::from(key), number);
      }
    }

    Ok(Self { available, values })
  }
}

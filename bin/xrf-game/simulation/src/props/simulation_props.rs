use std::collections::BTreeMap;

use xrf_game_config::{Condlist, ConfigReader};

/// What a simulation object offers squads (`misc\simulation_objects_props.ltx`, a section named for a smart terrain,
/// a squad section or the actor): how much of each kind of target it is, such as `resource = 2`, and whether it is a
/// target at all now (`sim_avail`, a condlist, `true` where it names none).
#[derive(Clone, Debug, PartialEq)]
pub struct SimulationProps {
  pub available: Condlist,
  pub values: BTreeMap<String, f32>,
}

impl SimulationProps {
  /// The key naming when the object is a target, the one prop that is no number.
  const AVAILABLE_KEY: &'static str = "sim_avail";

  /// The props `section` holds: a value that is no number is an issue and left out; none, said as an issue, where
  /// `sim_avail` is no condlist.
  pub fn read(section: &mut ConfigReader<'_, '_>) -> Option<Self> {
    Some(Self {
      available: section.read_or_parse(Self::AVAILABLE_KEY, "true")?,
      values: section.read_all(&[Self::AVAILABLE_KEY]),
    })
  }
}

use std::collections::BTreeMap;

/// What `level.ltx` says of the scene that built it.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LevelMapSettings {
  /// `[map_usage] ver`.
  pub map_version: Option<String>,
  /// The game types `[map_usage]` names, by their `EGameIDs` bits; single player where it names none.
  pub game_types: u16,
  /// `[level_map] bound_rect`: the lowest x and z, then the highest.
  pub bounds: Option<[f32; 4]>,
  /// `[level_map] texture`.
  pub texture: Option<String>,
  /// `[sub_level_map]`: each sector's part of the map, by the sector's number.
  pub sector_maps: BTreeMap<u16, u8>,
  /// The sections the Build does not write itself, from the build options' text or a later edit, as that text again:
  /// their lines joined by line breaks, `;` and blank ones left out.
  pub build_options: String,
  /// Names `[map_usage]` holds that are no game type the Build writes.
  pub unknown_game_types: Vec<String>,
}

use xrf_ltx::Section;

/// What a `system.ltx` section describes, told by the keys the engine reads for it, which the weapons and animations
/// checks both select their sections by.
pub(crate) struct GamedataItemSection;

impl GamedataItemSection {
  /// Whether the section is a weapon the engine instantiates, by its class keys and the item scheme or flame keys only a
  /// weapon carries.
  pub(crate) fn is_weapon(section: &Section) -> bool {
    section.contains_key("class")
      && section.contains_key("weapon_class")
      && (section.get("$scheme").is_some_and(|it| it == "$item_weapon")
        || section.contains_key("flame_particles")
        || section.contains_key("flame"))
  }

  /// Whether the section describes a hud item, a model attached to the player hands. The engine requires `item_visual`
  /// for every such section, see `attachable_hud_item::load`.
  pub(crate) fn is_hud_item(section: &Section) -> bool {
    section.contains_key("item_visual")
  }

  /// Whether the section describes the player hands, by the anchors the engine places them with.
  pub(crate) fn is_player_hud(section: &Section) -> bool {
    section.contains_key("visual")
      && section.contains_key("position")
      && section.contains_key("orientation")
      && section.contains_key("ancor_0")
      && section.contains_key("ancor_1")
  }

  /// The motion an `anm_*` value names, its first comma-separated token; the rest are its parameters.
  pub(crate) fn get_motion_name(value: &str) -> &str {
    value.split(',').next().unwrap_or(value).trim()
  }
}

#[cfg(test)]
mod tests {
  use super::GamedataItemSection;

  #[test]
  fn names_the_motion_before_its_parameters() {
    assert_eq!(GamedataItemSection::get_motion_name("anm_shots, 1.0"), "anm_shots");
    assert_eq!(GamedataItemSection::get_motion_name(" anm_idle "), "anm_idle");
  }
}

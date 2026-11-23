use crate::host::level::spawn::render_spawn_category::RenderSpawnCategory;

/// The group a spawned object is shown and hidden by: its category, whether a new game releases it, and whether it lies
/// hidden until a detector reveals it. A view hides a group where it hides any one of the three.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RenderSpawnGroup {
  pub category: RenderSpawnCategory,
  pub is_released: bool,
  pub is_detector_hidden: bool,
}

impl RenderSpawnGroup {
  /// Every group, each once.
  pub fn list_all() -> impl Iterator<Item = Self> {
    RenderSpawnCategory::ALL.into_iter().flat_map(|category| {
      [false, true].into_iter().flat_map(move |is_released| {
        [false, true].into_iter().map(move |is_detector_hidden| Self {
          category,
          is_released,
          is_detector_hidden,
        })
      })
    })
  }

  /// Its number, from one, which a scene's hidden groups mark it by: the category's, then four on for released objects
  /// and eight on for detector-hidden ones.
  pub const fn get_index(self) -> u32 {
    let category: u32 = match self.category {
      RenderSpawnCategory::Props => 1,
      RenderSpawnCategory::Items => 2,
      RenderSpawnCategory::Weapons => 3,
      RenderSpawnCategory::Lamps => 4,
    };

    category + if self.is_released { 4 } else { 0 } + if self.is_detector_hidden { 8 } else { 0 }
  }
}

impl From<RenderSpawnCategory> for RenderSpawnGroup {
  /// The group of a category's objects a new game keeps and nothing hides until detected.
  fn from(category: RenderSpawnCategory) -> Self {
    Self {
      category,
      is_released: false,
      is_detector_hidden: false,
    }
  }
}

use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_layout::StaticLayout;

/// One indirect draw a frame: every visible cluster of one layout drawn with one class's program.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StaticBatch {
  pub layout: StaticLayout,
  pub class: StaticClass,
}

impl StaticBatch {
  pub const COUNT: usize = StaticLayout::ALL.len() * StaticClass::ALL.len();

  pub fn get_index(self) -> u32 {
    (self.layout.get_index() * StaticClass::ALL.len() + self.class.get_index()) as u32
  }

  pub fn list() -> impl Iterator<Item = StaticBatch> {
    StaticLayout::ALL.into_iter().flat_map(|layout| {
      StaticClass::ALL
        .into_iter()
        .map(move |class| StaticBatch { layout, class })
    })
  }

  /// The batches drawn into the G-buffer and into shadows.
  pub fn list_deferred() -> impl Iterator<Item = StaticBatch> {
    Self::list().filter(|batch| batch.class.is_deferred())
  }

  /// The batches the water pass draws.
  pub fn list_water() -> impl Iterator<Item = StaticBatch> {
    Self::list().filter(|batch| batch.class == StaticClass::Water)
  }
}

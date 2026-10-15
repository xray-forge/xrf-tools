use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_surface_pass::StaticSurfacePass;

/// One indirect draw a frame: every visible cluster of one layout drawn with one class's program.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StaticBatch {
  pub layout: StaticLayout,
  pub class: StaticClass,
}

impl StaticBatch {
  pub const COUNT: usize = StaticLayout::ALL.len() * StaticClass::ALL.len();

  /// Draw lists the cull fills a frame: one a batch, then one a layout and surface pass, in that order.
  pub const LIST_COUNT: usize = Self::COUNT + StaticLayout::COUNT * StaticSurfacePass::COUNT;

  /// Bytes one draw list's indirect arguments take.
  pub const ARGS_BYTES: u64 = size_of::<wgpu::util::DrawIndirectArgs>() as u64;

  /// Where a draw list's indirect arguments start, by its number.
  pub fn get_list_args_offset(list: u32) -> u64 {
    u64::from(list) * Self::ARGS_BYTES
  }

  /// Where the batch's own draw list's indirect arguments start.
  pub fn get_args_offset(self) -> u64 {
    Self::get_list_args_offset(self.get_index())
  }

  pub fn get_index(self) -> u32 {
    (self.layout.get_index() * StaticClass::ALL.len() + self.class.get_index()) as u32
  }

  /// The draw list of a layout's surfaces taking part in a pass, after every batch's.
  pub fn get_pass_list(layout: StaticLayout, pass: StaticSurfacePass) -> u32 {
    (Self::COUNT + layout.get_index() * StaticSurfacePass::COUNT + pass.get_index()) as u32
  }

  /// Every draw list a slot of this batch is appended to: its own, then each pass's its surface takes part in.
  pub fn list_draw_lists(self, passes: u32) -> impl Iterator<Item = usize> {
    std::iter::once(self.get_index() as usize).chain(
      StaticSurfacePass::ALL
        .into_iter()
        .filter(move |pass| passes & 1 << pass.get_index() != 0)
        .map(move |pass| Self::get_pass_list(self.layout, pass) as usize),
    )
  }

  /// Every batch, each layout with each class, in the order their draw lists are kept.
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

  /// The batches the composited pass draws over the lit frame in the cull's order; the sorted ones it draws from the
  /// view's list.
  pub fn list_composited() -> impl Iterator<Item = StaticBatch> {
    Self::list().filter(|batch| batch.class == StaticClass::Composited)
  }

  /// The batches the composited pass lays into the G-buffer's albedo.
  pub fn list_wallmarks() -> impl Iterator<Item = StaticBatch> {
    Self::list().filter(|batch| batch.class == StaticClass::Wallmark)
  }
}

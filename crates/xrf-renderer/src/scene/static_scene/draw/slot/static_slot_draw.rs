use crate::scene::static_scene::draw::static_batch::StaticBatch;

/// How a slot draws: once at its own place or at every place its rows name ([`StaticSlot::SINGLE`] or
/// [`StaticSlot::LISTED`]), in which batch, with which surface's row, and in which surface passes.
///
/// [`StaticSlot::SINGLE`]: crate::scene::static_scene::draw::slot::static_slot::StaticSlot::SINGLE
/// [`StaticSlot::LISTED`]: crate::scene::static_scene::draw::slot::static_slot::StaticSlot::LISTED
#[derive(Clone, Copy, Debug)]
pub(crate) struct StaticSlotDraw {
  pub kind: u32,
  pub batch: StaticBatch,
  pub surface: u32,
  pub passes: u32,
}

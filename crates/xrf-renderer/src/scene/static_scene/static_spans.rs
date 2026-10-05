use xrf_renderer_core::Span;

use crate::scene::static_scene::static_layout::StaticLayout;

/// The ranges one scene item holds of the static scene's span buffers, none where it put nothing in one.
#[derive(Debug, Default)]
pub struct StaticSpans {
  pub words: [Option<Span>; StaticLayout::COUNT],
  pub indices: Option<Span>,
  /// Its clusters and their spheres.
  pub clusters: Option<Span>,
  pub slots: Option<Span>,
  pub places: Option<Span>,
  /// Its impostors, with their corners, terms and list entries.
  pub impostors: Option<Span>,
  pub skins: Option<Span>,
}

impl StaticSpans {
  /// Where a range starts, zero for one not held, which is what a record's local index is moved by.
  pub fn get_offset(span: &Option<Span>) -> u32 {
    span.as_ref().map_or(0, Span::get_offset)
  }

  /// Whether a range holds an element of the buffer, by its index there.
  pub fn contains(span: &Option<Span>, index: u32) -> bool {
    span
      .as_ref()
      .is_some_and(|span| (span.get_offset()..span.get_offset() + span.get_count()).contains(&index))
  }
}

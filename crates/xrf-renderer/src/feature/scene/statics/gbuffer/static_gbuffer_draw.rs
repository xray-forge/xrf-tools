/// How one G-buffer draw of the static scene goes.
#[derive(Clone, Copy, Debug, Default)]
pub struct StaticGbufferDraw {
  /// Whether the frame is drawn for an inspection view, through its own pipelines.
  pub is_inspected: bool,
  /// Whether the opaque surfaces' depth is drawn ahead of their shading.
  pub is_prepassed: bool,
  /// Which of its passes this is.
  pub part: StaticGbufferPart,
}

/// One of a G-buffer draw's passes, each a raster pass of its own so the frame times them apart.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StaticGbufferPart {
  /// The cut-out surfaces' depth, and the opaque ones' where it prepasses, ahead of their shading.
  #[default]
  Depth,
  /// Every surface shaded into the G-buffer, against the depth drawn ahead where it was.
  Shading,
  /// The impostors, with the frame's first draw: no occlusion sets one aside.
  Impostors,
}

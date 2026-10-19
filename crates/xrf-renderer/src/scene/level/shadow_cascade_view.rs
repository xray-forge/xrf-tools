use glam::Vec3;

use crate::lighting::sun_cascade::SunCascade;
use crate::scene::level::shadow_cut::ShadowCut;
use crate::scene::level::shadow_prefetch::ShadowPrefetch;

/// One cascade of a level's sun shadow: its fit, the cut it is culled and drawn by, and what its map was last drawn at;
/// and, while it keeps a static layer, which layer that is, what it was drawn at, and the next one drawn ahead.
pub struct ShadowCascadeView {
  pub cascade: SunCascade,
  pub cut: ShadowCut,
  /// The version and what the scene held when its map was last drawn, or none before it was drawn.
  pub drawn: Option<(u64, usize)>,
  /// The sway's time its map was last drawn at.
  pub drawn_at: f32,
  /// The fit its map was last drawn at, which a fit strayed too far from draws again whatever its stagger.
  pub drawn_fit: SunCascade,
  /// Frames it has waited for a draw it wants.
  pub waited: u32,
  /// Whether its map holds the grass's cards, which stand where they stood when it was drawn: it draws again once the
  /// grass no longer casts into it, so none of them stay behind.
  pub has_drawn_grass: bool,
  /// Whether its kept layer holds its static layer and trees as its map was last drawn from, which a frame drawing
  /// only the grass copies in.
  pub is_kept: bool,
  /// The eye and the look its kept layer's trees were kept for, where they were kept only where the camera saw their
  /// shadows.
  pub received_at: Option<(Vec3, Vec3)>,
  /// The sway's time a banded redraw of its trees started at and the band it draws next, while one runs.
  pub sway_sweep: Option<(f32, u32)>,
  /// The layer of the static layers it shows, and the version and contents its static layer was drawn at, none before
  /// it was or since it went stale; and the fit it was drawn at.
  pub static_layer: u32,
  pub drawn_static: Option<(u64, usize)>,
  pub static_fit: SunCascade,
  /// Its next static layer, while one is drawn ahead, and the cut its bands are culled and drawn by.
  pub prefetch: Option<ShadowPrefetch>,
  pub band: Option<ShadowCut>,
  /// Whether its static layer was just replaced by the one drawn ahead, which its map has yet to show.
  pub is_swapped: bool,
}

impl ShadowCascadeView {
  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, index: usize) -> Self {
    Self {
      cascade: SunCascade::default(),
      cut: ShadowCut::new(device, view_layout, "sun shadow"),
      drawn: None,
      drawn_at: 0.0,
      drawn_fit: SunCascade::default(),
      waited: 0,
      has_drawn_grass: false,
      is_kept: false,
      received_at: None,
      sway_sweep: None,
      static_layer: index as u32,
      drawn_static: None,
      static_fit: SunCascade::default(),
      prefetch: None,
      band: None,
      is_swapped: false,
    }
  }

  /// Lets go of what it holds, its cut's lists and its band, and of what its map was drawn at: drawn afresh once it is
  /// asked for again.
  pub fn release(&mut self, index: usize) {
    self.cut.release_lists();
    self.band = None;
    self.drawn = None;
    self.forget_static(index);
  }

  /// The bytes of its cut's lists and its band's, while it holds them.
  pub fn get_memory(&self) -> u64 {
    self.cut.get_memory() + self.band.as_ref().map_or(0, ShadowCut::get_memory)
  }

  /// Forgets its static layer: drawn again before it is read, from the cascade's own layer.
  pub fn forget_static(&mut self, index: usize) {
    self.static_layer = index as u32;
    self.drawn_static = None;
    self.prefetch = None;
    self.is_swapped = false;
    self.is_kept = false;
    self.received_at = None;
    self.sway_sweep = None;
  }
}

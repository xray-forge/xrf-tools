use serde::{Deserialize, Serialize};

use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_asset_preview::RenderAssetPreview;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_show_flags::RenderShowFlags;
use crate::contract::render_tonemap_mode::RenderTonemapMode;
use crate::contract::render_view_features::RenderViewFeatures;
use crate::contract::render_view_mode::RenderViewMode;
use crate::contract::render_view_output::RenderViewOutput;

/// What one viewport draws its scene with, split by who owns each part: what it shows, how it shades it, each
/// feature's settings, what its frame is drawn at, and an asset viewer's staging. What of the world plays is the
/// world's own.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderViewOptions {
  pub show: RenderShowFlags,
  pub mode: RenderViewMode,
  pub features: RenderViewFeatures,
  pub output: RenderViewOutput,
  pub asset: RenderAssetPreview,
}

impl RenderViewOptions {
  /// Whether the scene holds linear light at its exposure, which the window pass tonemaps: the enhanced tonemap over a
  /// lit, solid view.
  pub fn is_scene_linear(&self) -> bool {
    self.features.tonemap == RenderTonemapMode::Enhanced && self.mode.is_lit && !self.mode.is_wireframe
  }

  /// Whether the view inspects its surfaces rather than shows them: a wireframe, the asset checker, alpha hidden, or the
  /// parallax, bumps, heights or sampled mip drawn for their debug views.
  pub fn is_inspecting_surfaces(&self) -> bool {
    self.mode.is_wireframe
      || self.asset.checker > 0.0
      || !self.show.is_alpha_visible
      || matches!(
        self.mode.debug_view,
        RenderDebugView::Parallax | RenderDebugView::Bumps | RenderDebugView::Heights | RenderDebugView::SampledMip
      )
  }

  /// Whether the view traces rays through its scene: its reflections or its light's occlusion traced.
  pub fn is_scene_traced(&self) -> bool {
    (self.features.reflections.is_traced && self.features.reflections.is_drawn()) || self.is_light_traced()
  }

  /// Whether the view traces what occludes its light: its ambient occlusion traced, or DLSS ray reconstruction asked
  /// for over linear light, which traces the sun's and the lamps' shadows.
  pub fn is_light_traced(&self) -> bool {
    let features: &RenderViewFeatures = &self.features;
    let is_reconstruction_asked: bool = features.antialiasing == RenderAntialiasing::Dlss
      && self.output.upscaling.is_ray_reconstructed
      && self.is_scene_linear();

    (features.ambient_occlusion.is_enabled && features.ambient_occlusion.is_traced()) || is_reconstruction_asked
  }
}

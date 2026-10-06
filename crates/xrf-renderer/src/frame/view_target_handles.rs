use xrf_renderer_core::{FrameGraph, GraphBindings, GraphTexture};

use crate::frame::view_targets::ViewTargets;

/// A view's targets as its frame graph imports them, once a frame, for every pass drawing into or reading them.
#[derive(Clone, Copy)]
pub struct ViewTargetHandles {
  /// The targets' size, which a pass's own transients take.
  pub size: (u32, u32),
  pub albedo: GraphTexture,
  pub normal: GraphTexture,
  pub material: GraphTexture,
  pub motion: GraphTexture,
  pub depth: GraphTexture,
  pub light: GraphTexture,
  pub scene: GraphTexture,
  pub distortion: GraphTexture,
  /// The ambient occlusion's two half-size targets, searched into the first and denoised through the second.
  pub occlusion: [GraphTexture; 2],
  pub haze: GraphTexture,
  pub high: GraphTexture,
  pub bloom: [GraphTexture; 2],
}

impl ViewTargetHandles {
  pub fn import<'r>(graph: &mut FrameGraph<'_>, bindings: &mut GraphBindings<'r>, targets: &'r ViewTargets) -> Self {
    let mut import = |label: &'static str, view: &'r wgpu::TextureView| bindings.import_view(graph, label, view);

    Self {
      size: (targets.width, targets.height),
      albedo: import("albedo", &targets.albedo),
      normal: import("normal", &targets.normal),
      material: import("material", &targets.material),
      motion: import("motion", &targets.motion),
      depth: import("depth", &targets.depth),
      light: import("light", &targets.light),
      scene: import("scene", &targets.scene),
      distortion: import("distortion", &targets.distortion),
      occlusion: [
        import("ambient occlusion", &targets.occlusion[0]),
        import("ambient occlusion denoised", &targets.occlusion[1]),
      ],
      haze: import("sky haze", &targets.haze),
      high: import("high", &targets.high),
      bloom: [
        import("bloom", &targets.bloom[0]),
        import("bloom blurred", &targets.bloom[1]),
      ],
    }
  }

  /// The G-buffer's colour targets, in the order its draws attach them.
  pub fn get_gbuffer(&self) -> [GraphTexture; 4] {
    [self.albedo, self.normal, self.material, self.motion]
  }
}

use xrf_renderer_core::{FrameGraph, GraphBindings, GraphTexture, GraphTextureDescriptor};

use crate::frame::view_target_uses::ViewTargetUses;
use crate::frame::view_targets::ViewTargets;

/// A view's targets as its frame graph knows them, once a frame, for every pass drawing into or reading them: the
/// G-buffer, depth, light and scene the view keeps, imported; and the targets each frame writes afresh, made in the
/// graph where the frame writes them and a texel of nothing in their place where it does not.
#[derive(Clone, Copy)]
pub struct ViewTargetHandles {
  /// The targets' size, which a pass's own transients take.
  pub size: (u32, u32),
  pub albedo: GraphTexture,
  pub normal: GraphTexture,
  pub material: GraphTexture,
  pub motion: GraphTexture,
  pub relief: GraphTexture,
  pub depth: GraphTexture,
  pub light: GraphTexture,
  pub scene: GraphTexture,
  /// How far the water and the distorting particles move what is seen through them, cleared at the frame's start.
  pub distortion: GraphTexture,
  /// The ambient occlusion's two half-size targets, searched into the first and denoised through the second.
  pub occlusion: [GraphTexture; 2],
  pub haze: GraphTexture,
  /// `rt_Generic_1`: what combine, the sky and the sun shafts leave of the scene's light past the tonemap.
  pub high: GraphTexture,
  /// `rt_Bloom_1` and `rt_Bloom_2`, the engine's bloom built and blurred to and fro between them.
  pub bloom: [GraphTexture; 2],
}

impl ViewTargetHandles {
  /// Imports the targets the view keeps and makes the frame's own by `uses`, `none` standing in for what is not made.
  pub fn import<'r>(
    graph: &mut FrameGraph<'_>,
    bindings: &mut GraphBindings<'r>,
    (targets, uses, none): (&'r ViewTargets, ViewTargetUses, &'r wgpu::TextureView),
  ) -> Self {
    let (width, height) = (targets.width, targets.height);
    let half: (u32, u32) = (width.div_ceil(2), height.div_ceil(2));
    let mut import = |label: &'static str, view: &'r wgpu::TextureView| bindings.import_view(&mut *graph, label, view);
    let albedo: GraphTexture = import("albedo", &targets.albedo);
    let normal: GraphTexture = import("normal", &targets.normal);
    let material: GraphTexture = import("material", &targets.material);
    let motion: GraphTexture = import("motion", &targets.motion);
    let relief: GraphTexture = import("relief", &targets.relief);
    let depth: GraphTexture = import("depth", &targets.depth);
    let light: GraphTexture = import("light", &targets.light);
    let scene: GraphTexture = import("scene", &targets.scene);
    let [no_occlusion, no_haze, no_bloom] =
      ["ambient occlusion none", "sky haze none", "bloom none"].map(|label| import(label, none));
    let mut create = |label: &'static str, (width, height): (u32, u32), format: wgpu::TextureFormat| {
      graph.create_texture(GraphTextureDescriptor::new_2d(label, width, height, format))
    };

    Self {
      size: (width, height),
      albedo,
      normal,
      material,
      motion,
      relief,
      depth,
      light,
      scene,
      distortion: create("distortion", (width, height), ViewTargets::DISTORTION),
      occlusion: if uses.is_occluded {
        [
          create("ambient occlusion", half, ViewTargets::OCCLUSION),
          create("ambient occlusion denoised", half, ViewTargets::OCCLUSION),
        ]
      } else {
        [no_occlusion; 2]
      },
      haze: if uses.is_hazed {
        create("sky haze", ViewTargets::HAZE_SIZE, ViewTargets::HAZE)
      } else {
        no_haze
      },
      high: create("high", (width, height), ViewTargets::HIGH),
      bloom: if uses.is_engine_bloomed {
        [
          create("bloom", ViewTargets::BLOOM_SIZE, ViewTargets::BLOOM),
          create("bloom blurred", ViewTargets::BLOOM_SIZE, ViewTargets::BLOOM),
        ]
      } else {
        [no_bloom; 2]
      },
    }
  }

  /// The G-buffer's colour targets, in the order its draws attach them.
  pub fn get_gbuffer(&self) -> [GraphTexture; 5] {
    [self.albedo, self.normal, self.material, self.motion, self.relief]
  }
}

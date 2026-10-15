use xrf_renderer_core::{FrameGraph, GraphBindings, GraphBuffer, GraphTexture};

use crate::scene::level::feature::fsr_frame::FsrFrame;
use crate::scene::level::feature::fsr_targets::FsrTargets;

/// FSR's targets imported into a frame's graph: the reconstruction's buffer, the frame's own targets, and this frame's
/// side of the two the frames swap with the last frame's.
#[derive(Clone, Copy)]
pub struct FsrGraphTextures {
  pub reconstructed: GraphBuffer,
  pub opaque: GraphTexture,
  pub reactive: GraphTexture,
  pub dilated_depth: GraphTexture,
  pub dilated_motion: GraphTexture,
  pub farthest_depth: GraphTexture,
  pub half_farthest_depth: GraphTexture,
  /// The shading change pyramid's three levels, the finest first.
  pub pyramid: [GraphTexture; 3],
  pub shading_change: GraphTexture,
  pub masks: GraphTexture,
  pub new_locks: GraphTexture,
  pub luma_instability: GraphTexture,
  pub luma: GraphTexture,
  pub previous_luma: GraphTexture,
  pub accumulation: GraphTexture,
  pub previous_accumulation: GraphTexture,
  pub luma_history: GraphTexture,
  pub previous_luma_history: GraphTexture,
  pub history: GraphTexture,
  pub previous_history: GraphTexture,
}

impl FsrGraphTextures {
  pub fn import<'a>(graph: &mut FrameGraph<'a>, bindings: &mut GraphBindings<'a>, fsr: &'a FsrTargets) -> Self {
    let reconstructed: GraphBuffer = bindings.import_buffer(graph, "fsr reconstructed", &fsr.reconstructed);
    let mut import = |label: &'static str, view: &'a wgpu::TextureView| bindings.import_view(graph, label, view);
    // This frame's targets, written, and the last frame's, read.
    let sides: [&'a FsrFrame; 2] = fsr.frames.get_sides();
    let [luma, previous_luma] = sides.map(|frame| import("fsr luma", &frame.luma.view));
    let [accumulation, previous_accumulation] = sides.map(|frame| import("fsr accumulation", &frame.accumulation.view));
    let [luma_history, previous_luma_history] = sides.map(|frame| import("fsr luma history", &frame.luma_history.view));
    let [history, previous_history] = sides.map(|frame| import("fsr history", &frame.history.view));

    Self {
      reconstructed,
      opaque: import("fsr opaque", &fsr.opaque.view),
      reactive: import("fsr reactive", &fsr.reactive.view),
      dilated_depth: import("fsr dilated depth", &fsr.dilated_depth.view),
      dilated_motion: import("fsr dilated motion", &fsr.dilated_motion.view),
      farthest_depth: import("fsr farthest depth", &fsr.farthest_depth.view),
      half_farthest_depth: import("fsr half farthest depth", &fsr.half_farthest_depth.view),
      pyramid: fsr
        .shading_pyramid
        .each_ref()
        .map(|level| import("fsr shading pyramid", &level.view)),
      shading_change: import("fsr shading change", &fsr.shading_change.view),
      masks: import("fsr masks", &fsr.masks.view),
      new_locks: import("fsr new locks", &fsr.new_locks.view),
      luma_instability: import("fsr luma instability", &fsr.luma_instability.view),
      luma,
      previous_luma,
      accumulation,
      previous_accumulation,
      luma_history,
      previous_luma_history,
      history,
      previous_history,
    }
  }
}

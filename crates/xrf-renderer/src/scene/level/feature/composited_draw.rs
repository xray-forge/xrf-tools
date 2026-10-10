use xrf_renderer_core::{GraphBuffer, StorageValue};

use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::composited_parameters::CompositedParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::level::view_record::ViewRecord;
use crate::scene::static_scene::static_layout::StaticLayout;

/// What the composited surfaces' passes draw with in a frame: the view and the bindless textures, the targets, the
/// static scene's handles and the camera's layout draws, its draw arguments (the early phase's, and the late phase's
/// where occlusion culls), and the parameters every one of them binds.
#[derive(Clone)]
pub struct CompositedDraw<'a> {
  pub view: &'a ViewBinding,
  pub textures: &'a wgpu::BindGroup,
  pub handles: ViewTargetHandles,
  pub statics: StaticSceneHandles,
  pub layouts: [StaticDrawParameters; StaticLayout::COUNT],
  pub args: Vec<GraphBuffer>,
  pub composited: CompositedParameters<'a>,
}

impl<'a> CompositedDraw<'a> {
  /// The draws of a frame's `record`, from the cull's argument buffers `args`.
  pub fn new(record: ViewRecord<'a>, args: Vec<GraphBuffer>) -> Self {
    let lit: LightingHandles<'a> = record.lit;

    Self {
      view: record.view,
      textures: record.textures,
      handles: record.handles,
      statics: record.statics,
      layouts: record.statics.get_camera_draws().layouts,
      args,
      composited: CompositedParameters {
        lighting: lit.lighting,
        exposure: StorageValue::new(lit.exposure),
        material_lut: lit.material_lut,
        lut_sampler: lit.lut_sampler,
        shadow_maps: lit.shadow_maps,
        shadows: lit.shadows,
      },
    }
  }
}

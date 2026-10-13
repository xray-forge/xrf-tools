use glam::Vec4;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureDescriptor, UniformBinding,
};

use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::contact_shadow_parameters::ContactShadowParameters;
use crate::pass::contact_shadow_pass::ContactShadowPass;
use crate::pass::contact_shadow_uniform::ContactShadowUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;

/// The contact shadows, marched over the frame's depth where the view is lit and solid: the sun's under its cascades, at
/// half the frame's size into a target the sun brings up and multiplies its own by, and the lights', which the lights
/// march in their own pass.
#[derive(Default)]
pub struct ContactShadowFeature {
  /// Whether the sun's are marched this frame.
  is_sun: bool,
  uniform: ContactShadowUniform,
}

impl ContactShadowFeature {
  /// Takes this frame's contact shadows towards the sun's view space direction `to_sun`, for a frame `height` texels
  /// tall, their noise turned by `noise_frame`.
  pub fn prepare(&mut self, options: &RenderViewOptions, to_sun: Vec4, (height, noise_frame): (u32, u32)) {
    let shadows: &RenderShadowSettings = &options.features.shadows;
    let is_marched: bool = options.mode.is_lit && !options.mode.is_wireframe;

    self.is_sun = is_marched && shadows.is_sun_contact_drawn();
    self.uniform = if is_marched {
      ContactShadowUniform::new(&shadows.contact, to_sun, height, noise_frame)
    } else {
      ContactShadowUniform::default()
    };
  }

  /// Lights each texel marches contact shadows towards at most this frame: none while the lights march none.
  pub fn get_light_count(&self) -> u32 {
    self.uniform.lights
  }

  /// Adds the sun's contact shadows, answering what the sun multiplies its own by, a lit texel where none are marched,
  /// and the uniform the lights read theirs by.
  pub fn add_passes<'a>(
    &self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    targets: ViewTargetHandles,
  ) -> (GraphTexture, UniformBinding<ContactShadowUniform>) {
    let contact: UniformBinding<ContactShadowUniform> = runtime.push_uniform(&self.uniform);

    if !self.is_sun {
      return (
        bindings.import_view(graph, "contact shadows lit", passes.contact_shadows.get_lit()),
        contact,
      );
    }

    let (width, height) = targets.size;
    let marched: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "contact shadows",
      width.div_ceil(2),
      height.div_ceil(2),
      ContactShadowPass::FORMAT,
    ));
    let parameters: ContactShadowParameters = ContactShadowParameters {
      normal_target: targets.normal,
      depth_target: targets.depth,
      contact,
    };

    graph
      .add_raster_pass("contact shadows")
      .parameters(&parameters)
      .color(GraphColorAttachment::new(
        marched,
        wgpu::LoadOp::Clear(wgpu::Color::WHITE),
      ))
      .record(move |context| passes.contact_shadows.record(context, view, &parameters));

    (marched, contact)
  }
}

impl ViewFeature for ContactShadowFeature {
  /// Its target is the frame's own.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {}
}

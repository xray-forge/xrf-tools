use glam::{Vec2, Vec3, Vec4};
use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureDescriptor, UniformBinding};

use crate::contract::render_grass_settings::RenderGrassSettings;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::feature::lighting::contact_shadows::contact_shadow_parameters::ContactShadowParameters;
use crate::feature::lighting::contact_shadows::contact_shadow_pass::ContactShadowPass;
use crate::feature::lighting::contact_shadows::contact_shadow_uniform::ContactShadowUniform;
use crate::feature::lighting::contact_shadows::contact_shadows_recorded::ContactShadowsRecorded;
use crate::feature::scene::grass::cards::grass_cards::GrassCards;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;

/// The contact shadows, marched over the frame's depth where the view is lit and solid: the sun's under its cascades,
/// at the frame's size or half of it into a target the sun brings up and multiplies its own by, none while the sun
/// lights nothing (a weather's night); and the lights', which the lights march in their own pass.
#[derive(Default)]
pub struct ContactShadowFeature {
  /// Whether the sun's are marched this frame.
  is_sun: bool,
  uniform: ContactShadowUniform,
}

impl ContactShadowFeature {
  /// Takes this frame's contact shadows towards the sun as the frame's `lighting` gives it, for a frame `height` texels
  /// tall, their noise turned by `noise_frame`.
  pub fn prepare(&mut self, options: &RenderViewOptions, lighting: &LightingUniform, height: u32, noise_frame: u32) {
    let (to_sun, sun): (Vec4, Vec4) = (lighting.to_sun, lighting.sun);

    let shadows: &RenderShadowSettings = &options.features.shadows;
    let is_marched: bool = options.mode.is_lit && !options.mode.is_wireframe;
    // A sun of no colour adds nothing for its contact shadows to take away.
    let is_sunlit: bool = sun.truncate() != Vec3::ZERO;

    self.is_sun = is_marched && is_sunlit && shadows.is_sun_contact_drawn();
    self.uniform = if is_marched {
      ContactShadowUniform::new(
        &shadows.contact,
        to_sun,
        height,
        noise_frame,
        Self::get_grass_reach(options),
      )
    } else {
      ContactShadowUniform::default()
    };
  }

  /// Metres from the eye the grass hides contact shadows' rays to, and from where it fades out: none while it casts
  /// none.
  fn get_grass_reach(options: &RenderViewOptions) -> Vec2 {
    let grass: &RenderGrassSettings = &options.features.grass;

    if !grass.is_enabled || !grass.is_shadowed {
      return Vec2::ZERO;
    }

    let (reach, fade) = GrassCards::get_reach(grass, None);

    Vec2::new(reach, reach - fade)
  }

  /// Lights each texel marches contact shadows towards at most this frame: none while the lights march none.
  pub fn get_light_count(&self) -> u32 {
    self.uniform.lights
  }

  /// Adds the sun's contact shadows, answering what the sun multiplies its own by, a lit texel where none are marched,
  /// and the uniform the lights read theirs by.
  pub fn add_passes<'a>(
    &self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    targets: ViewTargetHandles,
  ) -> ContactShadowsRecorded {
    let contact: UniformBinding<ContactShadowUniform> = runtime.push_uniform(&self.uniform);

    if !self.is_sun {
      return ContactShadowsRecorded {
        sun: bindings.import_view(graph, "contact shadows lit", passes.lighting.contact_shadows.get_lit()),
        uniform: contact,
      };
    }

    let (width, height) = targets.size;
    let stride: u32 = self.uniform.stride.max(1);
    let marched: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "contact shadows",
      width.div_ceil(stride),
      height.div_ceil(stride),
      ContactShadowPass::FORMAT,
    ));
    let parameters: ContactShadowParameters = ContactShadowParameters {
      normal_target: targets.normal,
      material_target: targets.material,
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
      .record(move |context| passes.lighting.contact_shadows.record(context, view, &parameters));

    ContactShadowsRecorded {
      sun: marched,
      uniform: contact,
    }
  }
}

impl ViewFeature for ContactShadowFeature {
  /// Its target is the frame's own.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {}
}

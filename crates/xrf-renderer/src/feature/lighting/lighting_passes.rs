use xrf_error::XrfResult;

use crate::feature::lighting::ambient_occlusion::ambient_occlusion_pass::AmbientOcclusionPass;
use crate::feature::lighting::bitmask::vbao::vbao_pass::VbaoPass;
use crate::feature::lighting::combine::combine_pass::CombinePass;
use crate::feature::lighting::contact_shadows::contact_shadow_pass::ContactShadowPass;
use crate::feature::lighting::lights::lights_pass::LightsPass;
use crate::feature::lighting::shadows::static_shadow_pass::StaticShadowPass;
use crate::feature::lighting::sun::sun_pass::SunPass;
use crate::feature::lighting::tables::brdf_table::BrdfTable;
use crate::feature::lighting::tables::material_table::MaterialTable;
use crate::pass::build::pass_build::PassBuild;
use crate::pass::build::pipeline_source::PipelineSource;

/// The passes lighting the G-buffer and shadowing it: the static scene's shadow draws, the sun and its contact
/// shadows, the local lights, the screen's occlusion and the bitmask search, the combine, and the tables every lit pass
/// reads.
pub struct LightingPasses {
  pub shadow: StaticShadowPass,
  pub sun: SunPass,
  pub contact_shadows: ContactShadowPass,
  pub lights: LightsPass,
  pub ambient_occlusion: AmbientOcclusionPass,
  pub vbao: VbaoPass,
  pub combine: CombinePass,
  pub table: MaterialTable,
  pub brdf_table: BrdfTable,
}

impl LightingPasses {
  /// Builds every pass at once, each on a thread of its own.
  ///
  /// # Errors
  ///
  /// Returns an error when a pass's shaders do not compose or compile.
  pub fn new(build: PassBuild<'_>) -> XrfResult<Self> {
    let PassBuild {
      source,
      queue,
      view,
      scene,
      texture,
      sky,
      ..
    } = build;

    std::thread::scope(|threads| {
      let shadow = threads.spawn(move || StaticShadowPass::new(source, view, scene, texture));
      let sun = threads.spawn(move || SunPass::new(queue, source, view));
      let contact_shadows = threads.spawn(move || ContactShadowPass::new(queue, source, view));
      let lights = threads.spawn(move || LightsPass::new(source, view, texture));
      let ambient_occlusion = threads.spawn(move || AmbientOcclusionPass::new(source, view));
      let vbao = threads.spawn(move || VbaoPass::new(queue, source, view));
      let combine = threads.spawn(move || CombinePass::new(source, view, sky));

      Ok(Self {
        shadow: PassBuild::join(shadow)?,
        sun: PassBuild::join(sun)?,
        contact_shadows: PassBuild::join(contact_shadows)?,
        lights: PassBuild::join(lights)?,
        ambient_occlusion: PassBuild::join(ambient_occlusion)?,
        vbao: PassBuild::join(vbao)?,
        combine: PassBuild::join(combine)?,
        table: MaterialTable::new(source.device, queue),
        brdf_table: BrdfTable::new(source.device, queue),
      })
    })
  }

  /// Rebuilds the pipelines a shader reload made stale, each pass keeping its last good ones where a rebuild fails.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    self.shadow.refresh(source);
    self.sun.refresh(source);
    self.contact_shadows.refresh(source);
    self.lights.refresh(source);
    self.ambient_occlusion.refresh(source);
    self.vbao.refresh(source);
    self.combine.refresh(source);
  }
}

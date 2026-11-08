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
#[cfg(test)]
use crate::shader::generated_modules::GeneratedModules;

/// The passes lighting the G-buffer and shadowing it: the static scene's shadow draws, the sun and its contact
/// shadows, the local lights, the screen's occlusion and the bitmask search, the combine, and the tables every lit pass
/// reads.
pub(crate) struct LightingPasses {
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
  pub(crate) fn new(build: PassBuild<'_>) -> XrfResult<Self> {
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
  pub(crate) fn refresh(&mut self, source: PipelineSource<'_>) {
    self.shadow.refresh(source);
    self.sun.refresh(source);
    self.contact_shadows.refresh(source);
    self.lights.refresh(source);
    self.ambient_occlusion.refresh(source);
    self.vbao.refresh(source);
    self.combine.refresh(source);
  }

  /// Declares the generated WGSL its passes draw with: the structs their uniforms share and each module's bindings.
  ///
  /// # Errors
  ///
  /// Returns an error when passes sharing a module declare one of its bindings otherwise.
  #[cfg(test)]
  pub(crate) fn declare_shaders(modules: &mut GeneratedModules) -> XrfResult<()> {
    use crate::feature::lighting::ambient_occlusion::ambient_occlusion_parameters::AmbientOcclusionParameters;
    use crate::feature::lighting::ambient_occlusion::ambient_occlusion_uniform::AmbientOcclusionUniform;
    use crate::feature::lighting::bitmask::vbao::vbao_parameters::VbaoParameters;
    use crate::feature::lighting::bitmask::vbao::vbao_uniform::VbaoUniform;
    use crate::feature::lighting::combine::combine_parameters::CombineParameters;
    use crate::feature::lighting::contact_shadows::contact_shadow_parameters::ContactShadowParameters;
    use crate::feature::lighting::contact_shadows::contact_shadow_uniform::ContactShadowUniform;
    use crate::feature::lighting::lights::contact::light_contact_parameters::LightContactParameters;
    use crate::feature::lighting::lights::lights_parameters::LightsParameters;
    use crate::feature::lighting::lights::lights_uniform::LightsUniform;
    use crate::feature::lighting::lights::tracing::light_trace_uniform::LightTraceUniform;
    use crate::feature::lighting::lights::tracing::light_tracing_parameters::LightTracingParameters;
    use crate::feature::lighting::lights::visibility::light_binning_parameters::LightBinningParameters;
    use crate::feature::lighting::lights::visibility::light_shadowing_parameters::LightShadowingParameters;
    use crate::feature::lighting::lights::visibility::light_visibility_parameters::LightVisibilityParameters;
    use crate::feature::lighting::shadows::shadow_tile_restore_parameters::ShadowTileRestoreParameters;
    use crate::feature::lighting::shadows::shadow_uniform::ShadowUniform;
    use crate::feature::lighting::sun::sun_parameters::SunParameters;

    modules
      .declare::<ShadowUniform>()
      .declare::<ContactShadowUniform>()
      .declare::<LightTraceUniform>()
      .declare::<LightsUniform>()
      .declare::<AmbientOcclusionUniform>()
      .declare::<VbaoUniform>();
    modules
      .add::<ContactShadowParameters>("generated/frame/contact_shadows")?
      .add::<SunParameters<'_>>("generated/frame/sun")?
      .add::<LightBinningParameters>("generated/frame/light_binning")?
      .add::<LightsParameters<'_>>("generated/frame/lights")?
      .add::<LightContactParameters>("generated/frame/light_contact")?
      .add::<LightShadowingParameters<'_>>("generated/frame/light_shadowing")?
      .add::<LightTracingParameters>("generated/frame/light_tracing")?
      .add::<LightVisibilityParameters>("generated/frame/light_visibility")?
      .add::<AmbientOcclusionParameters>("generated/frame/ambient_occlusion")?
      .add::<VbaoParameters>("generated/frame/vbao")?
      .add::<CombineParameters<'_>>("generated/frame/combine")?
      .add::<ShadowTileRestoreParameters>("generated/frame/shadow_tile_restore")?;

    Ok(())
  }
}

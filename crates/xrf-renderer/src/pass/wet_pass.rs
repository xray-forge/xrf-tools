use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{create_fullscreen_pipeline, create_fullscreen_pipeline_into};
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;
use crate::pass::wet_apply_parameters::WetApplyParameters;
use crate::pass::wet_patch_parameters::WetPatchParameters;
use crate::scene::level::level_surface::LevelSurface;

/// Rain on the G-buffer before any light, as `draw_rain` wets it (`r3_rendertarget_draw_rain.cpp`): where the rain
/// reaches near the camera its normals patched into the light target, borrowed before the sun clears it, then written
/// back, then the albedo darkened and the gloss raised by how wet it is. Enhanced, the same three stages wet the
/// surfaces out to the distance and gather puddles on terrain.
pub struct WetPass {
  view_layout: wgpu::BindGroupLayout,
  /// The patch's, then the write back's and the wetting's.
  layouts: [wgpu::BindGroupLayout; 2],
  sampler: wgpu::Sampler,
  /// The level's surface where none is made: no lowest heights, no puddles and no water, a texel each.
  dry: [wgpu::TextureView; 3],
  /// The patch, the normal written back, the albedo and gloss wetted; the engine's, then the enhanced.
  pipelines: [[wgpu::RenderPipeline; 3]; 2],
  generation: u64,
}

impl WetPass {
  /// What the enhanced patch writes beside the light target, for the wet look, the reflections and combine to read: how
  /// much of the surface a puddle's clear coat of water covers, how much a puddle, the water's height over the ground in
  /// metres, and the variance of the ripples' tilt the patch's filtering folded away.
  pub const SURFACE: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// What both patches write beside the light target for combine: the gloss the reflections' wet reach added beyond
  /// the engine's own wetting, which the lights' specular leaves out.
  pub const REACH: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

  /// The level's surface where none is made, its lowest heights, its puddles and its water, each of nothing.
  pub fn get_dry(&self) -> [&wgpu::TextureView; 3] {
    [&self.dry[0], &self.dry[1], &self.dry[2]]
  }

  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let patch: wgpu::BindGroupLayout = WetPatchParameters::create_layout(device);
    let apply: wgpu::BindGroupLayout = WetApplyParameters::create_layout(device);
    let layouts: [wgpu::BindGroupLayout; 2] = [patch, apply];

    let texel = |label: &str, format: wgpu::TextureFormat| -> wgpu::TextureView {
      device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some(label),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
        .create_view(&Default::default())
    };

    Ok(Self {
      dry: [
        texel("level surface lowest none", LevelSurface::HEIGHT_FORMAT),
        texel("puddles none", LevelSurface::SITES_FORMAT),
        texel("level water none", SunShadowMaps::FORMAT),
      ],
      pipelines: Self::create_pipelines(source, view_layout, &layouts)?,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("wet"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layouts,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.view_layout, &self.layouts) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Wet surfaces rejected, wetting with the last ones: {error}"),
      }
    }
  }

  /// Draws one of its three stages into the target the pass draws into: the wet patches into the light, then the wet
  /// look into the normals and the albedo, each from the group it reads.
  /// The sampler the splash volume and the flow are read through, which the patch's parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// Draws one stage, the engine's or the enhanced: the patches, bound by `P` as [`WetPatchParameters`], or a wet look,
  /// as [`WetApplyParameters`].
  pub fn record<P: PassParameters>(
    &self,
    context: &mut RasterContext<'_>,
    (stage, is_enhanced): (usize, bool),
    view: &ViewBinding,
    parameters: &P,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines[usize::from(is_enhanced)][stage]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layouts: &[wgpu::BindGroupLayout; 2],
  ) -> XrfResult<[[wgpu::RenderPipeline; 3]; 2]> {
    Ok([
      Self::create_stages(
        source,
        view_layout,
        layouts,
        (["fs_wet_patch", "fs_wet_normal", "fs_wet_gloss"], false),
      )?,
      Self::create_stages(
        source,
        view_layout,
        layouts,
        (
          [
            "fs_wet_patch_enhanced",
            "fs_wet_normal_enhanced",
            "fs_wet_gloss_enhanced",
          ],
          true,
        ),
      )?,
    ])
  }

  /// The three stages by their entry points (`entries`: the patch, the normals and the gloss), the enhanced patch's
  /// beside it.
  fn create_stages(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    [patch, apply]: &[wgpu::BindGroupLayout; 2],
    ([patch_entry, normal_entry, gloss_entry], is_enhanced): ([&'static str; 3], bool),
  ) -> XrfResult<[wgpu::RenderPipeline; 3]> {
    // The enhanced patch writes the wet surface beside the light target; both, the wet reach's gloss last.
    let targets: Vec<Option<wgpu::ColorTargetState>> = std::iter::once(ViewTargets::LIGHT)
      .chain(is_enhanced.then_some(Self::SURFACE))
      .chain(std::iter::once(Self::REACH))
      .map(|format| Some(format.into()))
      .collect();
    let patched: wgpu::RenderPipeline = create_fullscreen_pipeline_into(
      source,
      "frame/wet_patch",
      patch_entry,
      &[Some(view_layout), Some(patch)],
      &targets,
    )?;
    let normal: wgpu::RenderPipeline = create_fullscreen_pipeline(
      source,
      "frame/wet_apply",
      normal_entry,
      &[Some(view_layout), Some(apply)],
      ViewTargets::NORMAL,
    )?;
    // `blend(zero, srccolor)` on colour and `(one, one)` on alpha: the albedo multiplied, the gloss added.
    let gloss: wgpu::RenderPipeline = create_fullscreen_pipeline(
      source,
      "frame/wet_apply",
      gloss_entry,
      &[Some(view_layout), Some(apply)],
      wgpu::ColorTargetState {
        format: ViewTargets::ALBEDO,
        blend: Some(wgpu::BlendState {
          color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Zero,
            dst_factor: wgpu::BlendFactor::Src,
            operation: wgpu::BlendOperation::Add,
          },
          alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
          },
        }),
        write_mask: wgpu::ColorWrites::ALL,
      },
    )?;

    Ok([patched, normal, gloss])
  }
}

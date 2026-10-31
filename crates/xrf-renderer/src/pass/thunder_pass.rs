use xrf_error::XrfResult;
use xrf_material::XraySurfaceDraw;
use xrf_renderer_core::{PassParameters, RasterContext, RenderPipelineDescription};

use crate::frame::view_targets::ViewTargets;
use crate::pass::particle_blend::ParticleBlend;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::thunder_parameters::ThunderParameters;
use crate::pass::view_binding::ViewBinding;

/// What draws a strike: its model, then its top glow, then its middle one, as `shaders/frame/thunder.wgsl` names them.
const ENTRIES: [&str; 3] = ["vs_model", "vs_glow_top", "vs_glow_center"];

/// The blends a bolt's shader composites by: blended, added, alpha added, multiplied, multiplied twice over.
const BLENDS: [ParticleBlend; 5] = [
  ParticleBlend::Blend,
  ParticleBlend::Add,
  ParticleBlend::AlphaAdd,
  ParticleBlend::Multiply,
  ParticleBlend::MultiplyDoubled,
];

/// Draws the bolt striking over a viewport's finished scene, as `dxThunderboltRender` does after the rain: its model
/// and its two glows, each composited as its shader says, tested against the G-buffer's depth without writing it.
pub struct ThunderPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  /// By `ENTRIES`, then by `BLENDS`.
  pipelines: Vec<wgpu::RenderPipeline>,
  generation: u64,
}

impl ThunderPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = ThunderParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(source, view_layout, &layout)?,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("thunder"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.view_layout, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Thunder rejected, striking with the last one: {error}"),
      }
    }
  }

  /// The sampler the strike's textures are read through, which its parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// Draws the strike: its model's indices, when it has a model, then both glows, each by its own blend.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    entries: &[ThunderParameters<'_>; 3],
    draws: [XraySurfaceDraw; 3],
    model_indices: u32,
  ) {
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);

    for (entry, (parameters, draw)) in entries.iter().zip(draws).enumerate() {
      let vertices: u32 = if entry == 0 { model_indices } else { 6 };

      if vertices == 0 {
        continue;
      }

      context.bind(parameters);
      context
        .get_pass()
        .set_pipeline(&self.pipelines[entry * BLENDS.len() + Self::get_blend_index(Self::to_blend(draw))]);
      context.get_pass().draw(0..vertices, 0..1);
    }
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Vec<wgpu::RenderPipeline>> {
    ENTRIES
      .iter()
      .flat_map(|entry| BLENDS.iter().map(move |blend| (*entry, *blend)))
      .map(|(entry, blend)| {
        let component: wgpu::BlendComponent = blend.get_component();

        // Both sides, `CULL_NONE`, and a glow faces the view anyway.
        source.get_render_pipeline(
          &RenderPipelineDescription::new("thunder", "frame/thunder", entry)
            .with_fragment(if blend.is_multiplied() {
              "fs_thunder_multiplied"
            } else {
              "fs_thunder"
            })
            .with_bind_group_layouts(&[Some(view_layout), Some(layout)])
            .with_depth_stencil(wgpu::DepthStencilState {
              format: ViewTargets::DEPTH,
              depth_write_enabled: Some(false),
              depth_compare: Some(wgpu::CompareFunction::Greater),
              stencil: Default::default(),
              bias: Default::default(),
            })
            .with_target(wgpu::ColorTargetState {
              format: ViewTargets::SCENE,
              blend: Some(wgpu::BlendState {
                color: component,
                alpha: component,
              }),
              write_mask: wgpu::ColorWrites::ALL,
            }),
        )
      })
      .collect()
  }

  /// A shader's blend among `BLENDS`; one that composites nothing, or writes over what is behind it, still lights the
  /// air, as the engine's lightning is added.
  fn to_blend(draw: XraySurfaceDraw) -> ParticleBlend {
    ParticleBlend::of(draw)
      .filter(|blend| BLENDS.contains(blend))
      .unwrap_or(ParticleBlend::Add)
  }

  /// A blend's pipelines among `BLENDS`, which `to_blend` answers from alone.
  fn get_blend_index(blend: ParticleBlend) -> usize {
    BLENDS
      .iter()
      .position(|it| *it == blend)
      .expect("a thunderbolt draws with its own blends alone")
  }
}

#[cfg(test)]
mod tests {
  use xrf_material::XraySurfaceDraw;

  use super::{BLENDS, ThunderPass};
  use crate::pass::particle_blend::ParticleBlend;

  #[test]
  fn composites_each_draw_by_its_blend_and_adds_what_has_none_among_them() {
    for (draw, blend) in [
      (XraySurfaceDraw::Blended { reference: 0 }, ParticleBlend::Blend),
      (
        XraySurfaceDraw::Added {
          reference: 0,
          is_weighted: false,
        },
        ParticleBlend::Add,
      ),
      (
        XraySurfaceDraw::Added {
          reference: 0,
          is_weighted: true,
        },
        ParticleBlend::AlphaAdd,
      ),
      (
        XraySurfaceDraw::Multiplied { is_doubled: false },
        ParticleBlend::Multiply,
      ),
      (
        XraySurfaceDraw::Multiplied { is_doubled: true },
        ParticleBlend::MultiplyDoubled,
      ),
      (XraySurfaceDraw::Opaque, ParticleBlend::Add),
      (XraySurfaceDraw::AlphaTested { reference: 0 }, ParticleBlend::Add),
      (XraySurfaceDraw::Invisible, ParticleBlend::Add),
      (XraySurfaceDraw::Water { is_soft: true }, ParticleBlend::Add),
    ] {
      assert_eq!(ThunderPass::to_blend(draw), blend, "{draw:?}");
      assert_eq!(BLENDS[ThunderPass::get_blend_index(blend)], blend, "{draw:?}");
    }
  }
}

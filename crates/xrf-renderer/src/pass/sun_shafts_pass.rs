use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{buffer_binding, create_fullscreen_pipeline_into, texture_binding};
use crate::pass::layout_entries::{storage_entry, texture_entry, uniform_entry};
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_shadows::LevelShadows;
use crate::shader::shader_library::ShaderLibrary;

/// Adds the sun's light shafts to a viewport's frame after its forward surfaces, as `phase_combine_volumetric` adds
/// what `accum_direct_volumetric` gathered: `one, one`.
pub struct SunShaftsPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl SunShaftsPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("sun shafts"),
      entries: &[
        texture_entry(
          0,
          fragment,
          wgpu::TextureSampleType::Depth,
          wgpu::TextureViewDimension::D2,
        ),
        texture_entry(
          1,
          fragment,
          wgpu::TextureSampleType::Depth,
          wgpu::TextureViewDimension::D2Array,
        ),
        uniform_entry(2, fragment),
        uniform_entry(3, fragment),
        storage_entry(4, fragment, false),
      ],
    });

    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipeline(device, shaders, &self.view_layout, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Sun shafts rejected, drawing with the last ones: {error}"),
      }
    }
  }

  /// What the shafts read: the frame's depth, the sun's shadow, the lighting and the exposure.
  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    shadows: &LevelShadows,
    (lighting, exposure): (&wgpu::Buffer, &wgpu::Buffer),
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("sun shafts"),
      layout: &self.layout,
      entries: &[
        texture_binding(0, &targets.depth),
        texture_binding(1, &shadows.get_maps().view),
        buffer_binding(2, shadows.get_uniform()),
        buffer_binding(3, lighting),
        buffer_binding(4, exposure),
      ],
    })
  }

  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    targets: &ViewTargets,
    view: &ViewBinding,
    group: &wgpu::BindGroup,
  ) {
    let added = |view| {
      Some(wgpu::RenderPassColorAttachment {
        view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Load,
          store: wgpu::StoreOp::Store,
        },
      })
    };
    let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("sun shafts"),
      color_attachments: &[added(&targets.scene), added(&targets.high)],
      ..Default::default()
    });

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    let added: wgpu::BlendComponent = wgpu::BlendComponent {
      src_factor: wgpu::BlendFactor::One,
      dst_factor: wgpu::BlendFactor::One,
      operation: wgpu::BlendOperation::Add,
    };

    let target = |format: wgpu::TextureFormat| {
      Some(wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState {
          color: added,
          alpha: added,
        }),
        write_mask: wgpu::ColorWrites::COLOR,
      })
    };

    create_fullscreen_pipeline_into(
      device,
      shaders,
      "frame/sun_shafts",
      "fs_sun_shafts",
      &[Some(view_layout), Some(layout)],
      &[target(ViewTargets::SCENE), target(ViewTargets::HIGH)],
    )
  }
}

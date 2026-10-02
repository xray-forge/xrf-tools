use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{begin_cleared_pass, create_fullscreen_pipeline, texture_binding};
use crate::pass::layout_entries::{texture_entry, uniform_entry};
use crate::pass::material_table::MaterialTable;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_shadows::LevelShadows;
use crate::shader::shader_library::ShaderLibrary;

/// Lights a viewport's G-buffer with the sun through its shadow's cascades, into the light its frame accumulates, which
/// it clears first.
pub struct SunPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl SunPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let unfiltered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: false };
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let [table, sampler] = MaterialTable::get_layout_entries(3);
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("sun"),
      entries: &[
        texture_entry(0, fragment, unfiltered, flat),
        texture_entry(1, fragment, unfiltered, flat),
        texture_entry(2, fragment, wgpu::TextureSampleType::Depth, flat),
        table,
        sampler,
        uniform_entry(5, fragment),
        texture_entry(
          6,
          fragment,
          wgpu::TextureSampleType::Depth,
          wgpu::TextureViewDimension::D2Array,
        ),
        uniform_entry(7, fragment),
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
        Err(error) => log::error!("Sun rejected, lighting with the last one: {error}"),
      }
    }
  }

  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    table: &MaterialTable,
    lighting: &wgpu::Buffer,
    shadows: &LevelShadows,
  ) -> wgpu::BindGroup {
    let [table, sampler] = table.get_entries(3);

    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("sun"),
      layout: &self.layout,
      entries: &[
        texture_binding(0, &targets.normal),
        texture_binding(1, &targets.material),
        texture_binding(2, &targets.depth),
        table,
        sampler,
        wgpu::BindGroupEntry {
          binding: 5,
          resource: lighting.as_entire_binding(),
        },
        texture_binding(6, &shadows.get_maps().view),
        wgpu::BindGroupEntry {
          binding: 7,
          resource: shadows.get_uniform().as_entire_binding(),
        },
      ],
    })
  }

  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    targets: &ViewTargets,
    view: &ViewBinding,
    bind_group: &wgpu::BindGroup,
  ) {
    let mut pass: wgpu::RenderPass<'_> = begin_cleared_pass(encoder, "sun", &targets.light);

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      device,
      shaders,
      "frame/sun",
      "fs_sun",
      &[Some(view_layout), Some(layout)],
      ViewTargets::LIGHT,
    )
  }
}

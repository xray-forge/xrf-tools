use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{begin_cleared_pass_into, create_fullscreen_pipeline_into, texture_binding};
use crate::pass::layout_entries::{storage_entry, texture_entry, uniform_entry};
use crate::pass::material_table::MaterialTable;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Lights a viewport's G-buffer with the light it accumulated and the hemisphere, and tonemaps it into its scene.
pub struct CombinePass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl CombinePass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let unfiltered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: false };
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let [table, sampler] = MaterialTable::get_layout_entries(5);
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("combine"),
      entries: &[
        texture_entry(0, fragment, unfiltered, flat),
        texture_entry(1, fragment, unfiltered, flat),
        texture_entry(2, fragment, unfiltered, flat),
        texture_entry(3, fragment, wgpu::TextureSampleType::Depth, flat),
        texture_entry(4, fragment, unfiltered, flat),
        table,
        sampler,
        uniform_entry(7, fragment),
        storage_entry(8, fragment, false),
        texture_entry(9, fragment, unfiltered, flat),
        texture_entry(10, fragment, wgpu::TextureSampleType::Float { filterable: true }, flat),
      ],
    });

    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders, view_layout, &layout, sky_layout)?,
      view_layout: view_layout.clone(),
      sky_layout: sky_layout.clone(),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipeline(device, shaders, &self.view_layout, &self.layout, &self.sky_layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Combine rejected, combining with the last one: {error}"),
      }
    }
  }

  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    table: &MaterialTable,
    lighting: &wgpu::Buffer,
    exposure: &wgpu::Buffer,
  ) -> wgpu::BindGroup {
    let [table, sampler] = table.get_entries(5);

    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("combine"),
      layout: &self.layout,
      entries: &[
        texture_binding(0, &targets.albedo),
        texture_binding(1, &targets.normal),
        texture_binding(2, &targets.material),
        texture_binding(3, &targets.depth),
        texture_binding(4, &targets.light),
        table,
        sampler,
        wgpu::BindGroupEntry {
          binding: 7,
          resource: lighting.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 8,
          resource: exposure.as_entire_binding(),
        },
        texture_binding(9, &targets.occlusion[0]),
        texture_binding(10, &targets.haze),
      ],
    })
  }

  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    targets: &ViewTargets,
    view: &ViewBinding,
    bind_group: &wgpu::BindGroup,
    sky_group: &wgpu::BindGroup,
  ) {
    let mut pass: wgpu::RenderPass<'_> = begin_cleared_pass_into(encoder, "combine", &[&targets.scene, &targets.high]);

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, bind_group, &[]);
    pass.set_bind_group(2, sky_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline_into(
      device,
      shaders,
      "frame/combine",
      "fs_combine",
      &[Some(view_layout), Some(layout), Some(sky_layout)],
      &[Some(ViewTargets::SCENE.into()), Some(ViewTargets::HIGH.into())],
    )
  }
}

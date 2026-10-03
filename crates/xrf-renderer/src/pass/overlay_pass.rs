use std::collections::HashMap;

use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{buffer_binding, texture_binding};
use crate::pass::layout_entries::{texture_entry, uniform_entry};
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_overlays::LevelOverlays;
use crate::shader::shader_library::ShaderLibrary;

/// Draws a viewport's overlays into the window's pass after its frame: line segments, depth tested against the scene by
/// the shader since the window's pass holds no depth, and the sun's disc.
pub struct OverlayPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  /// The lines' and the sun's, one pair a window format, built on first use.
  pipelines: HashMap<wgpu::TextureFormat, [wgpu::RenderPipeline; 3]>,
  generation: u64,
}

impl OverlayPass {
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> Self {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let both: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("overlay"),
      entries: &[
        texture_entry(
          0,
          fragment,
          wgpu::TextureSampleType::Depth,
          wgpu::TextureViewDimension::D2,
        ),
        uniform_entry(1, both),
        uniform_entry(2, both),
      ],
    });

    Self {
      layout,
      view_layout: view_layout.clone(),
      pipelines: HashMap::new(),
      generation: shaders.get_generation(),
    }
  }

  /// Forgets pipelines built from a library reloaded since, so the next frame builds them again.
  pub fn refresh(&mut self, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();
      self.pipelines.clear();
    }
  }

  /// Builds the pipelines drawing into a window format, unless they are built.
  ///
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn prepare(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary, format: wgpu::TextureFormat) -> XrfResult {
    if self.pipelines.contains_key(&format) {
      return Ok(());
    }

    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/overlay")?;
    let layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("overlay"),
      bind_group_layouts: &[Some(&self.view_layout), Some(&self.layout)],
      ..Default::default()
    });
    let line_attributes: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];
    let sun_attributes: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x4];
    let point_attributes: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4];
    let create = |entries: (&str, &str), topology: wgpu::PrimitiveTopology, buffer: wgpu::VertexBufferLayout<'_>| {
      create_checked(device, "overlay", || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
          label: Some(entries.0),
          layout: Some(&layout),
          vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some(entries.0),
            compilation_options: Default::default(),
            buffers: &[Some(buffer)],
          },
          fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some(entries.1),
            compilation_options: Default::default(),
            targets: &[Some(format.into())],
          }),
          primitive: wgpu::PrimitiveState {
            topology,
            ..Default::default()
          },
          depth_stencil: None,
          multisample: Default::default(),
          multiview_mask: None,
          cache: None,
        })
      })
    };
    let lines: wgpu::RenderPipeline = create(
      ("vs_line", "fs_line"),
      wgpu::PrimitiveTopology::LineList,
      wgpu::VertexBufferLayout {
        array_stride: 28,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &line_attributes,
      },
    )?;
    let sun: wgpu::RenderPipeline = create(
      ("vs_sun", "fs_sun"),
      wgpu::PrimitiveTopology::TriangleList,
      wgpu::VertexBufferLayout {
        array_stride: 16,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &sun_attributes,
      },
    )?;

    let points: wgpu::RenderPipeline = create(
      ("vs_point", "fs_point"),
      wgpu::PrimitiveTopology::TriangleList,
      wgpu::VertexBufferLayout {
        array_stride: 32,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &point_attributes,
      },
    )?;

    self.pipelines.insert(format, [lines, sun, points]);

    Ok(())
  }

  /// Binds what the overlays read of a viewport's frame: its depth, where it stands in the window, and its lighting.
  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    present: &wgpu::Buffer,
    lighting: &wgpu::Buffer,
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("overlay"),
      layout: &self.layout,
      entries: &[
        texture_binding(0, &targets.depth),
        buffer_binding(1, present),
        buffer_binding(2, lighting),
      ],
    })
  }

  /// Draws into the window's pass, whose viewport and scissor are the viewport's rectangle already.
  pub fn draw(
    &self,
    pass: &mut wgpu::RenderPass<'_>,
    format: wgpu::TextureFormat,
    (view, bind_group): (&ViewBinding, &wgpu::BindGroup),
    overlays: &LevelOverlays,
  ) {
    let Some([lines, sun, points]) = self.pipelines.get(&format) else {
      return;
    };

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, bind_group, &[]);

    for (buffer, count) in [&overlays.lines, &overlays.skeleton_lines].into_iter().flatten() {
      pass.set_pipeline(lines);
      pass.set_vertex_buffer(0, buffer.slice(..));
      pass.draw(0..*count, 0..1);
    }

    if let Some((buffer, count)) = &overlays.points {
      pass.set_pipeline(points);
      pass.set_vertex_buffer(0, buffer.slice(..));
      pass.draw(0..6, 0..*count);
    }

    if let Some(buffer) = &overlays.sun {
      pass.set_pipeline(sun);
      pass.set_vertex_buffer(0, buffer.slice(..));
      pass.draw(0..6, 0..1);
    }
  }
}

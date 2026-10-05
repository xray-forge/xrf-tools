use std::collections::HashMap;

use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::pass::overlay_parameters::OverlayParameters;
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
    let layout: wgpu::BindGroupLayout = OverlayParameters::create_layout(device);

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

  /// Draws into the window's pass, whose viewport and scissor are the viewport's rectangle already.
  pub fn draw(
    &self,
    context: &mut RasterContext<'_>,
    format: wgpu::TextureFormat,
    (view, parameters): (&ViewBinding, &OverlayParameters),
    overlays: &LevelOverlays,
  ) {
    let Some([lines, sun, points]) = self.pipelines.get(&format) else {
      return;
    };

    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);

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

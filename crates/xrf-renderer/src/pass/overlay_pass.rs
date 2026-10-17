use std::collections::HashMap;

use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::pass::overlay_parameters::OverlayParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::shader_pipelines::create_checked;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_overlays::LevelOverlays;

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
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> Self {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = OverlayParameters::create_layout(device);

    Self {
      layout,
      view_layout: view_layout.clone(),
      pipelines: HashMap::new(),
      generation: source.get_generation(),
    }
  }

  /// Builds the pipelines again from a library reloaded since they were built, one a window format each; a broken edit
  /// keeps the last good ones.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() == self.generation {
      return;
    }

    self.generation = source.get_generation();

    let rebuilt: XrfResult<HashMap<wgpu::TextureFormat, [wgpu::RenderPipeline; 3]>> = self
      .pipelines
      .keys()
      .map(|format| Ok((*format, self.create_pipeline(source, *format)?)))
      .collect();

    match rebuilt {
      Ok(pipelines) => self.pipelines = pipelines,
      Err(error) => log::error!("Overlay shader rejected, drawing with the last one: {error}"),
    }
  }

  /// Builds the pipelines drawing into a window format, unless they are built.
  ///
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn prepare(&mut self, source: PipelineSource<'_>, format: wgpu::TextureFormat) -> XrfResult {
    if !self.pipelines.contains_key(&format) {
      let pipelines: [wgpu::RenderPipeline; 3] = self.create_pipeline(source, format)?;

      self.pipelines.insert(format, pipelines);
    }

    Ok(())
  }

  /// The lines', the sun's and the points' pipelines drawing into a window format.
  fn create_pipeline(
    &self,
    source: PipelineSource<'_>,
    format: wgpu::TextureFormat,
  ) -> XrfResult<[wgpu::RenderPipeline; 3]> {
    let device: &wgpu::Device = source.device;
    let module: wgpu::ShaderModule = source.get_module("frame/overlay")?;
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
          cache: source.get_driver_cache(),
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

    Ok([lines, sun, points])
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

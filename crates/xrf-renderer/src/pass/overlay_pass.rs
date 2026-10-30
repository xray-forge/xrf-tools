use std::collections::HashMap;

use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext, RenderPipelineDescription, VertexLayout};

use crate::pass::overlay_parameters::OverlayParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::helpers::helper_vertex::HelperVertex;
use crate::scene::level::level_overlays::LevelOverlays;

/// Draws a viewport's overlays into the window's pass after its frame: line segments and points, depth tested against
/// the scene by the shader since the window's pass holds no depth, the sun's disc, and the helpers' wires, blended by
/// how they show behind the scene and how far off they stand.
pub struct OverlayPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  /// The lines', the sun's, the points' and the helpers', one set a window format, built on first use.
  pipelines: HashMap<wgpu::TextureFormat, [wgpu::RenderPipeline; 4]>,
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

    let rebuilt: XrfResult<HashMap<wgpu::TextureFormat, [wgpu::RenderPipeline; 4]>> = self
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
      let pipelines: [wgpu::RenderPipeline; 4] = self.create_pipeline(source, format)?;

      self.pipelines.insert(format, pipelines);
    }

    Ok(())
  }

  /// The lines', the sun's, the points' and the helpers' pipelines drawing into a window format.
  fn create_pipeline(
    &self,
    source: PipelineSource<'_>,
    format: wgpu::TextureFormat,
  ) -> XrfResult<[wgpu::RenderPipeline; 4]> {
    let create = |(vertex, fragment): (&'static str, &'static str),
                  topology: wgpu::PrimitiveTopology,
                  buffer: VertexLayout,
                  blend: Option<wgpu::BlendState>|
     -> XrfResult<wgpu::RenderPipeline> {
      source.get_render_pipeline(
        &RenderPipelineDescription::new(vertex, "frame/overlay", vertex)
          .with_fragment(fragment)
          .with_bind_group_layouts(&[Some(&self.view_layout), Some(&self.layout)])
          .with_vertex_layout(buffer)
          .with_primitive(wgpu::PrimitiveState {
            topology,
            ..Default::default()
          })
          .with_target(wgpu::ColorTargetState {
            format,
            blend,
            write_mask: wgpu::ColorWrites::ALL,
          }),
      )
    };
    let lines: wgpu::RenderPipeline = create(
      ("vs_line", "fs_line"),
      wgpu::PrimitiveTopology::LineList,
      VertexLayout::new(
        (LevelOverlays::LINE_FLOATS * size_of::<f32>()) as u64,
        wgpu::VertexStepMode::Vertex,
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4],
      ),
      None,
    )?;
    let sun: wgpu::RenderPipeline = create(
      ("vs_sun", "fs_sun"),
      wgpu::PrimitiveTopology::TriangleList,
      VertexLayout::new(
        (LevelOverlays::SUN_FLOATS * size_of::<f32>()) as u64,
        wgpu::VertexStepMode::Instance,
        &wgpu::vertex_attr_array![0 => Float32x4],
      ),
      None,
    )?;
    let points: wgpu::RenderPipeline = create(
      ("vs_point", "fs_point"),
      wgpu::PrimitiveTopology::TriangleList,
      VertexLayout::new(
        (LevelOverlays::POINT_FLOATS * size_of::<f32>()) as u64,
        wgpu::VertexStepMode::Instance,
        &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4],
      ),
      None,
    )?;
    let helpers: wgpu::RenderPipeline = create(
      ("vs_helper", "fs_helper"),
      wgpu::PrimitiveTopology::LineList,
      VertexLayout::new(
        HelperVertex::SIZE,
        wgpu::VertexStepMode::Vertex,
        &HelperVertex::ATTRIBUTES,
      ),
      Some(wgpu::BlendState::ALPHA_BLENDING),
    )?;

    Ok([lines, sun, points, helpers])
  }

  /// Draws into the window's pass, whose viewport and scissor are the viewport's rectangle already: the overlays, then
  /// the helpers' wires, two vertices a wire, over them.
  pub fn draw(
    &self,
    context: &mut RasterContext<'_>,
    format: wgpu::TextureFormat,
    (view, parameters): (&ViewBinding, &OverlayParameters),
    (overlays, helpers): (&LevelOverlays, Option<(&wgpu::Buffer, u32)>),
  ) {
    let Some([lines, sun, points, wires]) = self.pipelines.get(&format) else {
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

    if let Some((buffer, count)) = helpers {
      pass.set_pipeline(wires);
      pass.set_vertex_buffer(0, buffer.slice(..));
      pass.draw(0..count, 0..1);
    }
  }
}

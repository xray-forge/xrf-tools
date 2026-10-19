use std::collections::HashMap;

use xrf_error::XrfResult;

use crate::frame::frame_encoder::FrameEncoder;
use crate::pass::backdrop_uniform::BackdropUniform;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::layout_entries::uniform_entry;
use crate::pass::pipeline_source::PipelineSource;
use crate::window::window_backdrop::WindowBackdrop;

/// Paints the page's backdrop over a window where it is transparent, under its viewports: each box the page paints
/// there, and its wash. Each window keeps its own backdrop's uniform.
pub struct BackdropPass {
  layout: wgpu::BindGroupLayout,
  /// One a window format, built on first use.
  pipelines: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
  generation: u64,
}

impl BackdropPass {
  pub fn new(source: PipelineSource<'_>) -> Self {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("backdrop"),
      entries: &[uniform_entry(0, wgpu::ShaderStages::FRAGMENT)],
    });

    Self {
      layout,
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

    let rebuilt: XrfResult<HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>> = self
      .pipelines
      .keys()
      .map(|format| Ok((*format, self.create_pipeline(source, *format)?)))
      .collect();

    match rebuilt {
      Ok(pipelines) => self.pipelines = pipelines,
      Err(error) => log::error!("Backdrop shader rejected, drawing with the last one: {error}"),
    }
  }

  /// Builds the pipeline drawing into a window format, unless it is built, and writes a window's backdrop where it
  /// changed, making the window's uniform the first time.
  ///
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn prepare(
    &mut self,
    (device, encoder): (&wgpu::Device, &mut FrameEncoder),
    source: PipelineSource<'_>,
    format: wgpu::TextureFormat,
    (window, backdrop): (&mut Option<WindowBackdrop>, &BackdropUniform),
  ) -> XrfResult {
    let window: &mut WindowBackdrop = window.get_or_insert_with(|| {
      let uniform: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("backdrop"),
        size: size_of::<BackdropUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
      let bind_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("backdrop"),
        layout: &self.layout,
        entries: &[wgpu::BindGroupEntry {
          binding: 0,
          resource: uniform.as_entire_binding(),
        }],
      });

      WindowBackdrop {
        uniform,
        bind_group,
        written: None,
      }
    });

    if window.written.as_ref() != Some(backdrop) {
      encoder.write_buffer(&window.uniform, 0, bytemuck::bytes_of(backdrop));
      window.written = Some(*backdrop);
    }

    if !self.pipelines.contains_key(&format) {
      let pipeline: wgpu::RenderPipeline = self.create_pipeline(source, format)?;

      self.pipelines.insert(format, pipeline);
    }

    Ok(())
  }

  fn create_pipeline(
    &self,
    source: PipelineSource<'_>,
    format: wgpu::TextureFormat,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(source, "frame/backdrop", "fs_backdrop", &[Some(&self.layout)], format)
  }

  /// Paints a window's backdrop over the whole window, before its viewports are drawn.
  pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>, format: wgpu::TextureFormat, window: &WindowBackdrop) {
    if let Some(pipeline) = self.pipelines.get(&format) {
      pass.set_pipeline(pipeline);
      pass.set_bind_group(0, &window.bind_group, &[]);
      pass.draw(0..3, 0..1);
    }
  }
}

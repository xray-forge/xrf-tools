use crate::host::render_bundle::RenderBundle;
use crate::pass::build::pipeline_source::PipelineSource;

/// What every stage's passes are built from: where they ask for their pipelines, the queue their tables are written
/// by, the files bundled beside the code, and the layouts their draws share with every other pass, so each pipeline a
/// view asks for is the one every view asks for.
#[derive(Clone, Copy)]
pub struct PassBuild<'a> {
  pub source: PipelineSource<'a>,
  pub queue: &'a wgpu::Queue,
  pub bundle: &'a dyn RenderBundle,
  /// The view's group, at index 0 of every pass drawing a view.
  pub view: &'a wgpu::BindGroupLayout,
  /// What every static draw binds its clusters by.
  pub scene: &'a wgpu::BindGroupLayout,
  /// The bindless textures, at index 1.
  pub texture: &'a wgpu::BindGroupLayout,
  /// The sky's textures.
  pub sky: &'a wgpu::BindGroupLayout,
}

impl PassBuild<'_> {
  /// What a pass built on a thread of its own came to; a panic there goes on here.
  pub fn join<T>(handle: std::thread::ScopedJoinHandle<'_, T>) -> T {
    handle.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))
  }
}

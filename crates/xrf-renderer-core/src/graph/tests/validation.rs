use crate::graph::tests::fixtures::{COLOR, clear, color_texture, draw, load, present};
use crate::graph::{FrameGraph, GraphCompileOptions, GraphDepthAttachment, GraphTextureAccess, GraphTextureDescriptor};

fn compile_error(graph: FrameGraph<'_>) -> String {
  match graph.compile(&GraphCompileOptions::default()) {
    Ok(_) => panic!("expected the graph to be refused"),
    Err(error) => error.to_string(),
  }
}

#[test]
fn refuses_reading_a_transient_nothing_wrote() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let never = graph.create_texture(color_texture("never"));
  let scene = graph.create_texture(color_texture("scene"));

  draw(&mut graph, "scene", scene, &[never]);
  present(&mut graph, scene);

  let error: String = compile_error(graph);

  assert!(error.contains("'scene' reads transient texture 'never'"), "{error}");
}

#[test]
fn refuses_loading_an_attachment_nothing_wrote() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let scene = graph.create_texture(color_texture("scene"));

  graph.add_raster_pass("scene").color(load(scene)).record(|_| {});
  present(&mut graph, scene);

  assert!(compile_error(graph).contains("'scene' reads transient texture 'scene'"));
}

#[test]
fn reads_an_import_without_a_writer() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let history = graph.import_texture(color_texture("history"));
  let scene = graph.create_texture(color_texture("scene"));

  draw(&mut graph, "scene", scene, &[history]);
  present(&mut graph, scene);

  assert!(graph.compile(&GraphCompileOptions::default()).is_ok());
}

#[test]
fn refuses_attachments_of_different_sizes() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let color = graph.create_texture(color_texture("color"));
  let depth = graph.create_texture(GraphTextureDescriptor::new_2d(
    "depth",
    32,
    32,
    wgpu::TextureFormat::Depth32Float,
  ));

  graph
    .add_raster_pass("scene")
    .color(clear(color))
    .depth(GraphDepthAttachment::new(depth, wgpu::LoadOp::Clear(0.0)))
    .keep()
    .record(|_| {});

  assert!(compile_error(graph).contains("different sizes"));
}

#[test]
fn sizes_an_attachment_by_its_mip() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let chain = graph.create_texture(GraphTextureDescriptor::new_2d("chain", 128, 128, COLOR).with_mip_level_count(2));
  let half = graph.create_texture(GraphTextureDescriptor::new_2d("half", 64, 64, COLOR));

  graph
    .add_raster_pass("scene")
    .color(clear(chain).with_mip_level(1))
    .color(clear(half))
    .keep()
    .record(|_| {});

  assert!(graph.compile(&GraphCompileOptions::default()).is_ok());
}

#[test]
fn refuses_a_raster_pass_with_no_attachment() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();

  graph.add_raster_pass("empty").keep().record(|_| {});

  assert!(compile_error(graph).contains("no attachment"));
}

#[test]
fn refuses_writing_an_attachment_otherwise() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let color = graph.create_texture(color_texture("color"));

  graph
    .add_raster_pass("scene")
    .color(clear(color))
    .texture(color, GraphTextureAccess::StorageWrite)
    .keep()
    .record(|_| {});

  assert!(compile_error(graph).contains("otherwise than by drawing"));
}

#[test]
fn refuses_the_same_attachment_twice() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let color = graph.create_texture(color_texture("color"));

  graph
    .add_raster_pass("scene")
    .color(clear(color))
    .color(clear(color))
    .keep()
    .record(|_| {});

  assert!(compile_error(graph).contains("twice"));
}

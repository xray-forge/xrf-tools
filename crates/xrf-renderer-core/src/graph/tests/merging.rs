use crate::graph::tests::fixtures::{clear, color_texture, load, present};
use crate::graph::{FrameGraph, GraphBufferAccess, GraphBufferDescriptor, GraphCompileOptions, GraphTextureAccess};

#[test]
fn draws_passes_loading_the_same_attachments_in_one_render_pass() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let scene = graph.create_texture(color_texture("scene"));

  graph.add_raster_pass("opaque").color(clear(scene)).record(|_| {});
  graph.add_raster_pass("blended").color(load(scene)).record(|_| {});
  graph.add_raster_pass("decals").color(load(scene)).record(|_| {});
  present(&mut graph, scene);

  let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();
  let report = compiled.describe();

  assert_eq!(compiled.get_render_pass_count(), 1);
  assert!(report.passes[..3].iter().all(|pass| pass.render_pass == Some(0)));
}

#[test]
fn opens_a_new_render_pass_for_a_clear_or_other_attachments() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let scene = graph.create_texture(color_texture("scene"));
  let other = graph.create_texture(color_texture("other"));

  graph.add_raster_pass("opaque").color(clear(scene)).record(|_| {});
  graph
    .add_raster_pass("other")
    .color(clear(other))
    .texture(scene, GraphTextureAccess::Sampled)
    .record(|_| {});
  graph
    .add_raster_pass("again")
    .color(clear(scene))
    .texture(other, GraphTextureAccess::Sampled)
    .record(|_| {});
  present(&mut graph, scene);

  assert_eq!(
    graph
      .compile(&GraphCompileOptions::default())
      .unwrap()
      .get_render_pass_count(),
    3
  );
}

#[test]
fn opens_a_new_render_pass_when_a_pass_samples_an_attachment() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let scene = graph.create_texture(color_texture("scene"));

  graph.add_raster_pass("opaque").color(clear(scene)).record(|_| {});
  graph
    .add_raster_pass("refraction")
    .color(load(scene))
    .texture(scene, GraphTextureAccess::Sampled)
    .record(|_| {});
  present(&mut graph, scene);

  assert_eq!(
    graph
      .compile(&GraphCompileOptions::default())
      .unwrap()
      .get_render_pass_count(),
    2
  );
}

#[test]
fn opens_a_new_render_pass_across_a_compute_pass_or_a_group() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let scene = graph.create_texture(color_texture("scene"));
  let counts = graph.create_buffer(GraphBufferDescriptor::new("counts", 16));

  graph.add_raster_pass("opaque").color(clear(scene)).record(|_| {});
  graph
    .add_compute_pass("count")
    .buffer(counts, GraphBufferAccess::StorageWrite)
    .record(|_| {});
  graph
    .add_raster_pass("blended")
    .color(load(scene))
    .buffer(counts, GraphBufferAccess::StorageRead)
    .record(|_| {});
  graph.begin_group("post");
  graph.add_raster_pass("decals").color(load(scene)).record(|_| {});
  present(&mut graph, scene);

  assert_eq!(
    graph
      .compile(&GraphCompileOptions::default())
      .unwrap()
      .get_render_pass_count(),
    3
  );
}

#[test]
fn opens_a_render_pass_for_every_raster_pass_with_merging_off() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let scene = graph.create_texture(color_texture("scene"));

  graph.add_raster_pass("opaque").color(clear(scene)).record(|_| {});
  graph.add_raster_pass("blended").color(load(scene)).record(|_| {});
  present(&mut graph, scene);

  let compiled = graph
    .compile(&GraphCompileOptions {
      is_merging: false,
      ..GraphCompileOptions::default()
    })
    .unwrap();

  assert_eq!(compiled.get_render_pass_count(), 2);
}

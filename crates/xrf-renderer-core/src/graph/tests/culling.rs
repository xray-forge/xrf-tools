use crate::graph::tests::fixtures::{color_texture, draw, present};
use crate::graph::{FrameGraph, GraphCompileOptions};

#[test]
fn culls_a_pass_whose_output_nothing_reads() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let scene = graph.create_texture(color_texture("scene"));
  let unused = graph.create_texture(color_texture("unused"));

  draw(&mut graph, "scene", scene, &[]);
  draw(&mut graph, "unused", unused, &[]);
  present(&mut graph, scene);

  let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();

  assert_eq!(compiled.list_pass_names(), ["scene", "present"]);
  assert_eq!(compiled.get_culled(), ["unused"]);
  assert!(compiled.get_texture_slot(unused).is_none());
}

#[test]
fn keeps_every_pass_feeding_an_import() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let first = graph.create_texture(color_texture("first"));
  let second = graph.create_texture(color_texture("second"));

  draw(&mut graph, "first", first, &[]);
  draw(&mut graph, "second", second, &[first]);
  present(&mut graph, second);

  let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();

  assert_eq!(compiled.list_pass_names(), ["first", "second", "present"]);
  assert!(compiled.get_culled().is_empty());
}

#[test]
fn keeps_a_kept_pass_and_a_bridge_whatever_reads_them() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let readback = graph.create_texture(color_texture("readback"));

  graph
    .add_raster_pass("readback")
    .color(crate::graph::tests::fixtures::clear(readback))
    .keep()
    .record(|_| {});
  graph.add_encoder_pass("legacy").bridge().record(|_| {});

  let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();

  assert_eq!(compiled.list_pass_names(), ["readback", "legacy"]);
}

#[test]
fn keeps_everything_with_culling_off() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let unused = graph.create_texture(color_texture("unused"));

  draw(&mut graph, "unused", unused, &[]);

  let compiled = graph
    .compile(&GraphCompileOptions {
      is_culling: false,
      ..GraphCompileOptions::default()
    })
    .unwrap();

  assert_eq!(compiled.list_pass_names(), ["unused"]);
}

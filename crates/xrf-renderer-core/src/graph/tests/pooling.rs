use crate::graph::tests::fixtures::{color_texture, draw, present};
use crate::graph::{FrameGraph, GraphCompileOptions, GraphTexture, GraphTextureDescriptor};

/// Three transients of one key, each drawn from the one before, the last sampled into an import: the first lives over
/// passes 0-1, the second 1-2, the third 2-3, so the first and the third never overlap.
fn chain(graph: &mut FrameGraph<'_>) -> [GraphTexture; 3] {
  let first = graph.create_texture(color_texture("first"));
  let second = graph.create_texture(color_texture("second"));
  let third = graph.create_texture(color_texture("third"));
  let target = graph.import_texture(color_texture("target"));

  draw(graph, "first", first, &[]);
  draw(graph, "second", second, &[first]);
  draw(graph, "third", third, &[second]);
  draw(graph, "target", target, &[third]);

  [first, second, third]
}

#[test]
fn shares_a_slot_between_transients_that_never_overlap() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let textures: [GraphTexture; 3] = chain(&mut graph);
  let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();

  assert_eq!(
    textures.map(|texture| compiled.get_texture_slot(texture).unwrap().ordinal),
    [0, 1, 0]
  );
  assert_eq!(compiled.describe().pooled_count, 2);
}

#[test]
fn keeps_transients_of_different_keys_apart() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let first = graph.create_texture(color_texture("first"));
  let small = graph.create_texture(GraphTextureDescriptor::new_2d(
    "small",
    32,
    32,
    wgpu::TextureFormat::Rgba8Unorm,
  ));

  draw(&mut graph, "first", first, &[]);
  draw(&mut graph, "small", small, &[first]);
  present(&mut graph, small);

  let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();

  assert_ne!(
    compiled.get_texture_slot(first).unwrap().key,
    compiled.get_texture_slot(small).unwrap().key
  );
  assert_eq!(compiled.get_texture_slot(small).unwrap().ordinal, 0);
}

#[test]
fn gathers_a_transients_usage_from_its_accesses() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let scene = graph.create_texture(color_texture("scene"));

  draw(&mut graph, "scene", scene, &[]);
  present(&mut graph, scene);

  let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();

  assert_eq!(
    compiled.get_texture_slot(scene).unwrap().key.usage,
    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC
  );
}

#[test]
fn gives_every_transient_its_own_slot_with_pooling_off() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let textures: [GraphTexture; 3] = chain(&mut graph);
  let compiled = graph
    .compile(&GraphCompileOptions {
      is_pooling: false,
      ..GraphCompileOptions::default()
    })
    .unwrap();

  assert_eq!(
    textures.map(|texture| compiled.get_texture_slot(texture).unwrap().ordinal),
    [0, 1, 2]
  );
}

use crate::graph::tests::fixtures::{color_texture, draw, present};
use crate::graph::{FrameGraph, GraphCompileOptions};

fn declare(graph: &mut FrameGraph<'_>) {
  let scene = graph.create_texture(color_texture("scene"));
  let post = graph.create_texture(color_texture("post"));

  graph.begin_group("scene");
  draw(graph, "scene", scene, &[]);
  graph.begin_group("unused");
  graph.begin_group("post");
  draw(graph, "post", post, &[scene]);
  present(graph, post);
}

#[test]
fn records_each_declared_group_apart_and_drops_empty_ones() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();

  declare(&mut graph);

  let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();
  let groups: Vec<(&str, std::ops::Range<usize>)> = compiled
    .get_groups()
    .iter()
    .map(|group| (group.name, group.passes.clone()))
    .collect();

  assert_eq!(groups, [("scene", 0..1), ("post", 1..3)]);
}

#[test]
fn records_the_whole_frame_as_one_group_with_grouping_off() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();

  declare(&mut graph);

  let compiled = graph.compile(&GraphCompileOptions::serial()).unwrap();

  assert_eq!(compiled.get_groups().len(), 1);
  assert_eq!(compiled.get_groups()[0].passes, 0..3);
}

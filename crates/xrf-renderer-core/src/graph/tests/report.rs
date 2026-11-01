use crate::graph::tests::fixtures::{clear, color_texture, draw, present};
use crate::graph::{FrameGraph, GraphCompileOptions, GraphPassKind};

#[test]
fn describes_the_passes_the_culled_and_the_transients() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let scene = graph.create_texture(color_texture("scene"));
  let unused = graph.create_texture(color_texture("unused"));

  draw(&mut graph, "scene", scene, &[]);
  draw(&mut graph, "unused", unused, &[]);
  present(&mut graph, scene);

  let report = graph.compile(&GraphCompileOptions::default()).unwrap().describe();

  assert_eq!(report.passes.len(), 2);
  assert_eq!(report.passes[0].kind, GraphPassKind::Raster);
  assert_eq!(report.passes[1].kind, GraphPassKind::Encoder);
  assert_eq!(report.culled, ["unused"]);
  assert_eq!(report.transients.len(), 1);
  assert_eq!(report.transients[0].label, "scene");
  assert_eq!(report.transients[0].bytes, 64 * 64 * 4);
  assert_eq!((report.transients[0].first, report.transients[0].last), (0, 1));
  assert_eq!(report.pooled_bytes, 64 * 64 * 4);
  assert_eq!(report.textures, ["scene", "unused"]);
  assert_eq!(
    (&*report.passes[0].reads, &*report.passes[0].writes),
    (&[][..], &[0][..])
  );
  assert_eq!(
    (&*report.passes[1].reads, &*report.passes[1].writes),
    (&[0][..], &[][..])
  );

  let json: String = serde_json::to_string(&report).unwrap();

  assert!(json.contains("\"renderPassCount\":1"), "{json}");
  assert!(json.contains("\"kind\":\"encoder\""), "{json}");
}

#[test]
fn carries_each_pass_tags_into_the_report() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let scene = graph.create_texture(color_texture("scene"));

  graph
    .add_raster_pass("tagged")
    .color(clear(scene))
    .tag("first")
    .tag("second")
    .record(|_| {});
  present(&mut graph, scene);

  let report = graph.compile(&GraphCompileOptions::default()).unwrap().describe();

  assert_eq!(report.passes[0].tags, ["first", "second"]);
  assert!(report.passes[1].tags.is_empty());
}

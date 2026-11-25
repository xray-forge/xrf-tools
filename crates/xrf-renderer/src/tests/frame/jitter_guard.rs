use xrf_renderer_core::{GraphOwner, GraphPassKind, GraphPassReport, GraphReport, GraphTextureReport};

use crate::frame::jitter_guard::JitterGuard;
use crate::frame::jitter_guard::jittered_read::JitteredRead;

/// A texture declared under `owner`, carrying no jitter of its own.
fn texture(label: &str, owner: GraphOwner) -> GraphTextureReport {
  GraphTextureReport {
    label: label.to_string(),
    owner,
    tags: Vec::new(),
  }
}

/// A target `owner` draws in its own screen space under the frame's jitter.
fn jittered(label: &str, owner: GraphOwner) -> GraphTextureReport {
  GraphTextureReport {
    tags: vec![JitterGuard::JITTERED],
    ..texture(label, owner)
  }
}

/// A compute pass, which carries what it reads to what it writes and nothing more.
fn pass(name: &str, owner: GraphOwner, reads: &[usize], writes: &[usize]) -> GraphPassReport {
  GraphPassReport {
    name: name.to_string(),
    kind: GraphPassKind::Compute,
    group: "post".to_string(),
    owner,
    render_pass: None,
    reads: reads.to_vec(),
    writes: writes.to_vec(),
    tags: Vec::new(),
  }
}

fn raster(pass: GraphPassReport) -> GraphPassReport {
  GraphPassReport {
    kind: GraphPassKind::Raster,
    ..pass
  }
}

fn tagged(pass: GraphPassReport, tags: &[&'static str]) -> GraphPassReport {
  GraphPassReport {
    tags: tags.to_vec(),
    ..pass
  }
}

fn report(textures: Vec<GraphTextureReport>, passes: Vec<GraphPassReport>) -> GraphReport {
  GraphReport {
    passes,
    culled: Vec::new(),
    groups: Vec::new(),
    render_pass_count: 0,
    textures,
    transients: Vec::new(),
    transient_bytes: 0,
    pooled_count: 0,
    pooled_bytes: 0,
  }
}

fn read(pass: &str, texture: &str) -> JitteredRead {
  JitteredRead {
    pass: pass.to_string(),
    texture: texture.to_string(),
  }
}

/// After a view's resolve, its own passes and the window's reading what carries its jitter are found; those reading
/// the resolved image or a target drawn anew after it are not, nor a read the pass is tagged with such as the exposure
/// meter's, nor anything of a view that resolves nothing.
#[test]
fn finds_the_reads_of_the_jittered_frame_after_a_resolve() {
  let (view, unresolved): (GraphOwner, GraphOwner) = (GraphOwner::Part(1), GraphOwner::Part(2));
  let report: GraphReport = report(
    vec![
      jittered("frame", view),
      texture("history", view),
      jittered("streaks", view),
      jittered("other frame", unresolved),
      jittered("high", view),
    ],
    vec![
      raster(pass("opaque", view, &[], &[0, 2, 4])),
      raster(pass("unresolved opaque", unresolved, &[], &[3])),
      tagged(pass("temporal", view, &[0], &[1]), &[JitterGuard::RESOLVES]),
      pass("sharpen", view, &[1], &[]),
      pass("bloom from the frame", view, &[0], &[]),
      tagged(pass("exposure", view, &[4], &[]), &[JitterGuard::READS_HIGH]),
      raster(pass("streaks cleared", view, &[], &[2])),
      pass("streaks read", view, &[2], &[]),
      tagged(
        pass("window", GraphOwner::Frame, &[0, 1, 3], &[]),
        &JitterGuard::PRESENT_READS,
      ),
    ],
  );

  assert_eq!(
    JitterGuard::list_jittered_reads(&report),
    [read("bloom from the frame", "frame"), read("window", "frame")]
  );
}

/// Under ray reconstruction the present binds the indirect light and the reflections traced raw for its debug views,
/// which show them as drawn: allowed as the gathered ones are, while the frame drawn before the resolve is still found.
#[test]
fn allows_the_present_the_raw_traced_targets_of_ray_reconstruction() {
  let view: GraphOwner = GraphOwner::Part(1);
  let report: GraphReport = report(
    vec![
      jittered("frame", view),
      texture("indirect traced", view),
      texture("reflections traced", view),
      texture("resolved", view),
    ],
    vec![
      pass("trace", view, &[0], &[1, 2]),
      tagged(
        pass("dlss ray reconstruction", view, &[0, 1, 2], &[3]),
        &[JitterGuard::RESOLVES],
      ),
      tagged(
        pass("window", GraphOwner::Frame, &[0, 1, 2, 3], &[]),
        &JitterGuard::PRESENT_READS,
      ),
    ],
  );

  assert_eq!(JitterGuard::list_jittered_reads(&report), [read("window", "frame")]);
}

/// The engine's bloom is built before the resolve from the jittered high part, so it carries the jitter; the present
/// reads it as one of its allowed targets, while any other pass after the resolve reading it is still found.
#[test]
fn allows_the_present_the_engines_bloom_built_before_the_resolve() {
  let view: GraphOwner = GraphOwner::Part(1);
  let report: GraphReport = report(
    vec![
      jittered("frame", view),
      jittered("high", view),
      texture("bloom", view),
      texture("resolved", view),
    ],
    vec![
      raster(pass("opaque", view, &[], &[0, 1])),
      raster(pass("bloom build", view, &[1], &[2])),
      tagged(pass("temporal", view, &[0], &[3]), &[JitterGuard::RESOLVES]),
      pass("bloom after the resolve", view, &[2], &[]),
      tagged(
        pass("window", GraphOwner::Frame, &[2, 3], &[]),
        &JitterGuard::PRESENT_READS,
      ),
    ],
  );

  assert_eq!(
    JitterGuard::list_jittered_reads(&report),
    [read("bloom after the resolve", "bloom")]
  );
}

/// The helper marks draw after the resolve over the view's depth, read-only: allowed by their tag, the marks target
/// they draw anew left unjittered for the present's outline.
#[test]
fn allows_the_helper_marks_the_depth_they_are_tested_against() {
  let view: GraphOwner = GraphOwner::Part(1);
  let report: GraphReport = report(
    vec![
      jittered("depth", view),
      texture("resolved", view),
      texture("helper marks", view),
    ],
    vec![
      raster(pass("opaque", view, &[], &[0])),
      tagged(pass("temporal", view, &[0], &[1]), &[JitterGuard::RESOLVES]),
      tagged(
        raster(pass("helper marks", view, &[0], &[2])),
        &[JitterGuard::READS_DEPTH],
      ),
      tagged(
        pass("window", GraphOwner::Frame, &[1, 2], &[]),
        &JitterGuard::PRESENT_READS,
      ),
    ],
  );

  assert!(JitterGuard::list_jittered_reads(&report).is_empty());
}

/// A view with no resolve, drawn and shown as it is, has no reads to find.
#[test]
fn finds_nothing_in_a_frame_never_resolved() {
  let view: GraphOwner = GraphOwner::Part(1);
  let report: GraphReport = report(
    vec![jittered("frame", view)],
    vec![
      raster(pass("opaque", view, &[], &[0])),
      pass("bloom from the frame", view, &[0], &[]),
      pass("window", GraphOwner::Frame, &[0], &[]),
    ],
  );

  assert!(JitterGuard::list_jittered_reads(&report).is_empty());
}

/// An overhead map the view draws before its resolve from nothing jittered, as the rain's cover, is not found when a
/// pass after the resolve reads it.
#[test]
fn finds_nothing_of_an_overhead_map_drawn_before_the_resolve() {
  let view: GraphOwner = GraphOwner::Part(1);
  let report: GraphReport = report(
    vec![
      jittered("frame", view),
      texture("rain cover", view),
      texture("overhead depth", view),
      texture("resolved", view),
    ],
    vec![
      raster(pass("overhead depth", view, &[], &[2])),
      raster(pass("rain cover", view, &[2], &[1])),
      raster(pass("opaque", view, &[1], &[0])),
      tagged(pass("temporal", view, &[0], &[3]), &[JitterGuard::RESOLVES]),
      raster(pass("rain", view, &[1], &[3])),
    ],
  );

  assert!(JitterGuard::list_jittered_reads(&report).is_empty());
}

/// A compute pass before the resolve reading a jittered target makes what it writes jittered, found when read after
/// the resolve.
#[test]
fn finds_what_a_compute_pass_wrote_from_the_jittered_frame() {
  let view: GraphOwner = GraphOwner::Part(1);
  let report: GraphReport = report(
    vec![
      jittered("depth", view),
      texture("indirect light", view),
      texture("resolved", view),
    ],
    vec![
      pass("indirect", view, &[0], &[1]),
      tagged(pass("temporal", view, &[], &[2]), &[JitterGuard::RESOLVES]),
      pass("bloom", view, &[1], &[]),
    ],
  );

  assert_eq!(
    JitterGuard::list_jittered_reads(&report),
    [read("bloom", "indirect light")]
  );
}

/// A raster pass drawing into a jittered target and another texture makes the other jittered, as one rasterisation.
#[test]
fn finds_what_a_raster_pass_drew_beside_a_jittered_target() {
  let view: GraphOwner = GraphOwner::Part(1);
  let report: GraphReport = report(
    vec![
      jittered("scene", view),
      texture("depth with water", view),
      texture("resolved", view),
    ],
    vec![
      raster(pass("water", view, &[], &[0, 1])),
      tagged(pass("temporal", view, &[], &[2]), &[JitterGuard::RESOLVES]),
      pass("fog", view, &[1], &[]),
    ],
  );

  assert_eq!(
    JitterGuard::list_jittered_reads(&report),
    [read("fog", "depth with water")]
  );
}

/// What the resolve writes is not found when read after it, though the resolve read the jittered frame.
#[test]
fn finds_nothing_of_what_the_resolve_wrote() {
  let view: GraphOwner = GraphOwner::Part(1);
  let report: GraphReport = report(
    vec![jittered("frame", view), texture("resolved", view)],
    vec![
      tagged(pass("temporal", view, &[0], &[1]), &[JitterGuard::RESOLVES]),
      pass("sharpen", view, &[1], &[]),
    ],
  );

  assert!(JitterGuard::list_jittered_reads(&report).is_empty());
}

/// A read after the resolve that its pass is tagged with is not found, and the same read untagged is.
#[test]
fn finds_a_read_after_the_resolve_unless_its_pass_is_tagged_with_it() {
  let view: GraphOwner = GraphOwner::Part(1);
  let report: GraphReport = report(
    vec![jittered("depth", view), texture("resolved", view)],
    vec![
      tagged(pass("temporal", view, &[], &[1]), &[JitterGuard::RESOLVES]),
      tagged(pass("depth of field", view, &[0], &[]), &[JitterGuard::READS_DEPTH]),
      pass("untagged depth of field", view, &[0], &[]),
    ],
  );

  assert_eq!(
    JitterGuard::list_jittered_reads(&report),
    [read("untagged depth of field", "depth")]
  );
}

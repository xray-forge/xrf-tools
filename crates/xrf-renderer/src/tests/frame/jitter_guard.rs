use xrf_renderer_core::{GraphOwner, GraphPassKind, GraphPassReport, GraphReport};

use crate::frame::jitter_guard::{JitterGuard, JitteredRead};

fn pass(name: &str, owner: GraphOwner, reads: &[usize], writes: &[usize]) -> GraphPassReport {
  GraphPassReport {
    name: name.to_string(),
    kind: GraphPassKind::Raster,
    group: "post".to_string(),
    owner,
    render_pass: None,
    reads: reads.to_vec(),
    writes: writes.to_vec(),
    tags: Vec::new(),
  }
}

fn tagged(pass: GraphPassReport, tags: &[&'static str]) -> GraphPassReport {
  GraphPassReport {
    tags: tags.to_vec(),
    ..pass
  }
}

fn report(textures: &[&str], passes: Vec<GraphPassReport>) -> GraphReport {
  GraphReport {
    passes,
    culled: Vec::new(),
    groups: Vec::new(),
    render_pass_count: 0,
    textures: textures.iter().map(|texture| texture.to_string()).collect(),
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

/// After a view's resolve, its own passes and the window's reading what it drew before the resolve are found; those
/// reading the resolved image or a target drawn anew after it are not, nor a read the pass is tagged with such as the
/// exposure meter's, nor anything of a view that resolves nothing.
#[test]
fn finds_the_reads_of_the_jittered_frame_after_a_resolve() {
  let (view, unresolved): (GraphOwner, GraphOwner) = (GraphOwner::Part(1), GraphOwner::Part(2));
  let report: GraphReport = report(
    &["frame", "history", "streaks", "other frame", "high"],
    vec![
      pass("opaque", view, &[], &[0, 2, 4]),
      pass("unresolved opaque", unresolved, &[], &[3]),
      tagged(pass("temporal", view, &[0], &[1]), &[JitterGuard::RESOLVES]),
      pass("sharpen", view, &[1], &[]),
      pass("bloom from the frame", view, &[0], &[]),
      tagged(pass("exposure", view, &[4], &[]), &[JitterGuard::READS_HIGH]),
      pass("streaks cleared", view, &[], &[2]),
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
    &["frame", "indirect traced", "reflections traced", "resolved"],
    vec![
      pass("opaque", view, &[], &[0, 1, 2]),
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

/// A view with no resolve, drawn and shown as it is, has no reads to find.
#[test]
fn finds_nothing_in_a_frame_never_resolved() {
  let report: GraphReport = report(
    &["frame"],
    vec![
      pass("opaque", GraphOwner::Part(1), &[], &[0]),
      pass("bloom from the frame", GraphOwner::Part(1), &[0], &[]),
      pass("window", GraphOwner::Frame, &[0], &[]),
    ],
  );

  assert!(JitterGuard::list_jittered_reads(&report).is_empty());
}

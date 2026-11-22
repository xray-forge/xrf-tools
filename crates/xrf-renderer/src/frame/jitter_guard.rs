#[cfg(debug_assertions)]
use xrf_renderer_core::{GraphPassReport, GraphReport};

#[cfg(debug_assertions)]
use crate::frame::jitter_guard::jitter_walk::JitterWalk;
#[cfg(debug_assertions)]
use crate::frame::jitter_guard::jittered_read::JitteredRead;

#[cfg(debug_assertions)]
mod jitter_walk;
#[cfg(debug_assertions)]
pub(crate) mod jittered_read;

/// Watches each frame's graph for what undoes a temporal resolve: a pass after it reading per pixel what carries the
/// view's jitter, drawn in its screen space jittered a little differently each frame, which the resolve exists to
/// average away. Such a read puts the jitter back on screen as shimmer, so whatever reads the frame after the resolve
/// reads what it resolved.
///
/// The jitter starts in the view's own targets, tagged `JITTERED`, and is followed through the passes in order: before
/// the view's resolve, a pass reading a jittered texture of the view, or a raster pass drawing into one, makes all it
/// writes jittered too; the resolve, tagged `RESOLVES`, writes nothing jittered; after it, a read of a jittered texture
/// is found unless the pass carries that texture's `READS_*` tag, beside the reason it gives, and a pass drawing a
/// jittered texture anew without reading it leaves it unjittered. Every build tags its textures and passes; only a
/// build with debug assertions watches.
#[derive(Default)]
pub(crate) struct JitterGuard {
  /// What was last warned of, warned again only once it changes.
  #[cfg(debug_assertions)]
  warned: Vec<JitteredRead>,
}

impl JitterGuard {
  /// The tag of a texture a view draws in its own screen space under the frame's jitter.
  pub(crate) const JITTERED: &'static str = "jittered";
  /// The tag of a pass resolving a view's frame: TAA, DLSS, its ray reconstruction, or FSR's accumulation, after its
  /// stages preparing what it accumulates.
  pub(crate) const RESOLVES: &'static str = "jitter resolves";
  pub(crate) const READS_ALBEDO: &'static str = "jitter reads albedo";
  pub(crate) const READS_AMBIENT_OCCLUSION: &'static str = "jitter reads ambient occlusion";
  pub(crate) const READS_DEPTH: &'static str = "jitter reads depth";
  pub(crate) const READS_DEPTH_WITH_WATER: &'static str = "jitter reads depth with water";
  pub(crate) const READS_HIGH: &'static str = "jitter reads high";
  pub(crate) const READS_INDIRECT_LIGHT: &'static str = "jitter reads indirect light";
  pub(crate) const READS_INDIRECT_TRACED: &'static str = "jitter reads indirect traced";
  pub(crate) const READS_LIGHT: &'static str = "jitter reads light";
  pub(crate) const READS_MATERIAL: &'static str = "jitter reads material";
  pub(crate) const READS_MOTION: &'static str = "jitter reads motion";
  pub(crate) const READS_NORMAL: &'static str = "jitter reads normal";
  pub(crate) const READS_REFLECTION_HISTORY_FULL: &'static str = "jitter reads reflection history full";
  pub(crate) const READS_REFLECTIONS_ENDS: &'static str = "jitter reads reflections ends";
  pub(crate) const READS_REFLECTIONS_TRACED: &'static str = "jitter reads reflections traced";
  pub(crate) const READS_SCENE: &'static str = "jitter reads scene";

  /// The drawn targets the present reads, for a window or a capture: the debug views' raw targets, the drawn scene
  /// (only the luminance view reads it), and the depth or marks a capture copies for a tool.
  pub(crate) const PRESENT_READS: [&'static str; 13] = [
    Self::READS_ALBEDO,
    Self::READS_AMBIENT_OCCLUSION,
    Self::READS_DEPTH,
    Self::READS_INDIRECT_LIGHT,
    Self::READS_INDIRECT_TRACED,
    Self::READS_LIGHT,
    Self::READS_MATERIAL,
    Self::READS_MOTION,
    Self::READS_NORMAL,
    Self::READS_REFLECTION_HISTORY_FULL,
    Self::READS_REFLECTIONS_ENDS,
    Self::READS_REFLECTIONS_TRACED,
    Self::READS_SCENE,
  ];
}

#[cfg(debug_assertions)]
impl JitterGuard {
  /// What the tag of a drawn texture a pass after the resolve reads on purpose starts with, the texture's label after.
  const READS: &'static str = "jitter reads ";

  /// Warns of the frame's reads after a resolve that are not allowed, once each time they change.
  pub(crate) fn watch(&mut self, report: &GraphReport) {
    let reads: Vec<JitteredRead> = Self::list_jittered_reads(report);

    if reads != self.warned {
      if !reads.is_empty() {
        log::warn!("Passes after the temporal resolve read the jittered frame: {reads:?}");
      }

      self.warned = reads;
    }
  }

  /// The reads after each view's resolve of what carries its jitter, its own passes' and the window's, but those a
  /// pass's tags allow; sorted, each once.
  pub(crate) fn list_jittered_reads(report: &GraphReport) -> Vec<JitteredRead> {
    let mut walk: JitterWalk = JitterWalk::of(report);
    let mut reads: Vec<JitteredRead> = Vec::new();

    for pass in &report.passes {
      reads.extend(
        walk
          .step(pass)
          .into_iter()
          .map(|texture| report.textures[texture].label.as_str())
          .filter(|texture| !Self::is_allowed(pass, texture))
          .map(|texture| JitteredRead {
            pass: pass.name.clone(),
            texture: texture.to_string(),
          }),
      );
    }

    reads.sort();
    reads.dedup();
    reads
  }

  /// Whether `pass` is tagged as reading the drawn `texture` on purpose.
  fn is_allowed(pass: &GraphPassReport, texture: &str) -> bool {
    pass
      .tags
      .iter()
      .any(|tag| tag.strip_prefix(Self::READS) == Some(texture))
  }
}

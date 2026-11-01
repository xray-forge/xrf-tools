use std::collections::{HashMap, HashSet};

use xrf_renderer_core::{GraphOwner, GraphPassReport, GraphReport};

/// A pass after a view's temporal resolve that reads a texture the view drew before it, by their labels.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JitteredRead {
  pub pass: String,
  pub texture: String,
}

/// Watches each frame's graph for what undoes a temporal resolve: a pass after it reading per pixel what the view drew
/// before it, at the drawn size and jittered a little differently each frame, which the resolve exists to average away.
/// Such a read puts the jitter back on screen as shimmer, so whatever reads the frame after the resolve reads what it
/// resolved. It reads the passes' tags: the resolves carry `RESOLVES`, and a pass whose read of a drawn texture averages
/// it or only shows it carries that texture's `READS_*` tag, beside the reason it gives.
#[derive(Default)]
pub struct JitterGuard {
  /// What was last warned of, warned again only once it changes.
  warned: Vec<JitteredRead>,
}

impl JitterGuard {
  /// The tag of a pass resolving a view's frame: TAA, DLSS, its ray reconstruction, or FSR's accumulation, after its
  /// stages preparing what it accumulates.
  pub const RESOLVES: &'static str = "jitter resolves";
  /// What the tag of a drawn texture a pass after the resolve reads on purpose starts with, the texture's label after.
  const READS: &'static str = "jitter reads ";
  pub const READS_ALBEDO: &'static str = "jitter reads albedo";
  pub const READS_AMBIENT_OCCLUSION: &'static str = "jitter reads ambient occlusion";
  pub const READS_DEPTH: &'static str = "jitter reads depth";
  pub const READS_DEPTH_WITH_WATER: &'static str = "jitter reads depth with water";
  pub const READS_HIGH: &'static str = "jitter reads high";
  pub const READS_INDIRECT_LIGHT: &'static str = "jitter reads indirect light";
  pub const READS_INDIRECT_TRACED: &'static str = "jitter reads indirect traced";
  pub const READS_LIGHT: &'static str = "jitter reads light";
  pub const READS_MATERIAL: &'static str = "jitter reads material";
  pub const READS_MOTION: &'static str = "jitter reads motion";
  pub const READS_NORMAL: &'static str = "jitter reads normal";
  pub const READS_REFLECTION_HISTORY_FULL: &'static str = "jitter reads reflection history full";
  pub const READS_REFLECTIONS_ENDS: &'static str = "jitter reads reflections ends";
  pub const READS_REFLECTIONS_TRACED: &'static str = "jitter reads reflections traced";
  pub const READS_SCENE: &'static str = "jitter reads scene";

  /// What the present reads of the drawn targets, the window's or a capture's: the debug views show them as they are,
  /// the indirect light and the reflections traced raw for ray reconstruction among them; the sky's debanding reads
  /// the depth over four drawn texels; the drawn scene is bound beside the upscaled image the frame shows, read only by
  /// the luminance view. A capture's copy takes the depth or the marks for a tool.
  pub const PRESENT_READS: [&'static str; 13] = [
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

  /// Warns of the frame's reads after a resolve that are not allowed, once each time they change.
  pub fn watch(&mut self, report: &GraphReport) {
    let reads: Vec<JitteredRead> = Self::list_jittered_reads(report);

    if reads != self.warned {
      if !reads.is_empty() {
        log::warn!("Passes after the temporal resolve read the jittered frame: {reads:?}");
      }

      self.warned = reads;
    }
  }

  /// The reads after each view's resolve of what the view drew before it, its own passes' and the window's, but those
  /// a pass's tags allow; sorted, each once. A texture a pass after the resolve draws anew, reading none of it, is no
  /// longer the jittered one.
  pub fn list_jittered_reads(report: &GraphReport) -> Vec<JitteredRead> {
    // What each view drew before its resolve, and whether it has resolved.
    let mut views: HashMap<GraphOwner, (HashSet<usize>, bool)> = HashMap::new();
    let mut reads: Vec<JitteredRead> = Vec::new();

    for pass in &report.passes {
      let mut jittered: Vec<usize> = Vec::new();

      if pass.owner == GraphOwner::Frame {
        jittered.extend(
          views
            .values()
            .filter(|(_, is_resolved)| *is_resolved)
            .flat_map(|(drawn, _)| pass.reads.iter().copied().filter(|texture| drawn.contains(texture))),
        );
      } else {
        let (drawn, is_resolved) = views.entry(pass.owner).or_default();

        if !*is_resolved {
          if pass.tags.contains(&Self::RESOLVES) {
            *is_resolved = true;

            for texture in &pass.writes {
              drawn.remove(texture);
            }
          } else {
            drawn.extend(pass.writes.iter().copied());
          }

          continue;
        }

        jittered.extend(pass.reads.iter().copied().filter(|texture| drawn.contains(texture)));

        for texture in pass.writes.iter().filter(|texture| !pass.reads.contains(texture)) {
          drawn.remove(texture);
        }
      }

      reads.extend(
        jittered
          .into_iter()
          .map(|texture| report.textures[texture].as_str())
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

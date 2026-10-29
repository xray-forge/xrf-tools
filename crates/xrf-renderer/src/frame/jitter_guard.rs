use std::collections::{HashMap, HashSet};

use xrf_renderer_core::{GraphOwner, GraphReport};

use crate::pass::fsr_pass::FsrPass;
use crate::scene::level::feature::resolve_feature::ResolveFeature;

/// A pass after a view's temporal resolve that reads a texture the view drew before it, by their labels.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JitteredRead {
  pub pass: String,
  pub texture: String,
}

/// Watches each frame's graph for what undoes a temporal resolve: a pass after it reading per pixel what the view drew
/// before it, at the drawn size and jittered a little differently each frame, which the resolve exists to average away.
/// Such a read puts the jitter back on screen as shimmer, so whatever reads the frame after the resolve reads what it
/// resolved; the few reads that average or only show the drawn targets are allowed by name (`ALLOWED`).
#[derive(Default)]
pub struct JitterGuard {
  /// What was last warned of, warned again only once it changes.
  warned: Vec<JitteredRead>,
}

impl JitterGuard {
  /// The passes that resolve a view's frame: TAA, DLSS, its ray reconstruction, and FSR's accumulation, after its
  /// stages preparing what it accumulates.
  pub const RESOLVES: [&str; 4] = [
    ResolveFeature::TEMPORAL,
    ResolveFeature::DLSS,
    ResolveFeature::RAY_RECONSTRUCTION,
    FsrPass::ACCUMULATE,
  ];

  /// The passes after the resolve allowed to read what was drawn before it, by the start of their names, with the
  /// textures they read; each says why none of its reads shows the jitter.
  const ALLOWED: [(&[&str], &[&str]); 5] = [
    // The build reads the fog's depth and the self-lit marks over the four drawn texels under each of its half-size
    // texels; the other stages share its layout and read neither.
    (&["enhanced bloom "], &["depth", "material"]),
    // The selected glass marked for the present's outline: an editor overlay, no light.
    (&["composited selection"], &["depth", "material"]),
    // The streaks over the resolved image read the depth behind them and the light about them bilinearly over four
    // drawn texels.
    (&["rain"], &["depth", "light"]),
    // The meter averages the whole frame into one exposure.
    (&["exposure"], &["depth", "high", "scene"]),
    // The present, the window's or a capture's: the debug views show the drawn targets as they are, the indirect light
    // and the reflections traced raw for ray reconstruction among them; the sky's debanding reads the depth over four
    // drawn texels; the drawn scene is bound beside the upscaled image the frame shows, read only by the luminance view.
    // A capture's copy takes the depth or the marks for a tool.
    (
      &["window", "screenshot", "capture shot", "burst frame"],
      &[
        "albedo",
        "ambient occlusion",
        "depth",
        "indirect light",
        "indirect traced",
        "light",
        "material",
        "motion",
        "normal",
        "reflection history full",
        "reflections ends",
        "reflections traced",
        "scene",
      ],
    ),
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
  /// `ALLOWED`; sorted, each once. A texture a pass after the resolve draws anew, reading none of it, is no longer the
  /// jittered one.
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
          if Self::RESOLVES.contains(&pass.name.as_str()) {
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
          .filter(|texture| !Self::is_allowed(&pass.name, texture))
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

  fn is_allowed(pass: &str, texture: &str) -> bool {
    Self::ALLOWED
      .iter()
      .any(|(passes, textures)| passes.iter().any(|allowed| pass.starts_with(allowed)) && textures.contains(&texture))
  }
}

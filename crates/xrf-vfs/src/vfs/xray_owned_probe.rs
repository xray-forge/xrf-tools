use crate::vfs::xray_probe::XrayProbeRecording;
use crate::{XrayCachePolicy, XrayProbe, XrayProbeStep, XrayVfs};

/// A probe's search over a world it owns, for a holder that outlives the probe it came from.
///
/// Forked from a probe, it reads what that probe reads without mounting anything again, and records into what it
/// recorded into: a fork keeps the mounts, their ids and their revisions, so an input answers here as it does there.
#[derive(Debug)]
pub struct XrayOwnedProbe {
  world: XrayVfs,
  steps: Vec<XrayProbeStep>,
  recording: Option<XrayProbeRecording>,
}

impl XrayOwnedProbe {
  /// A probe of `steps` over `world`, recording nothing.
  pub fn new(world: XrayVfs, steps: Vec<XrayProbeStep>) -> Self {
    Self {
      world,
      steps,
      recording: None,
    }
  }

  /// This probe with its world retaining what `policy` allows, from nothing retained.
  pub fn with_cache_policy(mut self, policy: XrayCachePolicy) -> Self {
    self.world = self.world.with_cache_policy(policy);

    self
  }

  /// The probe to search with, recording where the probe this one was forked from recorded.
  pub fn probe(&self) -> XrayProbe<'_> {
    XrayProbe::over(&self.world, self.steps.clone(), self.recording.clone())
  }

  pub fn get_world(&self) -> &XrayVfs {
    &self.world
  }
}

impl XrayProbe<'_> {
  /// This probe's search over a fork of its world, which its holder owns.
  pub fn fork(&self) -> XrayOwnedProbe {
    XrayOwnedProbe {
      world: self.get_vfs().fork(),
      steps: self.get_steps().to_vec(),
      recording: self.get_recording().cloned(),
    }
  }
}

use std::borrow::Borrow;
use std::time::{Duration, Instant};

use crate::contract::report::load::render_load_durations::RenderLoadDurations;
use crate::contract::report::load::render_load_report::RenderLoadReport;
use crate::host::level::part::level_part::LevelPart;
use crate::host::world::feedback::render_streaming_progress::RenderStreamingProgress;
use crate::host::world::render_scene_id::RenderSceneId;
use crate::scene::texture::texture_cache::TextureCache;

/// What a level's load reports: how far its world streamed it, and whether every texture the renderer samples of it is
/// settled; and how long each part took.
pub struct LevelLoad {
  /// When its scene was made, which its load is timed from until the world says when it began opening the level.
  started: Instant,
  progress: RenderStreamingProgress,
  durations: RenderLoadDurations,
}

impl LevelLoad {
  pub fn new(started: Instant) -> Self {
    Self {
      started,
      progress: RenderStreamingProgress::default(),
      durations: RenderLoadDurations::default(),
    }
  }

  /// Takes how far the world has streamed the level this frame, and notes the parts finished since the last; `slots`
  /// are every texture slot it samples.
  pub fn advance(&mut self, progress: RenderStreamingProgress, slots: &[u32], textures: &TextureCache) {
    self.progress = progress;

    // Every part has finished by the time the whole has.
    if self.durations.ready.is_some() {
      return;
    }

    let elapsed: Duration = progress.started.unwrap_or(self.started).elapsed();

    for part in LevelPart::LOADED
      .into_iter()
      .filter(|part| progress.done.contains(*part))
    {
      if let Some(duration) = Self::get_duration(&mut self.durations, part) {
        duration.get_or_insert(elapsed);
      }
    }

    if self.is_ready(slots, textures) {
      self.durations.ready.get_or_insert(elapsed);
    }
  }

  /// Where the report times a part the load waits for; none for one it does not.
  fn get_duration(durations: &mut RenderLoadDurations, part: LevelPart) -> Option<&mut Option<Duration>> {
    match part {
      LevelPart::Sectors => Some(&mut durations.sectors),
      LevelPart::Spawn => Some(&mut durations.spawn),
      LevelPart::Lights => Some(&mut durations.lights),
      LevelPart::Grass => Some(&mut durations.grass),
      LevelPart::Particles => Some(&mut durations.particles),
      LevelPart::Helpers | LevelPart::Weather | LevelPart::Motions => None,
    }
  }

  /// How far the level has loaded: its sectors taken in or failed, its spawn, its grass, lights and particles read, and
  /// every texture it samples (`slots`) settled; and how long each took.
  pub fn describe(&self, scene: RenderSceneId, slots: &[u32], textures: &TextureCache) -> RenderLoadReport {
    let progress: &RenderStreamingProgress = &self.progress;
    let settled: u32 = textures.count_settled(slots.iter().copied());
    let total: u32 = slots.len() as u32;

    RenderLoadReport {
      scene: scene.0,
      sectors: progress.sectors,
      sectors_total: progress.sectors_total,
      bytes: progress.bytes,
      textures: settled,
      textures_total: total,
      is_ready: settled == total && self.are_parts_done(),
      durations: self.durations,
    }
  }

  /// Whether the level has stood ready once since its scene was made, as its load reports the moment.
  pub fn has_stood_ready(&self) -> bool {
    self.durations.ready.is_some()
  }

  /// Whether everything the level opens with is resident, so it draws as it will.
  pub fn is_ready(&self, slots: impl IntoIterator<Item = impl Borrow<u32>>, textures: &TextureCache) -> bool {
    textures.is_settled(slots) && self.are_parts_done()
  }

  /// What the level still waits for, said for a log: the parts not finished and the textures not settled.
  pub fn describe_pending(&self, slots: &[u32], textures: &TextureCache) -> String {
    let settled: u32 = textures.count_settled(slots.iter().copied());
    let mut pending: Vec<String> = LevelPart::LOADED
      .into_iter()
      .filter(|part| !self.progress.done.contains(*part))
      .map(|part| part.get_name().to_string())
      .collect();

    if settled < slots.len() as u32 {
      pending.push(format!("textures {settled} of {} settled", slots.len()));
    }

    pending.join(", ")
  }

  /// Whether every part the load waits for has finished (`LevelPart::LOADED`): done once every texture settles too.
  fn are_parts_done(&self) -> bool {
    self.progress.done.contains_all(&LevelPart::LOADED)
  }
}

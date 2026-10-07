use std::time::{Duration, Instant};

use crate::contract::render_load_durations::RenderLoadDurations;
use crate::contract::render_load_report::RenderLoadReport;
use crate::host::render_streaming_progress::RenderStreamingProgress;
use crate::scene::texture::texture_cache::TextureCache;

/// What a level's load reports: how far its world streamed it, and what the renderer holds of it, its grass and
/// particles read and every texture it samples settled; and how long each part took.
pub struct LevelLoad {
  /// When its scene was made, which its load is timed from until the world says when it began opening the level.
  started: Instant,
  progress: RenderStreamingProgress,
  durations: RenderLoadDurations,
  reported: Option<RenderLoadReport>,
}

impl LevelLoad {
  pub fn new(started: Instant) -> Self {
    Self {
      started,
      progress: RenderStreamingProgress::default(),
      durations: RenderLoadDurations::default(),
      reported: None,
    }
  }

  /// Takes how far the world has streamed the level this frame, and notes the parts finished since the last; `scene` is
  /// whether its grass was read and every texture slot it samples.
  pub fn advance(&mut self, progress: RenderStreamingProgress, scene: (bool, &[u32]), textures: &TextureCache) {
    self.progress = progress;

    // Every part has finished by the time the whole has.
    if self.durations.ready.is_some() {
      return;
    }

    let elapsed: Duration = progress.started.unwrap_or(self.started).elapsed();
    let finished: [bool; 6] = [
      progress.is_sectors_done,
      progress.is_spawn_done,
      scene.0,
      progress.is_lights_done,
      progress.is_particles_done,
      self.describe(scene, textures).is_ready,
    ];
    let RenderLoadDurations {
      sectors,
      spawn,
      grass,
      lights,
      particles,
      ready,
    } = &mut self.durations;

    for (duration, is_finished) in [sectors, spawn, grass, lights, particles, ready]
      .into_iter()
      .zip(finished)
    {
      if is_finished {
        duration.get_or_insert(elapsed);
      }
    }
  }

  /// How far the level has loaded: its sectors taken in or failed, its spawn, its grass, lights and particles read, and
  /// every texture it samples settled; and how long each took.
  pub fn describe(&self, (is_grass_read, slots): (bool, &[u32]), textures: &TextureCache) -> RenderLoadReport {
    let progress: &RenderStreamingProgress = &self.progress;
    let settled: u32 = textures.count_settled(slots.iter().copied());
    let total: u32 = slots.len() as u32;
    let is_read: bool =
      progress.is_spawn_done && progress.is_lights_done && progress.is_particles_done && is_grass_read;

    RenderLoadReport {
      sectors: progress.sectors,
      sectors_total: progress.sectors_total,
      bytes: progress.bytes,
      textures: settled,
      textures_total: total,
      is_ready: progress.is_sectors_done && is_read && settled == total,
      durations: self.durations,
    }
  }

  /// How far the level has loaded, when that changed since it was last asked.
  pub fn take_report(&mut self, scene: (bool, &[u32]), textures: &TextureCache) -> Option<RenderLoadReport> {
    let report: RenderLoadReport = self.describe(scene, textures);

    if self.reported == Some(report) {
      return None;
    }

    self.reported = Some(report);

    Some(report)
  }
}

use std::borrow::Borrow;
use std::time::{Duration, Instant};

use crate::contract::render_load_durations::RenderLoadDurations;
use crate::contract::render_load_report::RenderLoadReport;
use crate::host::render_scene_id::RenderSceneId;
use crate::host::render_streaming_progress::RenderStreamingProgress;
use crate::scene::texture::texture_cache::TextureCache;

/// What a level's load reports: how far its world streamed it, and what the renderer holds of it, its grass and
/// particles read and every texture it samples settled; and how long each part took.
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
      self.is_ready(scene.0, scene.1, textures),
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
  pub fn describe(
    &self,
    scene: RenderSceneId,
    (is_grass_read, slots): (bool, &[u32]),
    textures: &TextureCache,
  ) -> RenderLoadReport {
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
      is_ready: self.is_done(is_grass_read, settled == total),
      durations: self.durations,
    }
  }

  /// Whether everything the level opens with is resident, so it draws as it will.
  pub fn is_ready(
    &self,
    is_grass_read: bool,
    slots: impl IntoIterator<Item = impl Borrow<u32>>,
    textures: &TextureCache,
  ) -> bool {
    self.is_done(is_grass_read, textures.is_settled(slots))
  }

  /// What the level still waits for, said for a log: the parts not finished and the textures not settled.
  pub fn describe_pending(&self, (is_grass_read, slots): (bool, &[u32]), textures: &TextureCache) -> String {
    let settled: u32 = textures.count_settled(slots.iter().copied());
    let mut pending: Vec<String> = self
      .list_parts(is_grass_read)
      .iter()
      .filter(|(_, is_done)| !is_done)
      .map(|(part, _)| (*part).to_string())
      .collect();

    if settled < slots.len() as u32 {
      pending.push(format!("textures {settled} of {} settled", slots.len()));
    }

    pending.join(", ")
  }

  /// Whether every sector is taken in, the spawn, grass, lights and particles read, and every texture settled.
  fn is_done(&self, is_grass_read: bool, is_settled: bool) -> bool {
    is_settled && self.list_parts(is_grass_read).iter().all(|(_, is_done)| *is_done)
  }

  /// The parts the level opens with beside its textures, by name, and whether each has finished.
  fn list_parts(&self, is_grass_read: bool) -> [(&'static str, bool); 5] {
    let progress: &RenderStreamingProgress = &self.progress;

    [
      ("sectors", progress.is_sectors_done),
      ("spawn", progress.is_spawn_done),
      ("lights", progress.is_lights_done),
      ("particles", progress.is_particles_done),
      ("grass", is_grass_read),
    ]
  }
}

use std::borrow::Borrow;
use std::time::{Duration, Instant};

use crate::contract::render_load_durations::RenderLoadDurations;
use crate::contract::render_load_report::RenderLoadReport;
use crate::host::render_scene_id::RenderSceneId;
use crate::host::render_streaming_progress::RenderStreamingProgress;
use crate::scene::texture::texture_cache::TextureCache;

/// What a level's load reports: how far its world streamed it, whether every texture the renderer samples of it is
/// settled, and how many of its tree types' baked impostors are ready; and how long each part took.
pub struct LevelLoad {
  /// When its scene was made, which its load is timed from until the world says when it began opening the level.
  started: Instant,
  progress: RenderStreamingProgress,
  /// How many tree types' impostors are ready and how many the sectors gave, as `LevelImpostors::count_ready` last
  /// said; none before a view said how it draws them.
  impostors: Option<(u32, u32)>,
  durations: RenderLoadDurations,
}

impl LevelLoad {
  pub fn new(started: Instant) -> Self {
    Self {
      started,
      progress: RenderStreamingProgress::default(),
      impostors: None,
      durations: RenderLoadDurations::default(),
    }
  }

  /// Takes how far the world has streamed the level this frame and how many impostors are ready, and notes the parts
  /// finished since the last; `slots` are every texture slot it samples.
  pub fn advance(
    &mut self,
    (progress, impostors): (RenderStreamingProgress, Option<(u32, u32)>),
    slots: &[u32],
    textures: &TextureCache,
  ) {
    self.progress = progress;
    self.impostors = impostors;

    // Every part has finished by the time the whole has.
    if self.durations.ready.is_some() {
      return;
    }

    let elapsed: Duration = progress.started.unwrap_or(self.started).elapsed();
    let finished: [bool; 7] = [
      progress.is_sectors_done,
      progress.is_spawn_done,
      progress.is_grass_done,
      progress.is_lights_done,
      progress.is_particles_done,
      self.is_impostors_done(),
      self.is_ready(slots, textures),
    ];
    let RenderLoadDurations {
      sectors,
      spawn,
      grass,
      lights,
      particles,
      impostors,
      ready,
    } = &mut self.durations;

    for (duration, is_finished) in [sectors, spawn, grass, lights, particles, impostors, ready]
      .into_iter()
      .zip(finished)
    {
      if is_finished {
        duration.get_or_insert(elapsed);
      }
    }
  }

  /// How far the level has loaded: its sectors taken in or failed, its spawn, its grass, lights and particles read,
  /// every texture it samples (`slots`) settled, and its impostors ready; and how long each took.
  pub fn describe(&self, scene: RenderSceneId, slots: &[u32], textures: &TextureCache) -> RenderLoadReport {
    let progress: &RenderStreamingProgress = &self.progress;
    let settled: u32 = textures.count_settled(slots.iter().copied());
    let total: u32 = slots.len() as u32;
    let (impostors, impostors_total): (u32, u32) = self.impostors.unwrap_or_default();

    RenderLoadReport {
      scene: scene.0,
      sectors: progress.sectors,
      sectors_total: progress.sectors_total,
      bytes: progress.bytes,
      textures: settled,
      textures_total: total,
      impostors,
      impostors_total,
      is_ready: self.is_done(settled == total),
      durations: self.durations,
    }
  }

  /// Whether everything the level opens with is resident, so it draws as it will.
  pub fn is_ready(&self, slots: impl IntoIterator<Item = impl Borrow<u32>>, textures: &TextureCache) -> bool {
    self.is_done(textures.is_settled(slots))
  }

  /// Whether everything the level opens with but its impostors is resident: what a bake waits for, so it draws the
  /// trees with the textures the level will.
  pub fn is_resident(&self, slots: impl IntoIterator<Item = impl Borrow<u32>>, textures: &TextureCache) -> bool {
    textures.is_settled(slots) && self.list_parts().iter().all(|(_, is_done)| *is_done)
  }

  /// What the level still waits for, said for a log: the parts not finished, the textures not settled and the
  /// impostors not ready.
  pub fn describe_pending(&self, slots: &[u32], textures: &TextureCache) -> String {
    let settled: u32 = textures.count_settled(slots.iter().copied());
    let mut pending: Vec<String> = self
      .list_parts()
      .iter()
      .filter(|(_, is_done)| !is_done)
      .map(|(part, _)| (*part).to_string())
      .collect();

    if settled < slots.len() as u32 {
      pending.push(format!("textures {settled} of {} settled", slots.len()));
    }

    match self.impostors {
      None => pending.push(String::from("impostors")),
      Some((ready, total)) if ready < total => pending.push(format!("impostors {ready} of {total} ready")),
      Some(_) => {}
    }

    pending.join(", ")
  }

  /// Whether every sector is taken in, the spawn, grass, lights and particles read, every texture settled, and every
  /// impostor ready.
  fn is_done(&self, is_settled: bool) -> bool {
    is_settled && self.is_impostors_done() && self.list_parts().iter().all(|(_, is_done)| *is_done)
  }

  /// Whether every tree type the sectors gave has its impostor, or none is baked: only once every sector is in, since
  /// before then nought ready of the nought types given says nothing.
  fn is_impostors_done(&self) -> bool {
    self.progress.is_sectors_done && self.impostors.is_some_and(|(ready, total)| ready >= total)
  }

  /// The parts the level opens with beside its textures and impostors, by name, and whether each has finished.
  fn list_parts(&self) -> [(&'static str, bool); 5] {
    let progress: &RenderStreamingProgress = &self.progress;

    [
      ("sectors", progress.is_sectors_done),
      ("spawn", progress.is_spawn_done),
      ("lights", progress.is_lights_done),
      ("particles", progress.is_particles_done),
      ("grass", progress.is_grass_done),
    ]
  }
}

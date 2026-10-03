use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};

use glam::Mat4;

use crate::contract::render_load_failure::RenderLoadFailure;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_level_spawn::RenderLevelSpawn;
use crate::host::render_spawn_models::RenderSpawnModels;
use crate::host::render_spawn_object::RenderSpawnObject;
use crate::scene::static_scene::static_model::StaticModel;
use crate::scene::static_scene::static_model_place::StaticModelPlace;

/// Visuals a loader thread reads in one batch: enough that the level's lighting is estimated for many at once, few
/// enough that objects appear while the rest are read.
const VISUALS_PER_BATCH: usize = 16;

/// One model packed for the scene, with every object standing as it.
pub type SpawnLoad = (StaticModel, Vec<StaticModelPlace>);

/// Reads a level's spawn on the loader threads: the objects once, then their visuals in batches side by side, each
/// model packed and handed to the render thread with the objects standing as it. Dropped, it lets go of the batches not
/// started yet.
pub struct SpawnLoader {
  receiver: Receiver<SpawnLoad>,
  /// Batches not finished yet, and whether the objects are still being read.
  pending: Arc<AtomicU32>,
  /// Models sent, and taken.
  sent: Arc<AtomicU32>,
  taken: u32,
  is_cancelled: Arc<AtomicBool>,
  /// The visuals that could not be read.
  failures: Arc<Mutex<Vec<RenderLoadFailure>>>,
}

impl SpawnLoader {
  pub fn start(source: Arc<dyn RenderLevelSource>) -> Self {
    let (sender, receiver) = channel();
    let pending: Arc<AtomicU32> = Arc::new(AtomicU32::new(1));
    let sent: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    let is_cancelled: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let failures: Arc<Mutex<Vec<RenderLoadFailure>>> = Arc::new(Mutex::new(Vec::new()));
    let counters: Counters = Counters {
      pending: Arc::clone(&pending),
      sent: Arc::clone(&sent),
      is_cancelled: Arc::clone(&is_cancelled),
      failures: Arc::clone(&failures),
    };
    let spawn_pending: Arc<AtomicU32> = Arc::clone(&pending);

    rayon::spawn(move || {
      match source.read_spawn() {
        Ok(spawn) => read_batches(&source, Arc::new(spawn), &sender, &counters),
        Err(error) => log::warn!("The level's spawned objects cannot be drawn: {error}"),
      }

      spawn_pending.fetch_sub(1, Ordering::AcqRel);
    });

    Self {
      receiver,
      pending,
      sent,
      taken: 0,
      is_cancelled,
      failures,
    }
  }

  /// The visuals that could not be read so far.
  pub fn list_failures(&self) -> Vec<RenderLoadFailure> {
    self.failures.lock().unwrap_or_else(PoisonError::into_inner).clone()
  }

  /// Whether every batch has been read and every model it packed taken.
  pub fn is_done(&self) -> bool {
    self.pending.load(Ordering::Acquire) == 0 && self.sent.load(Ordering::Acquire) == self.taken
  }

  /// The models finished since the last call, at most `limit` of them.
  pub fn take(&mut self, limit: usize) -> Vec<SpawnLoad> {
    let taken: Vec<SpawnLoad> = self.receiver.try_iter().take(limit).collect();

    self.taken += taken.len() as u32;

    taken
  }
}

impl Drop for SpawnLoader {
  fn drop(&mut self) {
    self.is_cancelled.store(true, Ordering::Release);
  }
}

/// What the loader's threads count by, shared with the loader.
#[derive(Clone)]
struct Counters {
  pending: Arc<AtomicU32>,
  sent: Arc<AtomicU32>,
  is_cancelled: Arc<AtomicBool>,
  failures: Arc<Mutex<Vec<RenderLoadFailure>>>,
}

impl Counters {
  fn fail(&self, failures: impl IntoIterator<Item = RenderLoadFailure>) {
    self
      .failures
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .extend(failures);
  }
}

fn read_batches(
  source: &Arc<dyn RenderLevelSource>,
  spawn: Arc<RenderLevelSpawn>,
  sender: &Sender<SpawnLoad>,
  counters: &Counters,
) {
  for (batch, names) in spawn.visuals.chunks(VISUALS_PER_BATCH).enumerate() {
    let (source, spawn, sender, counters) = (Arc::clone(source), Arc::clone(&spawn), sender.clone(), counters.clone());
    let (names, first) = (names.to_vec(), batch * VISUALS_PER_BATCH);

    counters.pending.fetch_add(1, Ordering::AcqRel);
    rayon::spawn(move || {
      if !counters.is_cancelled.load(Ordering::Acquire) {
        match source.read_spawn_models(&names) {
          Ok(mut models) => {
            counters.fail(std::mem::take(&mut models.failures));
            send_models(&spawn, first, models, &sender, &counters.sent);
          }
          Err(error) => {
            log::warn!("Spawned models cannot be drawn: {error}");
            counters.fail(names.iter().map(|name| RenderLoadFailure {
              name: name.clone(),
              reason: error.to_string(),
            }));
          }
        }
      }

      counters.pending.fetch_sub(1, Ordering::AcqRel);
    });
  }
}

/// Packs each model read and sends it with the objects standing as it, lit by their cubes where they have them.
fn send_models(
  spawn: &RenderLevelSpawn,
  first: usize,
  models: RenderSpawnModels,
  sender: &Sender<SpawnLoad>,
  sent: &AtomicU32,
) {
  let lighting: HashMap<u32, ([f32; 6], f32)> = models
    .lighting
    .into_iter()
    .map(|it| (it.object, (it.cube, it.sky)))
    .collect();

  for model in models.models {
    let Some(visual) = spawn
      .visuals
      .iter()
      .skip(first)
      .position(|name| *name == model.name)
      .map(|it| (it + first) as u32)
    else {
      continue;
    };
    let places: Vec<StaticModelPlace> = spawn
      .objects
      .iter()
      .filter(|object| object.visual == visual)
      .map(|object: &RenderSpawnObject| StaticModelPlace {
        object: object.index,
        transform: Mat4::from_cols_array(&object.transform),
        lighting: lighting.get(&object.index).copied(),
        group: object.category.get_group(),
      })
      .collect();
    let packed: StaticModel = StaticModel::pack(&model.name, &model.package, &model.surfaces, visual as u16);

    // Counted before it is sent, so the loader never sees it taken before it was sent.
    sent.fetch_add(1, Ordering::AcqRel);

    let _ = sender.send((packed, places));
  }
}

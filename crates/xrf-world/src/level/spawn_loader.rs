use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, PoisonError};

use glam::Mat4;
use xrf_renderer::{
  LoaderReceiver, RenderLevelSource, RenderLevelSpawn, RenderLoadFailure, RenderModelSkeleton, RenderSpawnModels,
  RenderSpawnObject, RenderWorkers, StaticModel, StaticModelPlace,
};

/// Visuals a loader thread reads in one batch: enough that the level's lighting is estimated for many at once, few
/// enough that objects appear while the rest are read.
const VISUALS_PER_BATCH: usize = 16;

/// One model packed for the scene, with every object standing as it, and the skeleton it is posed by where the
/// renderer poses it.
pub type SpawnLoad = (StaticModel, Vec<StaticModelPlace>, Option<RenderModelSkeleton>);

/// Reads a level's spawn on the loader threads: the objects once, then their visuals in batches side by side, each
/// model packed and handed to the render thread with the objects standing as it. Dropped, it lets go of the batches not
/// started yet.
pub struct SpawnLoader {
  receiver: LoaderReceiver<SpawnLoad>,
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
  pub fn start(source: Arc<dyn RenderLevelSource>, workers: &RenderWorkers) -> Self {
    let (sender, receiver) = LoaderReceiver::channel();
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

    let batch_workers: RenderWorkers = workers.clone();

    workers.spawn(move || {
      match source.read_spawn() {
        Ok(spawn) => read_batches(&source, Arc::new(spawn), &sender, &counters, &batch_workers),
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
  workers: &RenderWorkers,
) {
  for (batch, names) in spawn.visuals.chunks(VISUALS_PER_BATCH).enumerate() {
    let (source, spawn, sender, counters) = (Arc::clone(source), Arc::clone(&spawn), sender.clone(), counters.clone());
    let (names, first) = (names.to_vec(), batch * VISUALS_PER_BATCH);

    counters.pending.fetch_add(1, Ordering::AcqRel);
    workers.spawn(move || {
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
        group: if object.is_released {
          object.category.get_released_group()
        } else {
          object.category.get_group()
        },
      })
      .collect();
    let packed: StaticModel = StaticModel::pack_at(&model.package, &model.surfaces, visual as u16, spawn.detail);

    // Counted before it is sent, so the loader never sees it taken before it was sent.
    sent.fetch_add(1, Ordering::AcqRel);

    let _ = sender.send((packed, places, model.skeleton));
  }
}

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, PoisonError};

use glam::Mat4;
use xrf_renderer::{
  LoaderReceiver, RenderLevelSource, RenderLevelSpawn, RenderLoadFailure, RenderModelSkeleton, RenderSpawnModels,
  RenderSpawnObject, RenderWorkers, StaticModel, StaticModelPlace, take_answer,
};

use crate::level::loader_flight::LoaderFlight;

/// Visuals a loader thread reads in one batch: enough that the level's lighting is estimated for many at once, few
/// enough that objects appear while the rest are read.
const VISUALS_PER_BATCH: usize = 16;

/// One model packed for the scene, the visual it was read from, every object standing as it, and the skeleton it is
/// posed by where the renderer poses it.
pub type SpawnLoad = (StaticModel, String, Vec<StaticModelPlace>, Option<RenderModelSkeleton>);

/// Reads a level's spawn on the loader threads: the objects once, then their visuals in batches side by side, no more
/// than the pool's width of them running or queued at once (`LoaderFlight`), each model packed and handed to the render
/// thread with the objects standing as it; the next batches start as the render thread takes models, once a frame.
/// Dropped, it lets go of the batches not started yet.
pub struct SpawnLoader {
  source: Arc<dyn RenderLevelSource>,
  workers: RenderWorkers,
  /// The objects on their way from their loader, none once they came or could not be read.
  reading: Option<LoaderReceiver<Option<RenderLevelSpawn>>>,
  /// The objects once read, whose visuals the batches read.
  spawn: Option<Arc<RenderLevelSpawn>>,
  sender: Sender<SpawnLoad>,
  receiver: LoaderReceiver<SpawnLoad>,
  /// The batches started, in order, and how many of them finished.
  flight: LoaderFlight,
  /// Models sent, and taken.
  sent: Arc<AtomicU32>,
  taken: u32,
  is_cancelled: Arc<AtomicBool>,
  /// The visuals that could not be read.
  failures: Arc<Mutex<Vec<RenderLoadFailure>>>,
}

impl SpawnLoader {
  pub fn start(source: Arc<dyn RenderLevelSource>, workers: &RenderWorkers) -> Self {
    let (spawn_sender, spawn_receiver) = LoaderReceiver::channel();
    let (sender, receiver) = LoaderReceiver::channel();
    let reader: Arc<dyn RenderLevelSource> = Arc::clone(&source);

    workers.spawn(move || {
      let spawn: Option<RenderLevelSpawn> = reader
        .read_spawn()
        .inspect_err(|error| log::warn!("The level's spawned objects cannot be drawn: {error}"))
        .ok();

      let _ = spawn_sender.send(spawn);
    });

    Self {
      source,
      workers: workers.clone(),
      reading: Some(spawn_receiver),
      spawn: None,
      sender,
      receiver,
      flight: LoaderFlight::new(workers),
      sent: Arc::new(AtomicU32::new(0)),
      taken: 0,
      is_cancelled: Arc::new(AtomicBool::new(false)),
      failures: Arc::new(Mutex::new(Vec::new())),
    }
  }

  /// The visuals that could not be read so far.
  pub fn list_failures(&self) -> Vec<RenderLoadFailure> {
    self.failures.lock().unwrap_or_else(PoisonError::into_inner).clone()
  }

  /// Whether the objects were read, every batch of their visuals has been read and every model it packed taken.
  pub fn is_done(&self) -> bool {
    self.reading.is_none()
      && self.get_batch_count() == self.flight.get_started() as usize
      && self.flight.is_landed()
      && self.sent.load(Ordering::Acquire) == self.taken
  }

  /// The models finished since the last call, at most `limit` of them; takes the objects once read and starts as many
  /// more batches as finished.
  pub fn take(&mut self, limit: usize) -> Vec<SpawnLoad> {
    if let Some(spawn) = take_answer(&mut self.reading, "spawn") {
      self.spawn = spawn.map(Arc::new);
    }

    let taken: Vec<SpawnLoad> = self.receiver.try_iter().take(limit).collect();

    self.taken += taken.len() as u32;
    self.refill();

    taken
  }

  /// How many batches the objects' visuals make, none until they are read.
  fn get_batch_count(&self) -> usize {
    self
      .spawn
      .as_ref()
      .map_or(0, |spawn| spawn.visuals.len().div_ceil(VISUALS_PER_BATCH))
  }

  /// Starts the next batches while the pool's width of them are not read.
  fn refill(&mut self) {
    let Some(spawn) = self.spawn.clone() else {
      return;
    };
    let count: usize = self.get_batch_count();

    while (self.flight.get_started() as usize) < count
      && let Some(finished) = self.flight.start()
    {
      let first: usize = (self.flight.get_started() as usize - 1) * VISUALS_PER_BATCH;
      let names: Vec<String> = spawn.visuals[first..(first + VISUALS_PER_BATCH).min(spawn.visuals.len())].to_vec();
      let (source, spawn, sender) = (Arc::clone(&self.source), Arc::clone(&spawn), self.sender.clone());
      let (sent, is_cancelled, failures) = (
        Arc::clone(&self.sent),
        Arc::clone(&self.is_cancelled),
        Arc::clone(&self.failures),
      );

      self.workers.spawn(move || {
        if !is_cancelled.load(Ordering::Acquire) {
          match source.read_spawn_models(&names) {
            Ok(mut models) => {
              failures
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .extend(std::mem::take(&mut models.failures));
              Self::send_models(&spawn, first, models, &sender, &sent);
            }
            Err(error) => {
              log::warn!("Spawned models cannot be drawn: {error}");
              failures
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .extend(names.iter().map(|name| RenderLoadFailure {
                  name: name.clone(),
                  reason: error.to_string(),
                }));
            }
          }
        }

        finished.fetch_add(1, Ordering::AcqRel);
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

      let _ = sender.send((packed, model.name, places, model.skeleton));
    }
  }
}

impl Drop for SpawnLoader {
  fn drop(&mut self) {
    self.is_cancelled.store(true, Ordering::Release);
  }
}

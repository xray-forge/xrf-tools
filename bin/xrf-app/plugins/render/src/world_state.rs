use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use xrf_app_core::execution::ExecutionState;
use xrf_app_core::types::TauriResult;
use xrf_renderer::{RenderWorkers, RenderWorld};
use xrf_world::World;

/// The world the renderer draws, which the camera, weather and show commands steer; the render thread holds it while it
/// advances every viewport and scene before a frame. A clone steers the same world.
#[derive(Clone)]
pub struct WorldState {
  world: Arc<Mutex<World>>,
}

impl WorldState {
  /// A world whose levels stream on `workers`.
  pub fn new(workers: RenderWorkers) -> Self {
    Self {
      world: Arc::new(Mutex::new(World::new(workers))),
    }
  }

  /// The same world as the render thread advances it.
  pub fn as_render_world(&self) -> Arc<Mutex<dyn RenderWorld>> {
    Arc::clone(&self.world) as Arc<Mutex<dyn RenderWorld>>
  }

  /// The world, locked for a command.
  pub fn lock(&self) -> MutexGuard<'_, World> {
    profiling::scope!("lock world");

    self.world.lock().unwrap_or_else(PoisonError::into_inner)
  }

  /// Runs `read` over the world on the execution pool, so a command waiting for the render thread's frame to let go of
  /// it holds no async worker meanwhile.
  pub async fn read<T, F>(&self, execution: &ExecutionState, what: &str, read: F) -> TauriResult<T>
  where
    F: FnOnce(&World) -> T + Send + 'static,
    T: Send + 'static,
  {
    let world: WorldState = self.clone();

    execution.run_blocking(what, move || read(&world.lock())).await
  }
}

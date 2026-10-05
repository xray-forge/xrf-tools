use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::TryRecvError;

use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_motion::RenderMotion;
use crate::thread::loader_receiver::LoaderReceiver;
use crate::thread::render_workers::RenderWorkers;

/// One motion asked for: on its way from a loader thread, baked, or not to be had.
enum MotionState {
  Reading(LoaderReceiver<Option<RenderMotion>>),
  Baked(RenderMotion),
  Missing,
}

/// The motions a viewport's skinned models are posed by, each baked once on a loader thread the first time a pose
/// names it, and kept while the viewport shows the same scene.
pub struct ModelMotions {
  motions: HashMap<String, MotionState>,
  workers: RenderWorkers,
}

impl ModelMotions {
  pub fn new(workers: &RenderWorkers) -> Self {
    Self {
      motions: HashMap::new(),
      workers: workers.clone(),
    }
  }

  /// The motion by its name once it is baked, asking for it the first time; none while it is read or where it cannot
  /// be, which the caller poses the bind pose for.
  pub fn get(&mut self, source: &Arc<dyn RenderLevelSource>, name: &str) -> Option<&RenderMotion> {
    let workers: &RenderWorkers = &self.workers;
    let state: &mut MotionState = self.motions.entry(name.to_owned()).or_insert_with(|| {
      let (sender, receiver) = LoaderReceiver::channel();
      let (source, name) = (Arc::clone(source), name.to_owned());

      workers.spawn(move || {
        let motion: Option<RenderMotion> = match source.read_motion(&name) {
          Ok(motion) => Some(motion),
          Err(error) => {
            log::warn!("Motion '{name}' cannot be played: {error}");

            None
          }
        };

        let _ = sender.send(motion);
      });

      MotionState::Reading(receiver)
    });

    if let MotionState::Reading(receiver) = state {
      *state = match receiver.try_recv() {
        Ok(Some(motion)) => MotionState::Baked(motion),
        Ok(None) | Err(TryRecvError::Disconnected) => MotionState::Missing,
        Err(TryRecvError::Empty) => return None,
      };
    }

    match state {
      MotionState::Baked(motion) => Some(motion),
      _ => None,
    }
  }
}

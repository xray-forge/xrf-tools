use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use xrf_logging::ProcessStart;

use crate::load::game_load_stage::GameLoadStage;
use crate::load::game_load_stage_state::GameLoadStageState;

/// How far a load has come, stage by stage, shared by whoever does its work, on any thread, and the loading view that
/// shows it: each stage begins, may say how far along it is, and ends, and each counts an equal share of the whole, as
/// X-Ray's loading screen counts its stages. Each stage's time is logged as it ends.
#[derive(Debug)]
pub(crate) struct GameLoadProgress {
  stages: Mutex<Vec<GameLoadStageState>>,
}

impl GameLoadProgress {
  /// A load taking `stages`, none begun.
  pub(crate) fn new(stages: &[GameLoadStage]) -> Self {
    Self {
      stages: Mutex::new(
        stages
          .iter()
          .map(|stage| GameLoadStageState {
            stage: *stage,
            began: None,
            ended: None,
            fraction: 0.0,
          })
          .collect(),
      ),
    }
  }

  /// Runs `work` as `stage`, begun before it and ended after it, whatever it answers.
  pub(crate) fn run<T>(&self, stage: GameLoadStage, work: impl FnOnce() -> T) -> T {
    self.begin(stage);

    let done: T = work();

    self.finish(stage);

    done
  }

  /// Marks `stage` begun now; again, or for a stage the load does not take, nothing.
  pub(crate) fn begin(&self, stage: GameLoadStage) {
    if let Some(state) = self.lock().iter_mut().find(|state| state.stage == stage)
      && state.began.is_none()
    {
      state.began = Some(Instant::now());
    }
  }

  /// Marks `stage` `fraction` along, from 0 to 1, begun where it was not.
  pub(crate) fn advance(&self, stage: GameLoadStage, fraction: f32) {
    self.begin(stage);

    if let Some(state) = self.lock().iter_mut().find(|state| state.stage == stage) {
      state.fraction = fraction;
    }
  }

  /// Marks `stage` ended now and logs how long it took; again, or for a stage the load does not take, nothing.
  pub(crate) fn finish(&self, stage: GameLoadStage) {
    let mut stages: MutexGuard<'_, Vec<GameLoadStageState>> = self.lock();
    let Some(state) = stages.iter_mut().find(|state| state.stage == stage) else {
      return;
    };

    if state.ended.is_some() {
      return;
    }

    let now: Instant = Instant::now();
    let began: Instant = *state.began.get_or_insert(now);

    state.ended = Some(now);
    log::info!(
      "Loading: {} took {:.1?}, done {:.1?} after launch",
      stage.get_title(),
      now.duration_since(began),
      ProcessStart::get_uptime()
    );
  }

  /// Whether every stage ended.
  pub(crate) fn is_done(&self) -> bool {
    self.lock().iter().all(|state| state.ended.is_some())
  }

  /// How much of the load is done, from 0 to 1: each stage an equal share, a running one its fraction.
  pub(crate) fn get_progress(&self) -> f32 {
    let stages: MutexGuard<'_, Vec<GameLoadStageState>> = self.lock();

    if stages.is_empty() {
      return 1.0;
    }

    stages.iter().map(GameLoadStageState::get_share).sum::<f32>() / stages.len() as f32
  }

  /// The stages running now, in the load's order.
  pub(crate) fn list_running(&self) -> Vec<GameLoadStage> {
    self
      .lock()
      .iter()
      .filter(|state| state.is_running())
      .map(|state| state.stage)
      .collect()
  }

  fn lock(&self) -> MutexGuard<'_, Vec<GameLoadStageState>> {
    self.stages.lock().unwrap_or_else(PoisonError::into_inner)
  }
}

use std::sync::Arc;
use std::time::{Duration, Instant};

use rayon::{ThreadPool, ThreadPoolBuilder};
use xrf_game_movement::MovementMode;

use crate::load::game_load_progress::GameLoadProgress;
use crate::load::game_load_stage::GameLoadStage;
use crate::load::game_opened::GameOpened;
use crate::load::game_opening::GameOpening;
use crate::run::game_flow::GameFlow;
use crate::run::game_flow_stage::GameFlowStage;
use crate::session::game_load_request::GameLoadRequest;
use crate::session::game_load_source::GameLoadSource;

/// Longer than any of these waits should take, so a broken contract fails rather than hangs.
const WAIT: Duration = Duration::from_secs(10);

/// A load of a tree that is not there, which fails once its thread tries to mount it.
fn request() -> GameLoadRequest {
  GameLoadRequest {
    tree: std::env::temp_dir().join(format!("xrf-game-no-tree-{}", std::process::id())),
    level: String::from("zaton"),
    game: GameLoadSource::New { seed: 7 },
    mode: MovementMode::Fly,
    recording: None,
  }
}

fn pool() -> Arc<ThreadPool> {
  Arc::new(ThreadPoolBuilder::new().num_threads(2).build().expect("a pool"))
}

#[test]
fn launches_into_loading_with_no_game_running_while_it_opens() {
  let progress: Arc<GameLoadProgress> = Arc::new(GameLoadProgress::new(&GameLoadStage::LAUNCH));
  let flow: GameFlow = GameFlow::launch(&request(), pool(), progress).expect("the load starts");

  assert_eq!(flow.get_stage(), GameFlowStage::Loading);
  assert!(flow.get_running().is_none());
}

#[test]
fn tells_why_a_game_did_not_open() {
  let progress: Arc<GameLoadProgress> = Arc::new(GameLoadProgress::new(&GameLoadStage::LAUNCH));
  let opening: GameOpening = GameOpening::start(request(), pool(), Arc::clone(&progress)).expect("the load starts");
  let deadline: Instant = Instant::now() + WAIT;

  let error: String = loop {
    let session = opening.take().into_iter().find_map(|event| match event {
      GameOpened::Session(session) => Some(*session),
      GameOpened::Picture(_) => None,
    });

    if let Some(session) = session {
      break session
        .err()
        .expect("a tree that is not there opens nothing")
        .to_string();
    }

    assert!(Instant::now() < deadline, "the opening never answered");
    std::thread::sleep(Duration::from_millis(5));
  };

  assert!(!error.is_empty());
  // The tree's stage ended with its failure rather than running on.
  assert!(!progress.list_running().contains(&GameLoadStage::Tree));
}

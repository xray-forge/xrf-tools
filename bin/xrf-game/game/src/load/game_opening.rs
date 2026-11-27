use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::task::Waker;
use std::thread;

use rayon::ThreadPool;
use xrf_error::{XrfError, XrfResult};
use xrf_game_developer::DeveloperPicture;

use crate::host::game_tree::GameTree;
use crate::level::game_loading_picture::GameLoadingPicture;
use crate::load::game_load_progress::GameLoadProgress;
use crate::load::game_load_stage::GameLoadStage;
use crate::load::game_opened::GameOpened;
use crate::session::game_load_request::GameLoadRequest;
use crate::session::game_load_source::GameLoadSource;
use crate::session::game_session::GameSession;

/// A game opening on a thread of its own while the main thread draws the loading view: the tree mounted, its level's
/// picture sent, then the session opened from the request, its reads on the pool. Dropped before it ends, its thread
/// finishes alone and what it opened is let go.
pub(crate) struct GameOpening {
  events: Receiver<GameOpened>,
}

impl GameOpening {
  /// Starts opening what `request` asks, reading on `pool`, each stage reported to `progress`, the main thread woken
  /// through `waker` to each thing it tells.
  ///
  /// # Errors
  ///
  /// Returns why the thread cannot be started.
  pub(crate) fn start(
    request: GameLoadRequest,
    pool: Arc<ThreadPool>,
    progress: Arc<GameLoadProgress>,
    waker: Waker,
  ) -> XrfResult<Self> {
    let (sender, events) = mpsc::channel::<GameOpened>();

    thread::Builder::new()
      .name(String::from("xrf-game-open"))
      .spawn(move || {
        let tell = |opened: GameOpened| {
          // A main thread that stopped waiting has let the load go.
          if sender.send(opened).is_ok() {
            waker.wake_by_ref();
          }
        };

        Self::open(&request, &pool, &progress, &tell);
      })?;

    Ok(Self { events })
  }

  /// What the opening told since last asked, in order; a thread that ended without its session tells it panicked.
  pub(crate) fn take(&self) -> Vec<GameOpened> {
    let mut opened: Vec<GameOpened> = Vec::new();

    loop {
      match self.events.try_recv() {
        Ok(event) => opened.push(event),
        Err(TryRecvError::Empty) => return opened,
        Err(TryRecvError::Disconnected) => {
          let is_answered: bool = opened.iter().any(|event| matches!(event, GameOpened::Session(_)));

          if !is_answered {
            opened.push(GameOpened::Session(Box::new(Err(XrfError::new_unexpected_error(
              "opening the game panicked",
            )))));
          }

          return opened;
        }
      }
    }
  }

  /// Opens what `request` asks and tells each thing opened through `tell`, the session last.
  fn open(request: &GameLoadRequest, pool: &ThreadPool, progress: &GameLoadProgress, tell: &dyn Fn(GameOpened)) {
    let session: XrfResult<GameSession> = progress
      .run(GameLoadStage::Tree, || GameTree::mount(&request.tree))
      .and_then(|tree| {
        log::info!(
          "Game tree {} mounted, engine {:?}",
          request.tree.display(),
          tree.get_engine().engine
        );

        let GameLoadSource::New { seed } = request.game;
        let picture: Option<DeveloperPicture> = tree
          .search()
          .ok()
          .and_then(|search| GameLoadingPicture::read(&search, &request.level, seed));

        if let Some(picture) = picture {
          tell(GameOpened::Picture(picture));
        }

        GameSession::open(request, tree, pool, progress)
      });

    tell(GameOpened::Session(Box::new(session)));
  }
}

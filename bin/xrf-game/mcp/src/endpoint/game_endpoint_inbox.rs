use std::sync::mpsc::Sender;
use std::task::Waker;

use crate::wire::game_request::GameRequest;

/// Where the endpoint's reader puts each request for the game: the channel the game takes them from, and the waker
/// that ends the game's rest as one arrives, so a request never waits for the game to look.
pub(crate) struct GameEndpointInbox {
  pub sender: Sender<GameRequest>,
  pub waker: Waker,
}

impl GameEndpointInbox {
  /// Hands `request` to the game and wakes it, answering whether the game still takes requests.
  pub(crate) fn deliver(&self, request: GameRequest) -> bool {
    let is_taken: bool = self.sender.send(request).is_ok();

    if is_taken {
      self.waker.wake_by_ref();
    }

    is_taken
  }
}

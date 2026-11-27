use crate::bridge::game_attached::GameAttached;
use crate::bridge::game_channel::GameChannel;
use crate::bridge::game_child::GameChild;

/// The game the bridge drives: one it started as its child, or one started by hand it attached to.
pub(crate) enum GameLink {
  Child(GameChild),
  Attached(GameAttached),
}

impl GameLink {
  /// Whether the game can still be asked: a child that still runs; an attached game until a request finds it gone.
  pub(crate) fn is_running(&mut self) -> bool {
    match self {
      Self::Child(child) => child.is_running(),
      Self::Attached(_) => true,
    }
  }

  pub(crate) fn get_channel_mut(&mut self) -> &mut GameChannel {
    match self {
      Self::Child(child) => &mut child.channel,
      Self::Attached(attached) => &mut attached.channel,
    }
  }
}

use std::io::Write;
use std::net::{Ipv4Addr, Shutdown, TcpStream};
use std::time::Duration;

use xrf_error::XrfResult;

use crate::bridge::game_channel::GameChannel;
use crate::wire::game_attach::GameAttach;

/// A game started by hand that the bridge attached to, the line protocol over the loopback connection its `--listen`
/// opened. Leaving shuts the connection down, which the game reads as the bridge gone, and quits nothing: the next
/// bridge attaches.
pub(crate) struct GameAttached {
  pub(crate) attach: GameAttach,
  pub(crate) channel: GameChannel,
  /// The connection itself, shut down on leaving: the channel's reader holds a copy of it, which dropping the writer's
  /// alone would leave open.
  stream: TcpStream,
}

impl GameAttached {
  /// How long connecting to the game waits.
  const CONNECT_WAIT: Duration = Duration::from_secs(5);

  /// Connects to the game `attach` describes and sends its token.
  ///
  /// # Errors
  ///
  /// Returns why it cannot connect: the game is gone, or its port refuses.
  pub(crate) fn connect(attach: GameAttach) -> XrfResult<Self> {
    let mut stream: TcpStream =
      TcpStream::connect_timeout(&(Ipv4Addr::LOCALHOST, attach.port).into(), Self::CONNECT_WAIT)?;

    writeln!(stream, "{}", attach.token).and_then(|()| stream.flush())?;

    let channel: GameChannel = GameChannel::new(
      Box::new(stream.try_clone()?),
      stream.try_clone()?,
      "xrf-game-mcp-attached",
    )?;

    Ok(Self {
      attach,
      channel,
      stream,
    })
  }

  /// Shuts the connection down, so the game takes the next bridge.
  pub(crate) fn leave(&mut self) {
    self.channel.close_input();

    if let Err(error) = self.stream.shutdown(Shutdown::Both) {
      log::debug!("The connection to the game was closed already: {error}");
    }
  }
}

impl Drop for GameAttached {
  fn drop(&mut self) {
    self.leave();
  }
}

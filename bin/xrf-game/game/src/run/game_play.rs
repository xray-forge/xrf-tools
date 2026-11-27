use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_game_platform::{Platform, PlatformEvent, PlatformEventKind, PlatformScancode};
use xrf_game_settings::{GameSettingsChanges, GameSettingsGroup};

use crate::layer::game_layer_context::GameLayerContext;
use crate::layer::game_layers::GameLayers;
use crate::run::game_cursor::GameCursor;
use crate::run::game_startup::GameStartup;

/// The main thread from the window's opening to the game's end, as a chain of presentation layers over what they
/// share. Each frame it waits for the platform's events until a layer's next work is due or another thread wakes it
/// (the loop, the load, the control endpoint's reader, the renderer), offers each event down the
/// chain (what no layer keeps is the window's own: its close, its resize, the quit key), moves the flow on (a load's
/// view while it runs, then the game played) and every layer after it, and has them draw, until the player, a console
/// line or an agent quits, or the loop stops stepping.
pub(crate) struct GamePlay<'a> {
  /// What every layer acts on: the window, the view, the settings, the console, the flow.
  context: GameLayerContext<'a>,
  /// The layers, topmost first.
  layers: GameLayers<GameLayerContext<'a>>,
  /// How the hidden window first shows.
  startup: GameStartup,
}

impl<'a> GamePlay<'a> {
  /// The key that quits until the game has a menu: Escape's place.
  const QUIT_KEY: PlatformScancode = PlatformScancode(41);

  /// How often the main thread looks whether the hidden window may show: the frames it waits to see drawn wake nobody,
  /// as one comes many times a second for the whole run and only the first few matter.
  const HIDDEN_WAIT: Duration = Duration::from_millis(4);

  /// Play of `layers` over `context`, the window still hidden.
  pub(crate) fn new(context: GameLayerContext<'a>, layers: GameLayers<GameLayerContext<'a>>) -> Self {
    Self {
      context,
      layers,
      startup: GameStartup::new(),
    }
  }

  /// What every layer acts on, as play leaves it.
  pub(crate) fn get_context_mut(&mut self) -> &mut GameLayerContext<'a> {
    &mut self.context
  }

  /// Plays until the player or an agent quits.
  ///
  /// # Errors
  ///
  /// Returns why the game being loaded did not open, or its loop did not start or stopped stepping.
  pub(crate) fn run(&mut self, platform: &mut Platform) -> XrfResult<()> {
    let mut events: Vec<PlatformEvent> = Vec::new();

    loop {
      self.apply_settings();
      self.startup.show_when_covered(
        self.context.window,
        self.context.view,
        GameCursor::of_seized(self.layers.is_seized()),
      );
      events.clear();
      platform.wait(self.get_wait(), &mut events);
      self.layers.feed(platform.get_time(), &mut events);

      for event in &events {
        self.take_event(event);
      }

      if self.context.is_quitting() {
        return Ok(());
      }

      self
        .context
        .flow
        .advance(self.context.view, &self.context.settings.get().input)?;
      self.layers.update(&mut self.context)?;
      self.layers.draw(&mut self.context);

      if self.context.is_quitting() {
        return Ok(());
      }
    }
  }

  /// How long the wait for the platform's events may last: until the first layer's next work is due, however long
  /// that is, since what another thread brings (the loop's frames and events, the load's news, an agent's request, the
  /// level standing ready) ends the wait itself; and no longer than the hidden window's look while it is hidden.
  fn get_wait(&self) -> Duration {
    let due: Duration = self
      .layers
      .get_due()
      .map_or(Duration::MAX, |due| due.saturating_duration_since(Instant::now()));

    if self.startup.is_shown() {
      due
    } else {
      due.min(Self::HIDDEN_WAIT)
    }
  }

  /// Applies what changed in the settings since the last frame, by group: the view draws with its groups' from this
  /// frame, the running game binds its keys again with the input group's; the window's wait for the next start.
  fn apply_settings(&mut self) {
    let changes: GameSettingsChanges = self.context.settings.take_changes();

    if changes.has_any(&[
      GameSettingsGroup::Render,
      GameSettingsGroup::View,
      GameSettingsGroup::Upscaling,
    ]) {
      self.context.view.apply_settings(self.context.settings.get());
    }

    if changes.has(GameSettingsGroup::Input)
      && let Some(running) = self.context.flow.get_running_mut()
    {
      running.rebind(&self.context.settings.get().input);
    }
  }

  /// Takes one platform event: the window's own first (its close ends play, its resize fits the view), then down the
  /// chain, the cursor handed over as a layer seizes the keyboard and the mouse or gives them back; the quit key ends
  /// play where no layer kept it.
  fn take_event(&mut self, event: &PlatformEvent) {
    match &event.kind {
      PlatformEventKind::Quit => return self.context.quit(),
      PlatformEventKind::Resized { .. } => return self.context.view.fit(self.context.window),
      _ => {}
    }

    let was_seized: bool = self.layers.is_seized();
    let is_kept: bool = self.layers.take_event(event, &mut self.context);

    let is_seized: bool = self.layers.is_seized();

    if is_seized != was_seized {
      GameCursor::of_seized(is_seized).hand(self.context.window);
    }

    if !is_kept
      && matches!(
        event.kind,
        PlatformEventKind::Key {
          scancode: Self::QUIT_KEY,
          is_down: true,
          ..
        }
      )
    {
      self.context.quit();
    }
  }
}

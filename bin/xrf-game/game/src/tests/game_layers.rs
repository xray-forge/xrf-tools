use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};
use xrf_game_platform::{PlatformEvent, PlatformEventKind, PlatformScancode};

use crate::layer::game_input_fate::GameInputFate;
use crate::layer::game_layer::GameLayer;
use crate::layer::game_layers::GameLayers;

/// What the layers of a test did, in the order they did it.
type Told = Vec<String>;

/// What a telling layer does with a key.
struct KeyFate {
  key: PlatformScancode,
  fate: GameInputFate,
}

/// A layer that tells what it is asked and answers each key as its table says, passing every other.
struct TellingLayer {
  name: &'static str,
  fates: Vec<KeyFate>,
  /// A key it plays itself, once.
  fed: Option<PlatformScancode>,
  due: Option<Instant>,
  is_failing: bool,
}

impl TellingLayer {
  fn new(name: &'static str) -> Self {
    Self {
      name,
      fates: Vec::new(),
      fed: None,
      due: None,
      is_failing: false,
    }
  }

  fn with_fate(mut self, key: u16, fate: GameInputFate) -> Self {
    self.fates.push(KeyFate {
      key: PlatformScancode(key),
      fate,
    });
    self
  }
}

impl GameLayer<Told> for TellingLayer {
  fn feed(&mut self, at: Duration, events: &mut Vec<PlatformEvent>) {
    if let Some(scancode) = self.fed.take() {
      events.push(PlatformEvent {
        at,
        kind: PlatformEventKind::Key {
          scancode,
          is_down: true,
          is_repeat: false,
        },
      });
    }
  }

  fn take_event(&mut self, event: &PlatformEvent, told: &mut Told) -> GameInputFate {
    let PlatformEventKind::Key { scancode, .. } = &event.kind else {
      return GameInputFate::Passed;
    };

    told.push(format!("{} offered {}", self.name, scancode.0));

    self
      .fates
      .iter()
      .find(|known| known.key == *scancode)
      .map_or(GameInputFate::Passed, |known| known.fate)
  }

  fn release_input(&mut self, _: Duration, told: &mut Told) {
    told.push(format!("{} released", self.name));
  }

  fn update(&mut self, told: &mut Told) -> XrfResult<()> {
    told.push(format!("{} updated", self.name));

    if self.is_failing {
      return Err(XrfError::new_unexpected_error("the layer cannot go on"));
    }

    Ok(())
  }

  fn draw(&mut self, told: &mut Told) {
    told.push(format!("{} drew", self.name));
  }

  fn get_due(&self) -> Option<Instant> {
    self.due
  }
}

fn key(scancode: u16) -> PlatformEvent {
  PlatformEvent {
    at: Duration::from_millis(5),
    kind: PlatformEventKind::Key {
      scancode: PlatformScancode(scancode),
      is_down: true,
      is_repeat: false,
    },
  }
}

const OPEN: u16 = 53;
const CLOSE: u16 = 41;
const FORWARD: u16 = 26;

/// A console over a HUD over the player: the console opens and closes, the player moves.
fn layers() -> GameLayers<Told> {
  let mut layers: GameLayers<Told> = GameLayers::default();

  layers.push(
    TellingLayer::new("console")
      .with_fate(OPEN, GameInputFate::Seized)
      .with_fate(CLOSE, GameInputFate::Yielded),
  );
  layers.push(TellingLayer::new("hud"));
  layers.push(TellingLayer::new("player").with_fate(FORWARD, GameInputFate::Kept));
  layers
}

#[test]
fn offers_an_event_down_the_chain_until_a_layer_keeps_it() {
  let mut layers: GameLayers<Told> = layers();
  let mut told: Told = Vec::new();

  assert!(layers.take_event(&key(FORWARD), &mut told));
  assert_eq!(told, ["console offered 26", "hud offered 26", "player offered 26"]);

  told.clear();

  // A key no layer reads comes out of the chain's end, for play to take as the window's own.
  assert!(!layers.take_event(&key(4), &mut told));
  assert_eq!(told, ["console offered 4", "hud offered 4", "player offered 4"]);
}

#[test]
fn a_layer_seizing_input_makes_those_under_it_let_go_until_it_yields() {
  let mut layers: GameLayers<Told> = layers();
  let mut told: Told = Vec::new();

  assert!(!layers.is_seized());
  assert!(layers.take_event(&key(OPEN), &mut told));
  assert!(layers.is_seized());
  // The layers under the console let go, and were never offered the key it kept.
  assert_eq!(told, ["console offered 53", "hud released", "player released"]);

  told.clear();

  assert!(layers.take_event(&key(CLOSE), &mut told));
  assert!(!layers.is_seized());
  assert_eq!(told, ["console offered 41"]);
}

#[test]
fn a_layer_seizing_twice_holds_once_and_one_yield_frees_the_cursor() {
  let mut layers: GameLayers<Told> = layers();
  let mut told: Told = Vec::new();

  assert!(layers.take_event(&key(OPEN), &mut told));
  told.clear();
  assert!(layers.take_event(&key(OPEN), &mut told));
  assert!(layers.is_seized());
  // It held them already: the layers under it are not told to let go again.
  assert_eq!(told, ["console offered 53"]);

  assert!(layers.take_event(&key(CLOSE), &mut told));
  assert!(!layers.is_seized());
}

#[test]
fn a_yield_from_a_layer_holding_nothing_changes_nothing() {
  let mut layers: GameLayers<Told> = layers();
  let mut told: Told = Vec::new();

  assert!(layers.take_event(&key(CLOSE), &mut told));
  assert!(!layers.is_seized());

  // The seize after it counts whole.
  assert!(layers.take_event(&key(OPEN), &mut told));
  assert!(layers.is_seized());
}

#[test]
fn stays_seized_while_one_of_two_holding_layers_still_holds() {
  const MENU_OPEN: u16 = 58;
  const MENU_CLOSE: u16 = 59;

  let mut layers: GameLayers<Told> = GameLayers::default();
  let mut told: Told = Vec::new();

  layers.push(
    TellingLayer::new("console")
      .with_fate(OPEN, GameInputFate::Seized)
      .with_fate(CLOSE, GameInputFate::Yielded),
  );
  layers.push(
    TellingLayer::new("menu")
      .with_fate(MENU_OPEN, GameInputFate::Seized)
      .with_fate(MENU_CLOSE, GameInputFate::Yielded),
  );
  layers.push(TellingLayer::new("player"));

  assert!(layers.take_event(&key(MENU_OPEN), &mut told));
  assert!(layers.take_event(&key(OPEN), &mut told));
  assert!(layers.take_event(&key(CLOSE), &mut told));
  // The console gave them back; the menu still has them.
  assert!(layers.is_seized());

  assert!(layers.take_event(&key(MENU_CLOSE), &mut told));
  assert!(!layers.is_seized());
}

#[test]
fn updates_from_the_top_and_draws_from_the_bottom() {
  let mut layers: GameLayers<Told> = layers();
  let mut told: Told = Vec::new();

  layers.update(&mut told).expect("every layer goes on");
  layers.draw(&mut told);

  assert_eq!(
    told,
    [
      "console updated",
      "hud updated",
      "player updated",
      "player drew",
      "hud drew",
      "console drew"
    ]
  );
}

#[test]
fn a_layer_that_cannot_go_on_ends_the_update_there() {
  let mut layers: GameLayers<Told> = GameLayers::default();
  let mut told: Told = Vec::new();

  layers.push(TellingLayer::new("console"));
  layers.push(TellingLayer {
    is_failing: true,
    ..TellingLayer::new("hud")
  });
  layers.push(TellingLayer::new("player"));

  assert!(layers.update(&mut told).is_err());
  assert_eq!(told, ["console updated", "hud updated"]);
}

#[test]
fn feeds_what_a_layer_plays_to_every_layer_as_the_windows_own() {
  let mut layers: GameLayers<Told> = GameLayers::default();
  let mut told: Told = Vec::new();
  let mut events: Vec<PlatformEvent> = vec![key(4)];

  layers.push(TellingLayer {
    fed: Some(PlatformScancode(FORWARD)),
    ..TellingLayer::new("agent")
  });
  layers.push(TellingLayer::new("player").with_fate(FORWARD, GameInputFate::Kept));
  layers.feed(Duration::from_millis(9), &mut events);

  // After the window's own, stamped with the time given.
  assert_eq!(events.len(), 2);
  assert_eq!(events[1].at, Duration::from_millis(9));
  assert!(layers.take_event(&events[1], &mut told));
  assert_eq!(told, ["agent offered 26", "player offered 26"]);

  // Played once.
  layers.feed(Duration::from_millis(10), &mut events);
  assert_eq!(events.len(), 2);
}

#[test]
fn is_due_when_the_first_of_its_layers_is() {
  let now: Instant = Instant::now();
  let mut layers: GameLayers<Told> = GameLayers::default();

  layers.push(TellingLayer::new("console"));

  assert_eq!(layers.get_due(), None);

  layers.push(TellingLayer {
    due: Some(now + Duration::from_millis(33)),
    ..TellingLayer::new("hud")
  });
  layers.push(TellingLayer {
    due: Some(now + Duration::from_millis(8)),
    ..TellingLayer::new("player")
  });

  assert_eq!(layers.get_due(), Some(now + Duration::from_millis(8)));
}

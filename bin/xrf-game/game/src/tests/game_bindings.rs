use std::time::Duration;

use xrf_game_movement::{MovementAction, MovementCommand, MovementInput, MovementInputKind};
use xrf_game_platform::{PlatformEvent, PlatformEventKind, PlatformScancode};

use xrf_game_config::ConfigIssues;
use xrf_game_settings::{GameInputSettings, GameSettings};

use crate::input::game_action::GameAction;
use crate::input::game_bindings::GameBindings;
use crate::input::game_controls::GameControls;
use crate::input::game_key_names::GameKeyNames;

const AT: Duration = Duration::from_millis(42);

fn event(kind: PlatformEventKind) -> PlatformEvent {
  PlatformEvent { at: AT, kind }
}

fn key(scancode: u16, is_down: bool, is_repeat: bool) -> PlatformEvent {
  event(PlatformEventKind::Key {
    scancode: PlatformScancode(scancode),
    is_down,
    is_repeat,
  })
}

fn movement(kind: MovementInputKind) -> Option<MovementCommand> {
  Some(MovementCommand::Input(MovementInput { at: AT, kind }))
}

#[test]
fn reads_a_bound_key_by_its_place_and_skips_its_repeats() {
  let bindings: GameBindings = GameBindings::default();

  assert_eq!(
    bindings.read(&key(26, true, false)),
    movement(MovementInputKind::Action {
      action: MovementAction::Forward,
      is_pressed: true,
    })
  );
  assert_eq!(
    bindings.read(&key(225, false, false)),
    movement(MovementInputKind::Action {
      action: MovementAction::Accel,
      is_pressed: false,
    })
  );
  assert_eq!(bindings.read(&key(26, true, true)), None);
  // F12 is bound to nothing.
  assert_eq!(bindings.read(&key(69, true, false)), None);
}

#[test]
fn reads_the_noclip_key_as_a_mode_toggle_on_its_press_alone() {
  let bindings: GameBindings = GameBindings::default();

  assert_eq!(bindings.read(&key(17, true, false)), Some(MovementCommand::ToggleMode));
  assert_eq!(bindings.read(&key(17, true, true)), None);
  assert_eq!(bindings.read(&key(17, false, false)), None);
}

#[test]
fn reads_the_trees_controls_and_lays_the_players_bindings_over_them() {
  let mut issues: ConfigIssues = ConfigIssues::default();
  let controls: GameControls = GameControls::read(
    "unbindall\nbind forward kUP ; the arrows\nbind_sec forward kW\nbind back kDOWN\nbind wpn_fire mouse1\n\
     bind jump mouse2\n",
    &mut issues,
  );
  let mut input: GameInputSettings = GameInputSettings::default();

  input.bindings.back = Some(String::from("S"));
  input.bindings_secondary.jump = Some(String::from("kSPACE"));
  input.bindings.crouch = Some(String::from("Nowhere"));

  let bindings: GameBindings = GameBindings::new(&controls, &input);
  let pressed = |action: MovementAction| {
    movement(MovementInputKind::Action {
      action,
      is_pressed: true,
    })
  };

  // The tree's main and second keys; its back moved by the player; jump's second the player's, by X-Ray's name.
  assert_eq!(bindings.read(&key(82, true, false)), pressed(MovementAction::Forward));
  assert_eq!(bindings.read(&key(26, true, false)), pressed(MovementAction::Forward));
  assert_eq!(bindings.read(&key(22, true, false)), pressed(MovementAction::Back));
  assert_eq!(bindings.read(&key(81, true, false)), None);
  assert_eq!(bindings.read(&key(44, true, false)), pressed(MovementAction::Up));
  // Actions the tree leaves unbound stay so, an unknown name keeps the tree's; noclip keeps the game's N.
  assert_eq!(bindings.read(&key(224, true, false)), None);
  assert_eq!(bindings.read(&key(17, true, false)), Some(MovementCommand::ToggleMode));
  // A mouse button is no key the game binds yet: an issue, the action unbound.
  assert_eq!(
    issues.iter().map(ToString::to_string).collect::<Vec<_>>(),
    vec!["default_controls.ltx bind: binds jump to 'mouse2', which is no key the game reads"]
  );
}

#[test]
fn names_every_action_as_a_main_and_a_second_binding_of_the_settings() {
  let paths: Vec<String> = GameSettings::default()
    .get_entries()
    .into_iter()
    .map(|entry| entry.path)
    .collect();

  // The settings' fields and the actions are written apart; each action's name must be a setting of both groups.
  for action in GameAction::ALL {
    for group in ["input.bindings", "input.bindings_secondary"] {
      let path: String = format!("{group}.{}", action.get_name());

      assert!(paths.contains(&path), "the settings hold no {path}");
    }
  }

  assert_eq!(
    paths.iter().filter(|path| path.starts_with("input.bindings.")).count(),
    GameAction::ALL.len()
  );
}

#[test]
fn reads_x_rays_key_names_as_sdls() {
  assert_eq!(GameKeyNames::to_sdl("kW").as_deref(), Some("W"));
  assert_eq!(GameKeyNames::to_sdl("kLSHIFT").as_deref(), Some("Left Shift"));
  assert_eq!(GameKeyNames::to_sdl("kNUMPAD7").as_deref(), Some("Keypad 7"));
  assert_eq!(GameKeyNames::to_sdl("kF11").as_deref(), Some("F11"));
  assert_eq!(GameKeyNames::to_sdl("kGRAVE").as_deref(), Some("Grave"));
  assert_eq!(GameKeyNames::to_sdl("mouse1"), None);
  assert_eq!(GameKeyNames::to_sdl("kSOMETHING"), None);
}

#[test]
fn leaves_the_mouse_to_the_look_and_reads_losing_focus_as_letting_go() {
  let bindings: GameBindings = GameBindings::default();

  // The mouse turns presentation's look, never a command.
  assert_eq!(
    bindings.read(&event(PlatformEventKind::MouseMoved {
      x: 10.0,
      y: 20.0,
      dx: 3.0,
      dy: -2.0,
    })),
    None
  );
  assert_eq!(
    bindings.read(&event(PlatformEventKind::Focused { is_focused: false })),
    movement(MovementInputKind::ReleaseAll)
  );
  assert_eq!(
    bindings.read(&event(PlatformEventKind::Focused { is_focused: true })),
    None
  );
}

use std::time::{Duration, Instant};

use xrf_game_mcp::GameInput;
use xrf_game_platform::PlatformEventKind;

use crate::control::game_input_steps::GameInputSteps;
use crate::control::scheduled_input::ScheduledInput;

#[test]
fn waits_space_the_steps_and_a_last_wait_moves_the_end() {
  let start: Instant = Instant::now();
  let mut due: Instant = start;
  let scheduled: Vec<ScheduledInput> = GameInputSteps::read(
    vec![
      GameInput::Text {
        text: String::from("a"),
      },
      GameInput::Wait { ms: 100 },
      GameInput::Click {
        x: 1.0,
        y: 2.0,
        button: None,
      },
      GameInput::Wait { ms: 50 },
    ],
    &mut due,
  )
  .expect("the steps read");
  let offsets: Vec<Duration> = scheduled.iter().map(|input| input.due - start).collect();

  assert_eq!(
    offsets,
    [Duration::ZERO, Duration::from_millis(100), Duration::from_millis(100)]
  );
  assert!(matches!(
    scheduled[1].kind,
    PlatformEventKind::MouseButton { is_down: true, .. }
  ));
  assert_eq!(due - start, Duration::from_millis(150));
}

#[test]
fn refuses_waits_past_the_longest_and_unknown_buttons() {
  let mut due: Instant = Instant::now();
  let longest: u64 = u64::try_from(GameInputSteps::LONGEST.as_millis()).expect("it fits");

  assert!(GameInputSteps::read(vec![GameInput::Wait { ms: longest + 1 }], &mut due).is_err());
  assert!(
    GameInputSteps::read(
      vec![GameInput::Click {
        x: 0.0,
        y: 0.0,
        button: Some(String::from("thumb")),
      }],
      &mut due,
    )
    .is_err()
  );
}

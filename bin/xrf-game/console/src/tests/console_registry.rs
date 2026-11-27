use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::console_answer::ConsoleAnswer;
use crate::console_choice::ConsoleChoice;
use crate::console_command::ConsoleCommand;
use crate::console_completion::ConsoleCompletion;
use crate::console_context::ConsoleContext;
use crate::console_family::ConsoleFamily;
use crate::console_line::ConsoleLine;
use crate::console_registry::ConsoleRegistry;
use crate::console_variable::ConsoleVariable;

/// A context of two number variables and a count of how often `bump` ran.
#[derive(Default)]
struct Counter {
  resolution: u32,
  steps: u32,
  bumped: u32,
}

impl ConsoleContext for Counter {
  fn list_variables(&self) -> Vec<ConsoleVariable> {
    vec![
      ConsoleVariable {
        name: String::from("view.shadows.resolution"),
        value: self.resolution.to_string(),
      },
      ConsoleVariable {
        name: String::from("view.shadows.steps"),
        value: self.steps.to_string(),
      },
    ]
  }

  fn set_variable(&mut self, name: &str, value: &str) -> ConsoleAnswer {
    let Ok(number) = value.parse::<u32>() else {
      return ConsoleAnswer::fail(format!("'{value}' is no number"));
    };

    match name {
      "view.shadows.resolution" => self.resolution = number,
      _ => self.steps = number,
    }

    ConsoleAnswer::succeed(vec![format!("{name} = {number}")])
  }
}

/// The counter owns its variables, so it is its own family.
impl ConsoleFamily for Counter {
  type Context<'a> = Self;
}

fn create_registry() -> ConsoleRegistry<Counter> {
  ConsoleRegistry::default().with(ConsoleCommand::new(
    "bump",
    "[times]",
    "counts up",
    |counter: &mut Counter, argument: Option<&str>| {
      counter.bumped += argument.and_then(|times| times.parse().ok()).unwrap_or(1);

      ConsoleAnswer::succeed(vec![format!("bumped {}", counter.bumped)])
    },
  ))
}

#[test]
fn registers_a_command_that_carries_its_own_state_and_overrides_a_builtin() {
  let mut registry: ConsoleRegistry<Counter> = create_registry();
  let mut counter: Counter = Counter::default();
  // What a script's command would hold: something of its own, here how often it ran.
  let calls: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
  let counted: Arc<AtomicU32> = Arc::clone(&calls);

  registry.register(ConsoleCommand::new(
    "ping",
    "",
    "answers pong",
    move |_: &mut Counter, _| {
      counted.fetch_add(1, Ordering::Relaxed);

      ConsoleAnswer::succeed(vec![String::from("pong")])
    },
  ));

  assert_eq!(registry.execute(&mut counter, "ping").lines, vec!["pong"]);
  assert_eq!(calls.load(Ordering::Relaxed), 1);

  let replaced = registry.register(ConsoleCommand::new("help", "", "a mod's help", |_: &mut Counter, _| {
    ConsoleAnswer::succeed(vec![String::from("the mod's own help")])
  }));

  assert!(replaced.is_none(), "help is no registered command, so none is replaced");
  assert_eq!(registry.execute(&mut counter, "help").lines, vec!["the mod's own help"]);
  assert_eq!(registry.complete(&counter, "he").choices[0].hint, "a mod's help");
}

#[test]
fn parses_a_name_and_the_rest_of_the_line() {
  assert_eq!(ConsoleLine::parse("   "), None);
  assert_eq!(
    ConsoleLine::parse(" pose  1 2 3 "),
    Some(ConsoleLine {
      name: "pose",
      argument: Some("1 2 3"),
    })
  );
  assert_eq!(
    ConsoleLine::parse("quit"),
    Some(ConsoleLine {
      name: "quit",
      argument: None,
    })
  );
}

#[test]
fn runs_a_command_with_what_follows_its_name() {
  let registry: ConsoleRegistry<Counter> = create_registry();
  let mut counter: Counter = Counter::default();

  assert_eq!(
    registry.execute(&mut counter, "bump 3"),
    ConsoleAnswer::succeed(vec![String::from("bumped 3")])
  );
  assert_eq!(counter.bumped, 3);
}

#[test]
fn shows_a_variable_alone_and_sets_it_with_a_value() {
  let registry: ConsoleRegistry<Counter> = create_registry();
  let mut counter: Counter = Counter::default();

  assert_eq!(
    registry.execute(&mut counter, "view.shadows.resolution 2048").lines,
    vec!["view.shadows.resolution = 2048"]
  );
  assert_eq!(
    registry.execute(&mut counter, "view.shadows.resolution").lines,
    vec!["view.shadows.resolution = 2048"]
  );
  assert!(registry.execute(&mut counter, "view.shadows.resolution high").is_failed);
}

#[test]
fn completes_a_name_as_far_as_it_is_shared_then_a_group_at_a_time() {
  let registry: ConsoleRegistry<Counter> = create_registry();
  let counter: Counter = Counter::default();
  let complete = |line: &str| registry.complete(&counter, line);
  let choice = |name: &str, hint: &str| ConsoleChoice {
    name: String::from(name),
    hint: String::from(hint),
  };

  assert_eq!(
    complete("bu"),
    ConsoleCompletion {
      line: String::from("bump "),
      choices: vec![choice("bump", "[times] - counts up")],
    }
  );
  assert_eq!(
    complete("v"),
    ConsoleCompletion {
      line: String::from("view.shadows."),
      choices: vec![
        choice("view.shadows.resolution", "= 0"),
        choice("view.shadows.steps", "= 0")
      ],
    }
  );
  assert_eq!(complete("view.shadows.r").line, "view.shadows.resolution ");
  assert_eq!(complete("l").choices, vec![choice("list", "[part] - the variables")]);
  assert!(complete("").choices.is_empty());
  assert_eq!(
    complete("zzz"),
    ConsoleCompletion {
      line: String::from("zzz"),
      choices: Vec::new(),
    }
  );
  assert_eq!(complete("bump 3").line, "bump 3");
}

#[test]
fn lists_variables_and_commands_and_suggests_for_an_unknown_name() {
  let registry: ConsoleRegistry<Counter> = create_registry();
  let mut counter: Counter = Counter::default();

  assert_eq!(
    registry.execute(&mut counter, "list steps").lines,
    vec!["view.shadows.steps = 0"]
  );
  assert!(
    registry
      .execute(&mut counter, "help")
      .lines
      .contains(&String::from("bump [times] - counts up"))
  );

  let unknown: ConsoleAnswer = registry.execute(&mut counter, "shadows");

  assert!(unknown.is_failed);
  assert_eq!(
    unknown.lines,
    vec!["Unknown command or variable 'shadows'; did you mean view.shadows.resolution, view.shadows.steps?"]
  );
  assert_eq!(registry.execute(&mut counter, ""), ConsoleAnswer::default());
}

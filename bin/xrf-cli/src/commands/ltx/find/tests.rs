use std::path::PathBuf;

use xrf_utils::format_path;

use crate::commands::ltx::find::command::FindCommand;
use crate::core::command::command_testing::run_command_for_result;
use crate::core::terminal::ShellArgument;

// The path names nothing, so an accepted query fails on opening the project instead, with a different error.
fn is_refused(arguments: &[&str]) -> bool {
  let arguments: Vec<String> = ["find", "--silent", "absent-configs-root"]
    .iter()
    .chain(arguments)
    .map(|it| String::from(*it))
    .collect();

  match run_command_for_result(&FindCommand, &arguments) {
    Err(error) => error.to_string().contains("pass the"),
    Ok(_) => false,
  }
}

#[test]
fn refuses_a_section_written_as_a_header() {
  for query in ["[wpn_ak74]", "![wpn_ak74]", "@[wpn_ak74]", "!![wpn_ak74]"] {
    assert!(
      is_refused(&["--section", query]),
      "{query} can never match a recorded name"
    );
  }

  assert!(!is_refused(&["--section", "wpn_ak74"]));
}

#[test]
fn refuses_a_field_written_with_an_operation_prefix() {
  for query in ["!rpm", ">ammo_class", "<ammo_class"] {
    assert!(
      is_refused(&["--field", query]),
      "{query} can never match a recorded name"
    );
  }

  assert!(!is_refused(&["--field", "rpm"]));
}

#[test]
fn quotes_a_hinted_path_holding_a_space() {
  let path: PathBuf = PathBuf::from("game data").join("configs");
  let hint: String = FindCommand::resolution_hint(&path, "wpn_ak74", true);
  let expected: String = ShellArgument::quote(&format_path(&path).to_string()).into_owned();

  assert_ne!(expected, format_path(&path).to_string(), "a space is quoted");
  assert_eq!(
    hint,
    format!("Resolved values: ltx inspect {expected} --dltx --section wpn_ak74")
  );
}

#[test]
fn leaves_a_plain_hinted_path_bare() {
  assert_eq!(
    FindCommand::resolution_hint(&PathBuf::from("configs"), "wpn_ak74", false),
    "Resolved values: ltx inspect configs --section wpn_ak74"
  );
}

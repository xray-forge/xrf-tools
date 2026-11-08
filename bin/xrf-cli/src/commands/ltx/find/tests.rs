use crate::commands::ltx::find::command::FindCommand;
use crate::core::command::command_testing::run_command_for_result;

/// Whether `ltx find` refuses `arguments` as a query that can never match.
///
/// The path names nothing, so an accepted query fails on opening the project instead, with a different error.
fn is_refused(arguments: &[&str]) -> bool {
  let arguments: Vec<String> = ["find", "--silent", "--path", "absent-configs-root"]
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

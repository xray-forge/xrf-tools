use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_dialog::{DialogProject, DialogProjectLayout, DialogTextLanguage};
use xrf_error::XrfResult;

use crate::commands::dialog::dialog_roots::{DialogRootsArguments, requested_dialog_prefix, requested_dialog_roots};

const LANGUAGE_ARGUMENT: &str = "language";

/// Declares what opening a dialog project takes: its roots, and the language its lines are shown in.
pub trait DialogProjectArguments {
  /// Declares the dialog roots and `--language`.
  #[must_use]
  fn with_dialog_project(self) -> Self;
}

impl DialogProjectArguments for Command {
  fn with_dialog_project(self) -> Self {
    self.with_dialog_roots().arg(
      Arg::new(LANGUAGE_ARGUMENT)
        .help("Language to show phrase lines in, such as rus or eng. Defaults to the first the tree holds")
        .long(LANGUAGE_ARGUMENT)
        .value_parser(value_parser!(String)),
    )
  }
}

/// Open the dialog project a command's roots and prefix name, laid out as shipped gamedata.
///
/// # Errors
///
/// Returns an error when the roots cannot be mounted or hold no dialog files under the prefix.
pub fn open_dialog_project(matches: &ArgMatches) -> XrfResult<DialogProject> {
  let layout: DialogProjectLayout = DialogProjectLayout {
    dialogs_prefix: requested_dialog_prefix(matches).map(str::to_owned),
    ..DialogProjectLayout::default()
  };

  DialogProject::open(&requested_dialog_roots(matches)?, &layout)
}

/// The language a command was asked to show lines in, if it was asked for one.
pub fn requested_dialog_language(matches: &ArgMatches) -> Option<&str> {
  matches.get_one::<String>(LANGUAGE_ARGUMENT).map(String::as_str)
}

/// The text a command shows lines from: the requested language, or else the project's first.
///
/// `None` when the tree holds no text in that language, in which case phrases show their keys alone.
pub fn requested_dialog_text<'a>(
  matches: &'a ArgMatches,
  project: &'a DialogProject,
) -> Option<DialogTextLanguage<'a>> {
  requested_dialog_language(matches)
    .or_else(|| project.get_text().get_default_language())
    .and_then(|language| project.get_text().in_language(language))
}

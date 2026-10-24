use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_dialog::{DialogProject, DialogProjectLayout, DialogTextLanguage};
use xrf_error::XrfResult;

use crate::core::arguments::XrayRootsArguments;

/// What opening a dialog project takes, and the opening itself: its roots, and the language its lines are shown in.
pub struct DialogProjectOpening;

impl DialogProjectOpening {
  const LANGUAGE: &'static str = "language";

  /// Declares the roots every dialog command reads on `command`.
  pub fn declare_roots(command: Command) -> Command {
    XrayRootsArguments::declare(
      command,
      "Root holding dialog xml. Repeat to layer roots, highest priority first",
      "Limit to one logical subtree, such as configs\\gameplay",
    )
  }

  /// Declares the roots and `--language` on `command`, for a command showing phrase lines.
  pub fn declare(command: Command) -> Command {
    Self::declare_roots(command).arg(
      Arg::new(Self::LANGUAGE)
        .help("Language to show phrase lines in, such as rus or eng. Defaults to the first the tree holds")
        .long(Self::LANGUAGE)
        .value_parser(value_parser!(String)),
    )
  }

  /// The dialog project the command's roots and prefix name, laid out as shipped gamedata.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted or hold no dialog files under the prefix.
  pub fn open(matches: &ArgMatches) -> XrfResult<DialogProject> {
    DialogProject::open(&XrayRootsArguments::of(matches)?, &Self::to_layout(matches))
  }

  /// The project as [`Self::open`] opens it, without its text tree, for a command reporting keys alone.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted or hold no dialog files under the prefix.
  pub fn open_keys(matches: &ArgMatches) -> XrfResult<DialogProject> {
    let layout: DialogProjectLayout = DialogProjectLayout {
      is_text_skipped: true,
      ..Self::to_layout(matches)
    };

    DialogProject::open(&XrayRootsArguments::of(matches)?, &layout)
  }

  /// The language the command was asked to show lines in, if it was asked for one.
  pub fn language_of(matches: &ArgMatches) -> Option<&str> {
    matches.get_one::<String>(Self::LANGUAGE).map(String::as_str)
  }

  /// The text the command shows lines from: the requested language, or else the project's first; `None` when the tree
  /// holds no text in that language, in which case phrases show their keys alone.
  pub fn text_of<'a>(matches: &'a ArgMatches, project: &'a DialogProject) -> Option<DialogTextLanguage<'a>> {
    Self::language_of(matches)
      .or_else(|| project.get_text().get_default_language())
      .and_then(|language| project.get_text().in_language(language))
  }

  /// The layout the command's prefix names, as shipped gamedata.
  fn to_layout(matches: &ArgMatches) -> DialogProjectLayout {
    DialogProjectLayout {
      dialogs_prefix: XrayRootsArguments::prefix_of(matches).map(str::to_owned),
      ..DialogProjectLayout::default()
    }
  }
}

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_dialog::{DialogListedFiles, DialogProject, DialogProjectLayout, DialogTextLanguage};
use xrf_error::XrfResult;
use xrf_ltx_project::{LtxProject, LtxProjectOptions};
use xrf_vfs::{XrayRoots, XraySearch};

use crate::core::arguments::XrayRootsArguments;
use crate::core::command::CommandInput;

/// What opening a dialog project takes, and the opening itself: its roots, and the language its lines are shown in.
pub(crate) struct DialogProjectOpening;

impl DialogProjectOpening {
  const LANGUAGE: &'static str = "language";
  /// Where the game keeps its configs, `system.ltx` among them.
  const CONFIGS_DIRECTORY: &'static str = "configs";

  /// The roots every dialog command reads, as its input.
  pub(crate) fn input() -> CommandInput {
    CommandInput::roots(
      "Root holding dialog xml. Give several to layer them, highest priority first",
      "Limit to one logical subtree, such as configs\\gameplay",
    )
  }

  /// Declares `--language` on `command`, for a command showing phrase lines.
  pub(crate) fn declare_language(command: Command) -> Command {
    command.arg(
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
  pub(crate) fn open(matches: &ArgMatches) -> XrfResult<DialogProject> {
    DialogProject::open(&XrayRootsArguments::of(matches)?, &Self::parse_layout(matches))
  }

  /// The project as the engine loads it: the dialog and partner files `configs\system.ltx`, with its includes, lists,
  /// and every file named as one where the tree holds no `system.ltx` or it lists none.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted, `system.ltx` does not resolve, or a tree it lists nothing for
  /// holds no dialog files under the prefix.
  pub(crate) fn open_listed(matches: &ArgMatches) -> XrfResult<DialogProject> {
    let roots: XrayRoots = XrayRootsArguments::of(matches)?;
    let layout: DialogProjectLayout = Self::parse_layout(matches);
    let search: XraySearch = roots.to_search_plan()?.open()?;

    match Self::read_listed(&roots, &search)? {
      Some(listed) => Ok(DialogProject::from_listed(search, &roots, &layout, &listed)),
      None => DialogProject::from_search(search, &roots, &layout),
    }
  }

  /// The lists `configs\system.ltx` holds, or `None` for a tree without one, such as a folder of dialog files alone.
  fn read_listed(roots: &XrayRoots, search: &XraySearch) -> XrfResult<Option<DialogListedFiles>> {
    let configs: LtxProject = LtxProject::open_lean_at_search_opt(
      roots.describe(),
      search,
      Some(Self::CONFIGS_DIRECTORY),
      LtxProjectOptions::default(),
    )?;

    if search.find(configs.system_ltx_path()?.as_str())?.is_none() {
      return Ok(None);
    }

    Ok(DialogListedFiles::from_system_ltx(&configs.system_ltx()?.ltx))
  }

  /// The project as [`Self::open`] opens it, without its text tree, for a command reporting keys alone.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted or hold no dialog files under the prefix.
  pub(crate) fn open_keys(matches: &ArgMatches) -> XrfResult<DialogProject> {
    let layout: DialogProjectLayout = DialogProjectLayout {
      is_text_skipped: true,
      ..Self::parse_layout(matches)
    };

    DialogProject::open(&XrayRootsArguments::of(matches)?, &layout)
  }

  /// The language the command was asked to show lines in, if it was asked for one.
  pub(crate) fn language_of(matches: &ArgMatches) -> Option<&str> {
    matches.get_one::<String>(Self::LANGUAGE).map(String::as_str)
  }

  /// The text the command shows lines from: the requested language, or else the project's first; `None` when the tree
  /// holds no text in that language, in which case phrases show their keys alone.
  pub(crate) fn text_of<'a>(matches: &'a ArgMatches, project: &'a DialogProject) -> Option<DialogTextLanguage<'a>> {
    Self::language_of(matches)
      .or_else(|| project.get_text().get_default_language())
      .and_then(|language| project.get_text().in_language(language))
  }

  /// The layout the command's prefix names, as shipped gamedata.
  fn parse_layout(matches: &ArgMatches) -> DialogProjectLayout {
    DialogProjectLayout {
      dialogs_prefix: XrayRootsArguments::prefix_of(matches).map(str::to_owned),
      ..DialogProjectLayout::default()
    }
  }
}

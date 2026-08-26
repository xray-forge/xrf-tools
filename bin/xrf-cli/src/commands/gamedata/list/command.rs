use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots};

use crate::commands::gamedata::list::asset_lister::{AssetLister, AssetListing};
use crate::commands::gamedata::list::report::GamedataListReport;
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

/// Maximum entries printed per section before reporting the omitted count.
const PRINT_LIMIT: usize = 40;

#[derive(Default)]
pub struct ListCommand;

impl GenericCommand for ListCommand {
  fn operation(&self) -> &'static str {
    "list"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("List assets resolved by an installation or gamedata tree")
      .arg(
        Arg::new("path")
          .help("Path to a game installation or a gamedata tree")
          .short('p')
          .long("path")
          .required(true)
          // Both spellings layer: repeat the flag, or list several values after one of them.
          .action(ArgAction::Append)
          .num_args(1..)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("source")
          .help(
            "How to read the path: auto treats it as an installation only when it declares one, directory ignores any \
             declaration, installation requires one, containing-installation searches parent directories for one",
          )
          .long("source")
          .default_value("containing-installation")
          .value_parser(["auto", "directory", "installation", "containing-installation"]),
      )
      .arg(
        Arg::new("prefix")
          .help("Limit to one logical subtree, such as configs or textures\\wpn")
          .long("prefix")
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("ignore")
          .help("Logical prefixes the directory mounts omit, such as textures\\wip")
          .short('i')
          .long("ignore")
          .num_args(1..)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("loose")
          .help("List only loose files, ignoring archived entries")
          .long("loose")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("shadowed")
          .help("Also report entries hidden by a higher-priority mount")
          .long("shadowed")
          .action(ArgAction::SetTrue),
      )
  }

  /// Reports the assets a path resolves and their source mounts.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let paths: Vec<&PathBuf> = matches
      .get_many::<PathBuf>("path")
      .expect("Expected at least one path to be provided")
      .collect();
    let prefix: Option<&String> = matches.get_one::<_>("prefix");

    let output: OutputOptions = context.get_output().clone();

    let mode: XrayMountMode = XrayMountMode::try_from(
      matches
        .get_one::<String>("source")
        .expect("Expected source mode to default")
        .as_str(),
    )?;

    // One vocabulary for naming roots, so a loose tree can be listed in front of an installation.
    let roots: XrayRoots = XrayRoots::new(paths.iter().map(|path| XrayRoot::new(path.display().to_string(), mode)));

    let ignored: Vec<String> = matches
      .get_many::<String>("ignore")
      .map(|values| values.cloned().collect())
      .unwrap_or_default();

    let listing: AssetListing = AssetLister::new(&roots)
      .with_ignored(&ignored)
      .with_prefix(prefix.map(String::as_str))
      .with_loose_only(matches.get_flag("loose"))
      .with_shadowed(matches.get_flag("shadowed"))
      .run()?;

    let is_shadowed_included: bool = matches.get_flag("shadowed");

    xrf_output::info!(output, "Listing {}", listing.origin);

    for mount in &listing.mounts {
      xrf_output::info!(output, "  {mount}");
    }

    Self::print_entries(&output, &listing);

    if is_shadowed_included {
      Self::print_shadowed(&output, &listing);
    }

    Self::print_collisions(&output, &listing);
    Self::print_skipped(&output, &listing);

    xrf_output::success!(
      output,
      "{} asset(s) across {} mount(s) in {}",
      listing.entries.len(),
      listing.mounts.len(),
      xrf_utils::format_duration(listing.duration)
    );

    // Every entry is reported, not the printed first [`PRINT_LIMIT`]: a machine consumer has no screen
    // to spare and no chance to narrow the run with `--prefix` after the fact.
    context.set_result(|| GamedataListReport::new(&listing, is_shadowed_included))
  }
}

impl ListCommand {
  /// Prints winning entries up to [`PRINT_LIMIT`] and reports the omitted count.
  fn print_entries(output: &OutputOptions, listing: &AssetListing) {
    for location in listing.entries.iter().take(PRINT_LIMIT) {
      xrf_output::info!(
        output,
        "  {} [{}]",
        location.get_logical_path(),
        location.format_container()
      );
    }

    if listing.entries.len() > PRINT_LIMIT {
      xrf_output::info!(
        output,
        "  ... {} more not printed, narrow with --prefix",
        listing.entries.len() - PRINT_LIMIT
      );
    }
  }

  /// Warns about files a mount holds but cannot reach.
  ///
  /// Always reported rather than behind a flag: unlike a shadowed entry, which is how an override is meant to work, an
  /// unreachable file is an authoring mistake nobody asked to see because nobody knew about it.
  fn print_collisions(output: &OutputOptions, listing: &AssetListing) {
    if listing.collisions.is_empty() {
      return;
    }

    xrf_output::warning!(
      output,
      "{} file(s) cannot be reached, another file claims their path:",
      listing.collisions.len()
    );

    for collision in listing.collisions.iter().take(PRINT_LIMIT) {
      xrf_output::warning!(
        output,
        "  {} is unreachable, {} answers '{}'",
        collision.unreachable.display(),
        collision.kept.display(),
        collision.logical_path
      );
    }

    if listing.collisions.len() > PRINT_LIMIT {
      xrf_output::warning!(
        output,
        "  ... {} more not printed",
        listing.collisions.len() - PRINT_LIMIT
      );
    }
  }

  /// Warns about declared sources that could not be opened.
  ///
  /// Always reported: the count above is measured over what mounted, so an unopened source makes the listing quietly
  /// incomplete rather than wrong in a way anyone would notice.
  fn print_skipped(output: &OutputOptions, listing: &AssetListing) {
    if listing.skipped.is_empty() {
      return;
    }

    xrf_output::warning!(
      output,
      "{} declared source(s) could not be opened and are not listed:",
      listing.skipped.len()
    );

    for skipped in &listing.skipped {
      xrf_output::warning!(
        output,
        "  {} at {}: {}",
        skipped.origin,
        skipped.path.display(),
        skipped.reason
      );
    }
  }

  /// Prints shadowed entries up to [`PRINT_LIMIT`] and reports the omitted count.
  fn print_shadowed(output: &OutputOptions, listing: &AssetListing) {
    if listing.shadowed.is_empty() {
      xrf_output::info!(output, "Nothing is shadowed in this scope");

      return;
    }

    xrf_output::warning!(output, "{} entry(ies) are shadowed:", listing.shadowed.len());

    for location in listing.shadowed.iter().take(PRINT_LIMIT) {
      xrf_output::warning!(
        output,
        "  {} hidden in [{}]",
        location.get_logical_path(),
        location.format_container()
      );
    }

    if listing.shadowed.len() > PRINT_LIMIT {
      xrf_output::warning!(
        output,
        "  ... {} more not printed",
        listing.shadowed.len() - PRINT_LIMIT
      );
    }
  }
}

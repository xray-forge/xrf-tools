use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_utils::format_path;
use xrf_vfs::XrayRoots;

use crate::commands::gamedata::list::asset_lister::AssetLister;
use crate::commands::gamedata::list::asset_listing::AssetListing;
use crate::commands::gamedata::list::report::GamedataListReport;
use crate::core::arguments::XrayRootsArguments;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::terminal::{MountDescription, SourceReachability};

/// Maximum entries printed per section before reporting the omitted count: the same as a set of sources' listings.
const PRINT_LIMIT: usize = SourceReachability::PRINT_LIMIT;

#[derive(Default)]
pub struct ListCommand;

impl GenericCommand for ListCommand {
  fn operation(&self) -> &'static str {
    "list"
  }

  fn declare(&self, command: Command) -> Command {
    XrayRootsArguments::declare(
      command.about("List assets resolved by an installation or gamedata tree"),
      "Path to a game installation or a gamedata tree. Repeat to layer roots, highest priority first",
      "Limit to one logical subtree, such as configs or textures\\wpn",
    )
    .arg(
      Arg::new("ignore")
        .help("Logical prefixes the directory mounts omit, such as textures\\wip. Repeat to omit several")
        .short('i')
        .long("ignore")
        .action(ArgAction::Append)
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
        .help("Also report entries hidden by a higher-priority source, mount or volume")
        .long("shadowed")
        .action(ArgAction::SetTrue),
    )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();
    let roots: XrayRoots = XrayRootsArguments::of(matches)?;
    let ignored: Vec<String> = matches
      .get_many::<String>("ignore")
      .map(|values| values.cloned().collect())
      .unwrap_or_default();

    let listing: AssetListing = AssetLister::new(&roots)
      .with_ignored(&ignored)
      .with_prefix(XrayRootsArguments::prefix_of(matches))
      .with_loose_only(matches.get_flag("loose"))
      .with_shadowed(matches.get_flag("shadowed"))
      .run()?;

    let is_shadowed_included: bool = matches.get_flag("shadowed");

    xrf_output::info!(output, "Listing {}", listing.origin);

    for mount in &listing.mounts {
      xrf_output::info!(output, "  {}", MountDescription::describe(mount));
    }

    Self::print_entries(&output, &listing);

    if is_shadowed_included {
      Self::print_shadowed(&output, &listing);
    }

    SourceReachability::print_collisions(&output, &listing.collisions);
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
        format_path(&skipped.path),
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

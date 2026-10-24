use clap::{Arg, ArgAction, ArgMatches, Command};
use xrf_utils::HostLinkPolicy;

use crate::core::arguments::OptionalArgument;

/// `--refuse-links`, on a command that writes data-named files below a destination.
pub struct LinkPolicyArgument;

impl LinkPolicyArgument {
  const NAME: &'static str = "refuse-links";

  /// Declares `--refuse-links` on `command`.
  pub fn declare(command: Command) -> Command {
    command.arg(
      Arg::new(Self::NAME)
        .help("Refuse to write through links or junctions already below the destination, instead of following them")
        .long(Self::NAME)
        .required(false)
        .action(ArgAction::SetTrue),
    )
  }

  /// The link policy the command was asked for; following links when the flag was not declared or not given.
  pub fn of(matches: &ArgMatches) -> HostLinkPolicy {
    match OptionalArgument::read::<bool>(matches, Self::NAME) {
      Some(true) => HostLinkPolicy::Refuse,
      _ => HostLinkPolicy::Follow,
    }
  }
}

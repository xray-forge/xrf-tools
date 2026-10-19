use clap::{Arg, ArgAction, ArgMatches, Command};
use xrf_utils::HostLinkPolicy;

const REFUSE_LINKS_ARGUMENT: &str = "refuse-links";

/// Declares the link-policy argument on a command, in the builder style clap itself uses.
pub trait LinkPolicyArguments {
  /// Declares `--refuse-links` on a command that writes data-named files below a destination.
  #[must_use]
  fn with_link_policy(self) -> Self;
}

impl LinkPolicyArguments for Command {
  fn with_link_policy(self) -> Self {
    self.arg(
      Arg::new(REFUSE_LINKS_ARGUMENT)
        .help("Refuse to write through links or junctions already below the destination, instead of following them")
        .long(REFUSE_LINKS_ARGUMENT)
        .required(false)
        .action(ArgAction::SetTrue),
    )
  }
}

/// The link policy this command was asked for; following links when the flag was not declared or not given.
pub fn requested_link_policy(matches: &ArgMatches) -> HostLinkPolicy {
  if matches.try_get_one::<bool>(REFUSE_LINKS_ARGUMENT).ok().flatten() == Some(&true) {
    HostLinkPolicy::Refuse
  } else {
    HostLinkPolicy::Follow
  }
}

#[cfg(test)]
mod tests {
  use clap::Command;
  use xrf_utils::HostLinkPolicy;

  use super::{LinkPolicyArguments, requested_link_policy};

  fn parse(arguments: &[&str]) -> HostLinkPolicy {
    requested_link_policy(
      &Command::new("unpack")
        .with_link_policy()
        .try_get_matches_from(arguments)
        .expect("arguments parse"),
    )
  }

  #[test]
  fn follows_links_unless_asked_to_refuse_them() {
    assert_eq!(parse(&["unpack"]), HostLinkPolicy::Follow);
    assert_eq!(parse(&["unpack", "--refuse-links"]), HostLinkPolicy::Refuse);
  }

  #[test]
  fn follows_links_on_a_command_without_the_flag() {
    assert_eq!(
      requested_link_policy(
        &Command::new("pack")
          .try_get_matches_from(["pack"])
          .expect("arguments parse")
      ),
      HostLinkPolicy::Follow
    );
  }
}

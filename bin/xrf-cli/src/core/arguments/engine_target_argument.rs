use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_engine_target::XrayEngineChoice;

use crate::core::arguments::OptionalArgument;

/// `--engine`, on a command that reads configs the engines read differently.
pub struct EngineTargetArgument;

impl EngineTargetArgument {
  const NAME: &'static str = "engine";

  /// Declares `--engine` on `command`.
  pub fn declare(command: Command) -> Command {
    command.arg(
      Arg::new(Self::NAME)
        .help(
          "Read configs as this engine does: auto detects it from the tree, vanilla for OpenXRay and Call of \
           Pripyat, extended for Anomaly",
        )
        .long(Self::NAME)
        .required(false)
        .value_name("ENGINE")
        .default_value(XrayEngineChoice::default().as_str())
        .value_parser(value_parser!(XrayEngineChoice)),
    )
  }

  /// The engine the command was asked to read configs as; the default for a command that never declared the flag.
  pub fn of(matches: &ArgMatches) -> XrayEngineChoice {
    OptionalArgument::read::<XrayEngineChoice>(matches, Self::NAME)
      .copied()
      .unwrap_or_default()
  }
}

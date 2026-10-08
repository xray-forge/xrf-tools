use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_engine_target::XrayEngineChoice;

const ENGINE_ARGUMENT: &str = "engine";

/// Declares the engine-target argument on a command, in the builder style clap itself uses.
pub trait EngineTargetArguments {
  /// Declares `--engine` on a command that reads configs the engines read differently.
  #[must_use]
  fn with_engine_target(self) -> Self;
}

impl EngineTargetArguments for Command {
  fn with_engine_target(self) -> Self {
    self.arg(
      Arg::new(ENGINE_ARGUMENT)
        .help(
          "Read configs as this engine does: auto detects it from the tree, vanilla for OpenXRay and Call of \
           Pripyat, extended for Anomaly",
        )
        .long(ENGINE_ARGUMENT)
        .required(false)
        .value_name("ENGINE")
        .default_value(XrayEngineChoice::default().as_str())
        .value_parser(value_parser!(XrayEngineChoice)),
    )
  }
}

/// The engine this command was asked to read configs as; the default for a command that never declared the flag.
pub fn requested_engine(matches: &ArgMatches) -> XrayEngineChoice {
  matches
    .try_get_one::<XrayEngineChoice>(ENGINE_ARGUMENT)
    .ok()
    .flatten()
    .copied()
    .unwrap_or_default()
}

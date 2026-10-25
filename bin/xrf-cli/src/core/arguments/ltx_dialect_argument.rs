use std::sync::Arc;

use clap::{Arg, ArgAction, ArgMatches, Command};
use xrf_dltx::DltxDialect;
use xrf_ltx::LtxDialect;

use crate::core::arguments::OptionalArgument;

/// `--dltx`, on a command that resolves game configs.
pub struct LtxDialectArgument;

impl LtxDialectArgument {
  const NAME: &'static str = "dltx";

  /// Declares `--dltx` on `command`.
  pub fn declare(command: Command) -> Command {
    command.arg(
      Arg::new(Self::NAME)
        .help("Resolve configs with the Monolith/Anomaly DLTX patch dialect, applying mod_<base>_*.ltx files")
        .long(Self::NAME)
        .required(false)
        .action(ArgAction::SetTrue),
    )
  }

  /// The dialect the command was asked to resolve configs under: standard LTX for a command that never declared the
  /// flag, which is what it would have used anyway.
  pub fn of(matches: &ArgMatches) -> Arc<dyn LtxDialect> {
    DltxDialect::select(OptionalArgument::read::<bool>(matches, Self::NAME) == Some(&true))
  }
}

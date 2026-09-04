use std::sync::Arc;

use clap::{Arg, ArgAction, ArgMatches, Command};
use xrf_dltx::select_ltx_dialect;
use xrf_ltx::LtxDialect;

const DLTX_ARGUMENT: &str = "dltx";

/// Declares the config-dialect argument on a command, in the builder style clap itself uses.
///
/// The counterpart to [`crate::core::execution::ExecutionArguments`] and
/// [`crate::core::reporting::ReportingArguments`], so there is one way to attach a named group of arguments to a
/// command rather than one idiom per group. Five commands spelled the flag, its help text and its action by hand
/// before this existed, which is five places for the wording to drift.
pub trait LtxDialectArguments {
  /// Declares `--dltx` on a command that resolves game configs.
  #[must_use]
  fn with_ltx_dialect(self) -> Self;
}

impl LtxDialectArguments for Command {
  fn with_ltx_dialect(self) -> Self {
    self.arg(
      Arg::new(DLTX_ARGUMENT)
        .help("Resolve configs with the Monolith/Anomaly DLTX patch dialect, applying mod_<base>_*.ltx files")
        .long(DLTX_ARGUMENT)
        .required(false)
        .action(ArgAction::SetTrue),
    )
  }
}

/// The dialect this command was asked to resolve configs under.
///
/// Asking the parsed arguments rather than a registry, because clap already knows: a command that never declared the
/// flag cannot have matched it, and answers standard LTX, which is what it would have used anyway.
pub fn requested_ltx_dialect(matches: &ArgMatches) -> Arc<dyn LtxDialect> {
  select_ltx_dialect(matches.try_get_one::<bool>(DLTX_ARGUMENT).ok().flatten() == Some(&true))
}

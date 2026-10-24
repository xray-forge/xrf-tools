use clap::ArgMatches;

/// Reads an argument some commands declare and others do not.
pub struct OptionalArgument;

impl OptionalArgument {
  /// The value of the argument `name`, none where the command never declared it or the caller gave none: asking the
  /// parsed arguments rather than a registry, since clap already knows which commands declared what.
  pub fn read<'a, T: Clone + Send + Sync + 'static>(matches: &'a ArgMatches, name: &str) -> Option<&'a T> {
    matches.try_get_one::<T>(name).ok().flatten()
  }
}

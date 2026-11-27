use env_filter::Filter;
use env_logger::Builder;
use log::LevelFilter;
use xrf_env_flags::EnvFlags;

/// The filter rules every log of the application shares, read once, so the backend's records and the UI's answer to
/// one `RUST_LOG`.
pub struct LogFilter {
  /// The directives in `RUST_LOG`'s syntax, none where it is not set.
  directives: Option<String>,
}

impl LogFilter {
  /// The level every record is filtered at where no directive is given. Debug builds trace every command dispatch;
  /// release keeps the info lines the commands write, which is what makes a user's log useful in a bug report.
  const DEFAULT_LEVEL: LevelFilter = match cfg!(debug_assertions) {
    true => LevelFilter::Debug,
    false => LevelFilter::Info,
  };

  /// Dependencies held to warnings where no directive is given: the GPU stack logs at debug for every shader it
  /// compiles, naga each overload it weighs, millions of lines that slow the render thread and bury the log. A
  /// `RUST_LOG` directive naming one brings it back.
  const QUIET_TARGETS: [&'static str; 4] = ["naga", "wgpu", "wgpu_core", "wgpu_hal"];

  /// The rules `RUST_LOG` sets.
  pub fn from_environment() -> Self {
    Self {
      directives: EnvFlags::RUST_LOG.read_text(),
    }
  }

  /// The default rules, as with no `RUST_LOG`.
  #[cfg(any(test, feature = "fixtures"))]
  pub fn without_directives() -> Self {
    Self { directives: None }
  }

  /// The rules `directives` set, spelled as `RUST_LOG` spells them.
  #[cfg(any(test, feature = "fixtures"))]
  pub fn from_directives(directives: &str) -> Self {
    Self {
      directives: Some(String::from(directives)),
    }
  }

  /// Filters the backend's `logger` by the directives, else at [`Self::DEFAULT_LEVEL`] with
  /// [`Self::QUIET_TARGETS`] at warnings.
  pub fn configure(&self, logger: &mut Builder) {
    match &self.directives {
      Some(directives) => {
        logger.parse_filters(directives);
      }
      None => {
        logger.filter_level(Self::DEFAULT_LEVEL);

        for target in Self::QUIET_TARGETS {
          logger.filter_module(target, LevelFilter::Warn);
        }
      }
    }
  }

  /// The filter the directives make, else the default one, deciding as the backend's logger does
  /// ([`Self::configure`]). It only answers whether a record passes, so a log written apart from the process's logger
  /// follows the same rules.
  pub fn build_filter(&self) -> Filter {
    let mut filter: env_filter::Builder = env_filter::Builder::new();

    match &self.directives {
      Some(directives) => {
        filter.parse(directives);
      }
      None => {
        filter.filter_level(Self::DEFAULT_LEVEL);

        for target in Self::QUIET_TARGETS {
          filter.filter_module(target, LevelFilter::Warn);
        }
      }
    }

    filter.build()
  }

  /// The name each directive gives, cut as env_filter cuts it: every comma-separated directive before a `/` message
  /// filter, trimmed, up to its `=`. A directive that is a bare level gives that level, which names no target anyone
  /// logs at. env_filter keeps its own cut private, so a change to its directive syntax must be followed here.
  pub fn list_names(&self) -> Vec<&str> {
    self
      .directives
      .as_deref()
      .and_then(|directives| directives.split('/').next())
      .into_iter()
      .flat_map(|directives| directives.split(','))
      .map(str::trim)
      .filter(|directive| !directive.is_empty())
      .filter_map(|directive| directive.split('=').next())
      .collect()
  }
}

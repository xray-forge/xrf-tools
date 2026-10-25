use std::path::PathBuf;

use clap::ArgMatches;
use xrf_error::XrfResult;
use xrf_output::OutputOptions;
use xrf_pack::{ArchivePatchConfig, ArchivePatchOptions};
use xrf_utils::format_path;

use crate::commands::archive::{ArchiveVolumeArguments, ArchiveVolumeRequest};

/// What `archive pack-patch` was asked to do, read from its arguments with every path made absolute.
pub(crate) struct ArchivePatchRequest {
  pub(crate) config: ArchivePatchConfig,
  pub(crate) options: ArchivePatchOptions,
  pub(crate) is_dry_run: bool,
}

impl ArchivePatchRequest {
  /// Parses patch arguments and resolves absolute paths.
  ///
  /// # Errors
  ///
  /// Returns an error if path resolution fails, a header entry is not `<key>=<value>`, or the volume cap requires
  /// `--oversized-volumes`.
  pub(crate) fn of(matches: &ArgMatches, output: OutputOptions) -> XrfResult<Self> {
    let volumes: ArchiveVolumeRequest = ArchiveVolumeArguments::of(matches)?;
    let mut config: ArchivePatchConfig = ArchivePatchConfig::new(
      xrf_utils::to_absolute_path(
        matches
          .get_one::<PathBuf>("path")
          .expect("Expected a provided root to be read"),
      )?,
      &volumes.dest,
      &volumes.name,
    );

    if let Some(target) = matches.get_one::<PathBuf>("target") {
      config.target = Some(xrf_utils::to_absolute_path(target)?);
    }

    // One selection source or the other, never both: clap refuses `--config` beside a selection option, so whichever
    // is present supplies the whole scope. The run options below still layer over either.
    if let Some(path) = matches.get_one::<PathBuf>("config") {
      xrf_output::info!(output, "Patch config: {}", format_path(path));

      config = config.with_config_file(path)?;
    }

    if let Some(prefixes) = matches.get_many::<String>("include") {
      config.include = prefixes.cloned().collect();
    }

    if let Some(prefixes) = matches.get_many::<String>("ignore") {
      config.ignore = prefixes.cloned().collect();
    }

    if let Some(extensions) = matches.get_many::<String>("exclude-extension") {
      config.exclude_extensions = extensions.cloned().collect();
    }

    Ok(Self {
      config: volumes.apply_to_patch(config)?,
      options: ArchivePatchOptions::default()
        .with_output(output)
        .with_force(volumes.is_forced)
        .with_verified_payloads(matches.get_flag("verify-payload")),
      is_dry_run: matches.get_flag("dry-run"),
    })
  }
}

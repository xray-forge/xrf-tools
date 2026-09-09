use std::path::PathBuf;

use clap::ArgMatches;
use clap::parser::ValueSource;
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_pack::{
  ArchivePackHeaderEntry, ArchivePackMode, ArchivePatchConfig, ArchivePatchOptions, ArchivePatchShape,
  ArchiveVolumeExtension, VOLUME_SIZE_MAX,
};

/// Parsed arguments for `archive pack-patch`.
pub(crate) struct ArchivePatchArguments {
  pub(crate) config: ArchivePatchConfig,
  pub(crate) options: ArchivePatchOptions,
  pub(crate) is_dry_run: bool,
}

impl ArchivePatchArguments {
  /// Parses patch arguments and resolves absolute paths.
  ///
  /// # Errors
  ///
  /// Returns an error if path resolution fails, a header entry lacks `=`, or the volume cap requires
  /// `--oversized-volumes`.
  pub(crate) fn of(matches: &ArgMatches, output: OutputOptions) -> XrfResult<Self> {
    let mut config: ArchivePatchConfig = ArchivePatchConfig::new(
      Self::to_root(matches, "input")?,
      xrf_utils::to_absolute_path(
        matches
          .get_one::<PathBuf>("dest")
          .expect("Expected valid output path to be provided"),
      )?,
      matches
        .get_one::<String>("name")
        .expect("Expected valid archive name to be provided"),
    );

    if matches.contains_id("target") && matches.get_one::<PathBuf>("target").is_some() {
      config.target = Some(Self::to_root(matches, "target")?);
    }

    if matches.get_flag("release") {
      config.shape = ArchivePatchShape::Release;
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

    if let Some(entries) = matches.get_many::<String>("header") {
      config.header = Some(ArchivePackHeaderEntry::join(
        &entries
          .map(|entry| Self::to_header_entry(entry))
          .collect::<XrfResult<Vec<_>>>()?,
      ));
    }

    if matches.get_flag("store") {
      config.mode = ArchivePackMode::Store;
    }

    if matches.get_flag("xdb") {
      config.volume_extension = ArchiveVolumeExtension::Xdb;
    }

    config.is_with_oversized_volumes = matches.get_flag("oversized-volumes");
    config.max_volume_size =
      Self::to_volume_size(matches, config.is_with_oversized_volumes)?.unwrap_or(config.max_volume_size);

    Ok(Self {
      config,
      options: ArchivePatchOptions::default()
        .with_output(output)
        .with_force(matches.get_flag("force"))
        .with_strict(matches.get_flag("strict"))
        .with_verified_payloads(matches.get_flag("verify-payload")),
      is_dry_run: matches.get_flag("dry-run"),
    })
  }

  /// The root of one side, made absolute.
  fn to_root(matches: &ArgMatches, argument: &str) -> XrfResult<PathBuf> {
    xrf_utils::to_absolute_path(
      matches
        .get_one::<PathBuf>(argument)
        .expect("Expected a provided root to be read"),
    )
  }

  /// One `--header key=value` entry, split at the first `=`.
  fn to_header_entry(entry: &str) -> XrfResult<ArchivePackHeaderEntry> {
    let Some((key, value)) = entry.split_once('=') else {
      return Err(XrfError::new_invalid_error(format!(
        "Header entry '{entry}' names no key: write it as <key>=<value>"
      )));
    };

    Ok(ArchivePackHeaderEntry {
      key: key.trim().to_owned(),
      value: value.trim().to_owned(),
    })
  }

  /// Returns the explicit volume cap in bytes, or `None` to use the default.
  fn to_volume_size(matches: &ArgMatches, is_oversized_allowed: bool) -> XrfResult<Option<u64>> {
    if matches.value_source("max-size") != Some(ValueSource::CommandLine) {
      return Ok(None);
    }

    let Some(megabytes) = matches.get_one::<u64>("max-size") else {
      return Ok(None);
    };

    // `--max-size` is given in megabytes, matching the `-max_size` unit of xrCompress.
    let size: u64 = xrf_utils::megabytes_to_bytes(*megabytes);

    if size > VOLUME_SIZE_MAX && !is_oversized_allowed {
      return Err(XrfError::new_invalid_error(format!(
        "Volume size {} is past the {} MB the engine mounts. Pass --oversized-volumes to publish volumes only a fork \
         that raised XRP_MAX_SIZE can open.",
        xrf_utils::format_bytes(size),
        VOLUME_SIZE_MAX / xrf_utils::BYTES_PER_MEGABYTE
      )));
    }

    Ok(Some(size))
  }
}

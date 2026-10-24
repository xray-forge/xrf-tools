use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_pack::{ArchivePackHeaderEntry, VOLUME_SIZE_MAX, VOLUME_SIZE_MIN};

use crate::commands::archive::ArchiveVolumeRequest;

/// The volume flags `archive pack` and `archive pack-patch` share, declared and read in one place so the two cannot
/// drift: `--dest`, `--name`, `--header`, `--store`, `--max-size`, `--oversized-volumes`, `--xdb` and `--force`.
pub struct ArchiveVolumeArguments;

impl ArchiveVolumeArguments {
  const DEST: &'static str = "dest";
  const NAME: &'static str = "name";
  /// A header entry, which a configuration file also carries.
  pub const HEADER: &'static str = "header";
  const STORE: &'static str = "store";
  const MAX_SIZE: &'static str = "max-size";
  const OVERSIZED: &'static str = "oversized-volumes";
  const XDB: &'static str = "xdb";
  const FORCE: &'static str = "force";

  /// Declares the flags on `command`, the volumes named `default_name` unless `--name` says otherwise.
  pub fn declare(command: Command, default_name: &'static str) -> Command {
    command
      .arg(
        Arg::new(Self::DEST)
          .help("Folder the volumes are written to")
          .short('d')
          .long(Self::DEST)
          .default_value("packed")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new(Self::NAME)
          .help("Base name of the volumes, written as <name>.db0, <name>.db1 and so on")
          .short('n')
          .long(Self::NAME)
          .default_value(default_name)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new(Self::HEADER)
          .help("Header entry written into the archive as <key>=<value>, repeatable, merged over the default header")
          .long(Self::HEADER)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new(Self::STORE)
          .help("Store every file instead of compressing what the engine expects compressed")
          .long(Self::STORE)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new(Self::MAX_SIZE)
          .help(format!(
            "Maximum volume size in megabytes, from {} to {}",
            VOLUME_SIZE_MIN / xrf_utils::BYTES_PER_MEGABYTE,
            VOLUME_SIZE_MAX / xrf_utils::BYTES_PER_MEGABYTE
          ))
          .long(Self::MAX_SIZE)
          .value_parser(value_parser!(u64).range(1..)),
      )
      .arg(
        Arg::new(Self::OVERSIZED)
          .help(format!(
            "Let --max-size exceed {} MB, which only an engine fork that raised XRP_MAX_SIZE can mount",
            VOLUME_SIZE_MAX / xrf_utils::BYTES_PER_MEGABYTE
          ))
          .long(Self::OVERSIZED)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new(Self::XDB)
          .help("Write volumes with the xdb extension")
          .long(Self::XDB)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new(Self::FORCE)
          .help("Replace volumes of the same set the destination already holds")
          .short('f')
          .long(Self::FORCE)
          .action(ArgAction::SetTrue),
      )
  }

  /// The volumes the command was asked to write, its destination made absolute.
  ///
  /// # Errors
  ///
  /// Returns an error when the destination cannot be made absolute, a header entry is not `<key>=<value>`, or the
  /// volume size needs `--oversized-volumes`.
  pub fn of(matches: &ArgMatches) -> XrfResult<ArchiveVolumeRequest> {
    let is_oversized_allowed: bool = matches.get_flag(Self::OVERSIZED);

    Ok(ArchiveVolumeRequest {
      dest: xrf_utils::to_absolute_path(
        matches
          .get_one::<PathBuf>(Self::DEST)
          .expect("Expected the destination to default"),
      )?,
      name: matches
        .get_one::<String>(Self::NAME)
        .expect("Expected the volume name to default")
        .clone(),
      header: matches
        .get_many::<String>(Self::HEADER)
        .map(|entries| {
          entries
            .map(|entry| ArchivePackHeaderEntry::parse(entry))
            .collect::<XrfResult<Vec<_>>>()
        })
        .transpose()?
        // Merged over the defaults: naming one key on a command line is adding it, and the two the engine reads
        // unconditionally are the ones nobody retypes.
        .map(|entries| ArchivePackHeaderEntry::over_default(&entries)),
      is_stored: matches.get_flag(Self::STORE),
      is_xdb: matches.get_flag(Self::XDB),
      max_volume_size: Self::to_volume_size(matches, is_oversized_allowed)?,
      is_oversized_allowed,
      is_forced: matches.get_flag(Self::FORCE),
    })
  }

  /// The volume cap asked for, in bytes; none where `--max-size` was not given.
  fn to_volume_size(matches: &ArgMatches, is_oversized_allowed: bool) -> XrfResult<Option<u64>> {
    let Some(megabytes) = matches.get_one::<u64>(Self::MAX_SIZE) else {
      return Ok(None);
    };

    // In megabytes, matching the `-max_size` unit of xrCompress.
    let size: u64 = xrf_utils::megabytes_to_bytes(*megabytes);

    // Asked here as well as in the configuration, so the refusal can name the flag that lifts it. The configuration
    // refuses the same size regardless; this is the surface's own wording, not its own rule.
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

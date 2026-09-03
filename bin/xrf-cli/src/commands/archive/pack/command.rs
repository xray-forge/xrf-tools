use std::path::PathBuf;

use clap::parser::ValueSource;
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_pack::{
  ArchivePackConfig, ArchivePackMode, ArchivePackOptions, ArchivePackResult, ArchivePacker, ArchiveVolumeExtension,
  VOLUME_SIZE_MAX, VOLUME_SIZE_MIN,
};
use xrf_utils::format_path;

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct PackCommand;

impl GenericCommand for PackCommand {
  fn operation(&self) -> &'static str {
    "pack"
  }

  /// Create command to pack a directory into archive volumes.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to pack provided directory into *.db archive volumes")
      .arg(
        Arg::new("path")
          .help("Path to the directory to pack, normally a gamedata root")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to folder for writing the volumes")
          .short('d')
          .long("dest")
          .default_value("packed")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("name")
          .help("Base name of the volumes, written as <name>.db0, <name>.db1 and so on")
          .short('n')
          .long("name")
          .default_value("gamedata")
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("config")
          .help("Path to a packing configuration describing what to include, as *.ltx or *.json")
          .long("config")
          .required(false)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("store")
          .help("Store every file instead of compressing what the engine expects compressed")
          .long("store")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("max-size")
          .help(format!(
            "Maximum volume size in megabytes, from {} to {}",
            VOLUME_SIZE_MIN / xrf_utils::BYTES_PER_MEGABYTE,
            VOLUME_SIZE_MAX / xrf_utils::BYTES_PER_MEGABYTE
          ))
          .long("max-size")
          .required(false)
          .value_parser(value_parser!(u64).range(1..)),
      )
      .arg(
        Arg::new("oversized-volumes")
          .help(format!(
            "Let --max-size exceed {} MB, which only an engine fork that raised XRP_MAX_SIZE can mount",
            VOLUME_SIZE_MAX / xrf_utils::BYTES_PER_MEGABYTE
          ))
          .long("oversized-volumes")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("xdb")
          .help("Write volumes with the xdb extension")
          .long("xdb")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("no-skip-list")
          .help("Keep editor and source leftovers the engine build normally drops")
          .long("no-skip-list")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("force")
          .help("Replace volumes of the same set the destination already holds")
          .short('f')
          .long("force")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Pack a directory into xray engine database archives.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: PathBuf = xrf_utils::to_absolute_path(
      matches
        .get_one::<PathBuf>("path")
        .expect("Expected valid source path to be provided"),
    )?;

    let destination: PathBuf = xrf_utils::to_absolute_path(
      matches
        .get_one::<PathBuf>("dest")
        .expect("Expected valid output path to be provided"),
    )?;

    let name: &String = matches
      .get_one::<_>("name")
      .expect("Expected valid archive name to be provided");

    let output: OutputOptions = context.get_output().clone();

    let mut config: ArchivePackConfig = ArchivePackConfig::new(&path, &destination, name);

    // The configuration file supplies defaults; anything named on the command line wins over it.
    if let Some(path) = matches.get_one::<PathBuf>("config") {
      xrf_output::info!(output, "Pack config: {}", format_path(path));

      config = config.with_config_file(path)?;
    }

    if matches.get_flag("store") {
      config.mode = ArchivePackMode::Store;
    }

    if matches.get_flag("xdb") {
      config.volume_extension = ArchiveVolumeExtension::Xdb;
    }

    if matches.get_flag("no-skip-list") {
      config.is_with_skip_list = false;
    }

    // Before the size it lifts the bound for, which is the order `with_max_volume_size` reads them in.
    let is_oversized_allowed: bool = matches.get_flag("oversized-volumes");

    config = config.with_oversized_volumes(is_oversized_allowed);

    if matches.value_source("max-size") == Some(ValueSource::CommandLine)
      && let Some(size) = matches.get_one::<u64>("max-size")
    {
      // `--max-size` is given in megabytes, matching the `-max_size` unit of xrCompress.
      let size: u64 = xrf_utils::megabytes_to_bytes(*size);

      // Asked here as well as in the configuration, so the refusal can name the flag that lifts it. The
      // configuration refuses the same size regardless; this is the surface's own wording, not its own rule.
      if size > VOLUME_SIZE_MAX && !is_oversized_allowed {
        return Err(
          XrfError::new_invalid_error(format!(
            "Volume size {} is past the {} MB the engine mounts. Pass --oversized-volumes to publish \
             volumes only a fork that raised XRP_MAX_SIZE can open.",
            xrf_utils::format_bytes(size),
            VOLUME_SIZE_MAX / xrf_utils::BYTES_PER_MEGABYTE
          ))
          .into(),
        );
      }

      config = config.with_max_volume_size(size)?;
    }

    xrf_output::info!(output, "Pack source: {}", format_path(&path));
    xrf_output::info!(output, "Pack destination: {}", format_path(&destination));

    if config.max_volume_size > VOLUME_SIZE_MAX {
      xrf_output::warning!(
        output,
        "Volumes are capped at {}, past the {} MB no unmodified engine mounts. These volumes load only in \
         a fork that raised XRP_MAX_SIZE.",
        xrf_utils::format_bytes(config.max_volume_size),
        VOLUME_SIZE_MAX / xrf_utils::BYTES_PER_MEGABYTE
      );
    }

    // A headerless archive is not neutral to the engine: unless it is named `xdb`, the loader assumes it
    // is an encrypted Shadow of Chernobyl archive and decrypts it into nonsense.
    if config.header.is_none() && config.volume_extension != ArchiveVolumeExtension::Xdb {
      xrf_output::warning!(
        output,
        "No [header] section configured: the engine will read these volumes as encrypted ShoC archives. \
         Supply --config with a [header] naming an entry_point, or pass --xdb."
      );
    }

    let is_forced: bool = matches.get_flag("force");

    // Asked here as well as inside the pack, so the refusal can name the flag that lifts it. Packing refuses the same
    // destination regardless; this is the surface's own wording, not its own rule.
    if !is_forced {
      let published: Vec<PathBuf> = ArchivePacker::list_published_volumes(&config)?;

      if !published.is_empty() {
        return Err(
          XrfError::new_invalid_error(format!(
            "Destination '{}' already holds {} volume(s) of '{}'. Packing replaces them and cannot put them back if it \
           fails partway: move them aside, or pass --force to overwrite them.",
            format_path(&destination),
            published.len(),
            config.single_volume_name()
          ))
          .into(),
        );
      }
    }

    let result: ArchivePackResult =
      ArchivePacker::pack_opt(&config, ArchivePackOptions::default().with_force(is_forced))?;

    for volume in &result.volumes {
      xrf_output::info!(output, "Wrote {}", format_path(volume));
    }

    xrf_output::success!(
      output,
      "Packed {} file(s) into {} volume(s) in {}",
      result.files_total,
      result.volumes.len(),
      xrf_utils::format_duration(result.duration),
    );

    xrf_output::info!(
      output,
      "Summary: {} compressed, {} stored, {} aliased, {} skipped",
      result.files_compressed,
      result.files_stored,
      result.files_aliased,
      result.files_skipped,
    );

    let (size_source, size_written): (String, String) =
      xrf_utils::format_bytes_pair(result.size_source, result.size_written);

    xrf_output::info!(output, "Size: {size_source} source, {size_written} written");

    context.set_result(|| &result)?;

    Ok(())
  }
}

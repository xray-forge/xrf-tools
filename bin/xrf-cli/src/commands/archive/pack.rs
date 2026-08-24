use std::env;
use std::path::PathBuf;

use clap::parser::ValueSource;
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_pack::{ArchivePackConfig, ArchivePackMode, ArchivePackResult, ArchivePacker, ArchiveVolumeExtension};

use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

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
        Arg::new("ltx")
          .help("Path to an xrCompress configuration LTX describing what to include")
          .long("ltx")
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
          .help("Maximum volume size in megabytes")
          .long("max-size")
          .required(false)
          .value_parser(value_parser!(u64).range(1..)),
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
        Arg::new("silent")
          .help("Turn off logging")
          .short('s')
          .long("silent")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("verbose")
          .help("Turn on verbose logging")
          .short('v')
          .long("verbose")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Pack a directory into xray engine database archives.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid source path to be provided");

    let destination: &PathBuf = matches
      .get_one::<_>("dest")
      .expect("Expected valid output path to be provided");

    let destination: PathBuf = if destination.is_relative() {
      env::current_dir()?.join(destination)
    } else {
      destination.clone()
    };

    let name: &String = matches
      .get_one::<_>("name")
      .expect("Expected valid archive name to be provided");

    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    let mut config: ArchivePackConfig = ArchivePackConfig::new(path, &destination, name);

    // The configuration file supplies defaults; anything named on the command line wins over it.
    if let Some(ltx) = matches.get_one::<PathBuf>("ltx") {
      xrf_output::info!(output, "Pack config: {}", ltx.display());

      config = config.with_ltx_file(ltx)?;
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

    if matches.value_source("max-size") == Some(ValueSource::CommandLine)
      && let Some(size) = matches.get_one::<u64>("max-size")
    {
      // `--max-size` is given in megabytes, matching the `-max_size` unit of xrCompress.
      config = config.with_max_volume_size(xrf_utils::megabytes_to_bytes(*size))?;
    }

    xrf_output::info!(output, "Pack source: {}", path.display());
    xrf_output::info!(output, "Pack destination: {}", destination.display());

    // A headerless archive is not neutral to the engine: unless it is named `xdb`, the loader assumes it
    // is an encrypted Shadow of Chernobyl archive and decrypts it into nonsense.
    if config.header.is_none() && config.volume_extension != ArchiveVolumeExtension::Xdb {
      xrf_output::warning!(
        output,
        "No [header] section configured: the engine will read these volumes as encrypted ShoC archives. \
         Supply --ltx with a [header] naming an entry_point, or pass --xdb."
      );
    }

    let result: ArchivePackResult = ArchivePacker::pack(&config)?;

    for volume in &result.volumes {
      xrf_output::info!(output, "Wrote {}", volume.display());
    }

    xrf_output::info!(
      output,
      "Packed {} file(s) into {} volume(s), took {}",
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

    Ok(())
  }
}

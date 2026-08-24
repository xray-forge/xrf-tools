use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_dds::ImageFormat;
use xrf_error::XrfError;
use xrf_ltx::Ltx;
use xrf_output::OutputOptions;
use xrf_texture::{PackEquipmentOptions, PackEquipmentProcessor};

use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct PackEquipmentIconsCommand;

impl GenericCommand for PackEquipmentIconsCommand {
  fn operation(&self) -> &'static str {
    "pack-equipment-icons"
  }

  /// Create command for packing equipment icons.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to pack dds icons into single element")
      .arg(
        Arg::new("system-ltx")
          .help("Path to system ltx file or root folder with ltx files")
          .long("system-ltx")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("source")
          .help("Path to source folder with section icons")
          .long("source")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("output")
          .help("Path to output dds file")
          .long("output")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("gamedata")
          .help("Path to gamedata folder for resources usage")
          .long("gamedata")
          .required(false)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("silent")
          .help("Turn off logging")
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
      .arg(
        Arg::new("strict")
          .help("Turn on strict mode")
          .short('s')
          .long("strict")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Command to pack equipment icons files into single dds file.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let system_ltx_path: &PathBuf = matches
      .get_one::<PathBuf>("system-ltx")
      .expect("Expected valid path to be provided for system-ltx");

    let source: &PathBuf = matches
      .get_one::<PathBuf>("source")
      .expect("Expected valid source path to be provided");

    let gamedata: Option<&PathBuf> = matches.get_one::<PathBuf>("gamedata");

    let output: &PathBuf = matches
      .get_one::<PathBuf>("output")
      .expect("Expected valid output path to be provided");

    let is_strict: bool = matches.get_flag("strict");

    let output_options: OutputOptions =
      TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    if !source.is_dir() {
      return Err(
        XrfError::new_read_error(format!(
          "Expected valid source folder containing DDS icons, got: {}",
          source.display()
        ))
        .into(),
      );
    }

    xrf_output::info!(output_options, "Starting packing DDS icons file, parallel");
    xrf_output::info!(output_options, "System ltx: {}", system_ltx_path.display());
    xrf_output::info!(output_options, "Source icons dir: {}", source.display());
    xrf_output::info!(output_options, "Output dir: {}", output.display());

    let started_at: Instant = Instant::now();
    let system_ltx: Ltx = Ltx::read_from_file_full(system_ltx_path)?;

    let options = PackEquipmentOptions {
      ltx: system_ltx,
      source: source.into(),
      output: output_options.clone(),
      output_path: output.into(),
      gamedata: gamedata.cloned(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      is_strict,
    };

    log::info!("DDS format: {}", options.dds_compression_format);

    PackEquipmentProcessor::pack_sprites(options)?;

    xrf_output::info!(
      output_options,
      "Saved resulting file with combined icons {}",
      output.display()
    );

    log::info!(
      "Pack equipment took: {}",
      xrf_utils::format_duration(started_at.elapsed())
    );

    Ok(())
  }
}

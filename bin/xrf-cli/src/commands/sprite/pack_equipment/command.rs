use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_dds::ImageFormat;
use xrf_error::XrfError;
use xrf_ltx::Ltx;
use xrf_output::OutputOptions;
use xrf_texture::{PackEquipmentOptions, PackEquipmentProcessor, PackEquipmentResult};
use xrf_utils::format_path;

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::progress::new_logging_job;

#[derive(Default)]
pub struct PackEquipmentCommand;

impl GenericCommand for PackEquipmentCommand {
  fn operation(&self) -> &'static str {
    "pack-equipment"
  }

  /// Create command for packing the equipment sprite.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to pack an equipment sprite from separate icon files")
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
        Arg::new("strict")
          .help("Turn on strict mode")
          .long("strict")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Command to pack separate icon files into the equipment sprite.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
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

    let output_options: OutputOptions = context.get_output().clone();

    if !source.is_dir() {
      return Err(
        XrfError::new_read_error(format!(
          "Expected valid source folder containing DDS icons, got: {}",
          format_path(source)
        ))
        .into(),
      );
    }

    xrf_output::info!(output_options, "Starting packing the equipment sprite, parallel");
    xrf_output::info!(output_options, "System ltx: {}", format_path(system_ltx_path));
    xrf_output::info!(output_options, "Source icons dir: {}", format_path(source));
    xrf_output::info!(output_options, "Output dir: {}", format_path(output));

    let started_at: Instant = Instant::now();
    let system_ltx: Ltx = Ltx::read_from_file_full(system_ltx_path)?;

    let options = PackEquipmentOptions {
      job: new_logging_job(),
      ltx: system_ltx,
      source: source.into(),
      output: output_options.clone(),
      output_path: output.into(),
      gamedata: gamedata.cloned(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      is_strict,
    };

    log::info!("DDS format: {}", options.dds_compression_format);

    let result: PackEquipmentResult = PackEquipmentProcessor::pack_sprites(options)?;

    context.set_result(|| &result)?;

    xrf_output::info!(
      output_options,
      "Saved the packed equipment sprite {}",
      format_path(output)
    );

    log::info!(
      "Pack equipment took: {}",
      xrf_utils::format_duration(started_at.elapsed())
    );

    Ok(())
  }
}

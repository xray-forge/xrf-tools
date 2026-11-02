use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_dds::ImageFormat;
use xrf_error::XrfError;
use xrf_ltx::Ltx;
use xrf_output::OutputOptions;
use xrf_texture::{EquipmentSpritePackOptions, EquipmentSpritePackResult, EquipmentSpritePacker};
use xrf_utils::format_path;

use crate::core::arguments::LtxDialectArgument;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

#[derive(Default)]
pub struct PackEquipmentCommand;

impl GenericCommand for PackEquipmentCommand {
  fn operation(&self) -> &'static str {
    "pack-equipment"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about("Pack an equipment sprite from separate icon files")
      .arg(
        Arg::new("system-ltx")
          .help("Path to system ltx file or root folder with ltx files")
          .long("system-ltx")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("path")
          .help("Folder holding one icon per section")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Equipment sprite dds file to write")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("gamedata")
          .help("Path to gamedata folder for resources usage")
          .long("gamedata")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("strict")
          .help("Fail on an icon that cannot be read or placed instead of skipping it")
          .long("strict")
          .action(ArgAction::SetTrue),
      );

    LtxDialectArgument::declare(command)
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let system_ltx_path: &PathBuf = matches
      .get_one::<PathBuf>("system-ltx")
      .expect("Expected valid path to be provided for system-ltx");

    let source: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid source path to be provided");

    let gamedata: Option<&PathBuf> = matches.get_one::<PathBuf>("gamedata");

    let output: &PathBuf = matches
      .get_one::<PathBuf>("dest")
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

    xrf_output::info!(output_options, "Packing the equipment sprite");
    xrf_output::info!(output_options, "System ltx: {}", format_path(system_ltx_path));
    xrf_output::info!(output_options, "Source icons folder: {}", format_path(source));
    xrf_output::info!(output_options, "Sprite file: {}", format_path(output));

    let started_at: Instant = Instant::now();
    let system_ltx: Ltx = Ltx::read_from_file_with_dialect(system_ltx_path, LtxDialectArgument::of(matches).as_ref())?;

    let options = EquipmentSpritePackOptions {
      job: CliJob::for_output(&output_options),
      ltx: system_ltx,
      source: source.into(),
      output: output_options.clone(),
      output_path: output.into(),
      gamedata: gamedata.cloned(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      is_strict,
    };

    xrf_output::verbose!(output_options, "DDS format: {}", options.dds_compression_format);

    let result: EquipmentSpritePackResult = EquipmentSpritePacker::pack_sprites(options)?;

    context.set_result(|| &result)?;

    xrf_output::success!(
      output_options,
      "Saved the packed equipment sprite {} in {}",
      format_path(output),
      xrf_utils::format_duration(started_at.elapsed())
    );

    Ok(())
  }
}

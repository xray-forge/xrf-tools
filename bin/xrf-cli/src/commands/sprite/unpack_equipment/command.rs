use std::fs::create_dir_all;
use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_dds::{DdsFile, DdsMetadata, ImageFormat};
use xrf_ltx::Ltx;
use xrf_output::OutputOptions;
use xrf_texture::{EquipmentSpriteUnpackOptions, EquipmentSpriteUnpacker};
use xrf_utils::format_path;

use super::report::SpriteEquipmentUnpackReport;
use crate::core::arguments::{LinkPolicyArgument, LtxDialectArgument};
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

#[derive(Default)]
pub struct UnpackEquipmentCommand;

impl GenericCommand for UnpackEquipmentCommand {
  fn operation(&self) -> &'static str {
    "unpack-equipment"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about("Unpack separate icon files out of an equipment sprite")
      .arg(
        Arg::new("system-ltx")
          .help("Path to system ltx file or root folder with ltx files")
          .long("system-ltx")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("path")
          .help("Equipment sprite dds file to read")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Folder the section icons are written to")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      );

    LinkPolicyArgument::declare(LtxDialectArgument::declare(command))
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let system_ltx_path: &PathBuf = matches
      .get_one::<PathBuf>("system-ltx")
      .expect("Expected valid path to be provided for system-ltx");

    let source: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid source path to be provided");

    let output: &PathBuf = matches
      .get_one::<PathBuf>("dest")
      .expect("Expected valid output folder path to be provided");

    let output_options: OutputOptions = context.get_output().clone();

    let started_at: Instant = Instant::now();

    xrf_output::info!(output_options, "Opening DDS file: {}", format_path(source));

    let source_file: DdsFile = DdsFile::read_from_path(source)?;
    let metadata: DdsMetadata = source_file.metadata();

    xrf_output::info!(
      output_options,
      "Source DDS file details: {}x{}, mip-maps: {}, format: {:?}",
      metadata.width,
      metadata.height,
      metadata.declared_mipmap_levels.unwrap_or(0),
      metadata.dx10_format
    );

    let source_dds = source_file.decode_rgba(0)?;
    let system_ltx: Ltx = Ltx::read_from_file_with_dialect(system_ltx_path, LtxDialectArgument::of(matches).as_ref())?;

    xrf_output::info!(
      output_options,
      "Unpacking the equipment sprite into: {}",
      format_path(output)
    );

    create_dir_all(output)?;

    EquipmentSpriteUnpacker::unpack_sprites(EquipmentSpriteUnpackOptions {
      job: CliJob::for_output(&output_options),
      ltx: system_ltx,
      source: source_dds,
      output: output_options.clone(),
      output_path: output.into(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      link_policy: LinkPolicyArgument::of(matches),
    })?;

    context.set_result(|| SpriteEquipmentUnpackReport::new(source, system_ltx_path, output, &metadata))?;

    xrf_output::success!(
      output_options,
      "Unpacked the equipment sprite into icons named by its LTX sections in {}",
      xrf_utils::format_duration(started_at.elapsed())
    );

    Ok(())
  }
}

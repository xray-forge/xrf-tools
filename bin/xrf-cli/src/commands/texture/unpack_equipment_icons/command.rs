use std::fs::create_dir_all;
use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_dds::{DdsFile, DdsMetadata, ImageFormat};
use xrf_ltx::Ltx;
use xrf_output::OutputOptions;
use xrf_texture::{UnpackEquipmentOptions, UnpackEquipmentProcessor};

use super::report::TextureEquipmentUnpackReport;
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct UnpackEquipmentIconsCommand;

impl GenericCommand for UnpackEquipmentIconsCommand {
  fn operation(&self) -> &'static str {
    "unpack-equipment-icons"
  }

  /// Create command to unpack equipment icons.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to unpack dds icons into multiple icons")
      .arg(
        Arg::new("system-ltx")
          .help("Path to system ltx file or root folder with ltx files")
          .long("system-ltx")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("source")
          .help("Path to source dds file")
          .long("source")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("output")
          .help("Path to output folder for sections icons")
          .long("output")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let system_ltx_path: &PathBuf = matches
      .get_one::<PathBuf>("system-ltx")
      .expect("Expected valid path to be provided for system-ltx");

    let source: &PathBuf = matches
      .get_one::<PathBuf>("source")
      .expect("Expected valid source path to be provided");

    let output: &PathBuf = matches
      .get_one::<PathBuf>("output")
      .expect("Expected valid output folder path to be provided");

    let output_options: OutputOptions = context.get_output().clone();

    let started_at: Instant = Instant::now();

    xrf_output::info!(output_options, "Opening DDS file: {}", source.display());

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
    let system_ltx: Ltx = Ltx::read_from_file_full(system_ltx_path)?;

    xrf_output::info!(
      output_options,
      "Unpacking equipment DDS file into: {}",
      output.display()
    );

    create_dir_all(output)?;

    UnpackEquipmentProcessor::unpack_sprites(UnpackEquipmentOptions {
      ltx: system_ltx,
      source: source_dds,
      output: output_options.clone(),
      output_path: output.into(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
    })?;

    context.set_result(|| TextureEquipmentUnpackReport::new(source, system_ltx_path, output, &metadata))?;

    xrf_output::info!(output_options, "Successfully DDS equipment file based on LTX sections");

    log::info!(
      "Unpack equipment took: {}",
      xrf_utils::format_duration(started_at.elapsed())
    );

    Ok(())
  }
}

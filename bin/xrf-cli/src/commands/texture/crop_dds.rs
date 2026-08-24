use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_dds::{ImageFormat, Mipmaps};
use xrf_output::OutputOptions;
use xrf_texture::{CropTextureOptions, CropTextureProcessor, CropTextureResult};

use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct CropDdsCommand;

impl GenericCommand for CropDdsCommand {
  fn operation(&self) -> &'static str {
    "crop-dds"
  }

  /// Create command for cropping a region out of a dds file.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to crop a rectangular region out of a dds file into a new dds file")
      .arg(
        Arg::new("source")
          .help("Path to the dds file to read the region from")
          .long("source")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("output")
          .help("Path of the dds file to write")
          .long("output")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("x")
          .help("Left edge of the region, in pixels")
          .long("x")
          .required(true)
          .value_parser(value_parser!(u32)),
      )
      .arg(
        Arg::new("y")
          .help("Top edge of the region, in pixels")
          .long("y")
          .required(true)
          .value_parser(value_parser!(u32)),
      )
      .arg(
        Arg::new("width")
          .help("Width of the region, in pixels")
          .long("width")
          .required(true)
          .value_parser(value_parser!(u32)),
      )
      .arg(
        Arg::new("height")
          .help("Height of the region, in pixels")
          .long("height")
          .required(true)
          .value_parser(value_parser!(u32)),
      )
      .arg(
        Arg::new("fit-width")
          .help("Scale the cropped region to this width, preserving aspect and letterboxing")
          .long("fit-width")
          .requires("fit-height")
          .value_parser(value_parser!(u32)),
      )
      .arg(
        Arg::new("fit-height")
          .help("Scale the cropped region to this height, preserving aspect and letterboxing")
          .long("fit-height")
          .requires("fit-width")
          .value_parser(value_parser!(u32)),
      )
      .arg(
        Arg::new("silent")
          .help("Disable any logging")
          .short('s')
          .long("silent")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("verbose")
          .help("Turn on verbose logging")
          .short('v')
          .long("verbose")
          .action(ArgAction::SetTrue),
      )
  }

  /// Crop a region out of a dds file, optionally scaling it into different bounds.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let source: &PathBuf = matches
      .get_one::<PathBuf>("source")
      .expect("Expected valid source path to be provided");
    let output_path: &PathBuf = matches
      .get_one::<PathBuf>("output")
      .expect("Expected valid output path to be provided");

    let x: u32 = *matches.get_one::<u32>("x").expect("Expected valid x");
    let y: u32 = *matches.get_one::<u32>("y").expect("Expected valid y");
    let width: u32 = *matches.get_one::<u32>("width").expect("Expected valid width");
    let height: u32 = *matches.get_one::<u32>("height").expect("Expected valid height");

    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    let result: CropTextureResult = CropTextureProcessor::crop(&CropTextureOptions {
      source: source.clone(),
      output_path: output_path.clone(),
      output: output.clone(),
      x,
      y,
      width,
      height,
      fit_width: matches.get_one::<u32>("fit-width").copied(),
      fit_height: matches.get_one::<u32>("fit-height").copied(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      // A cropped region is packing input read at its base level, so a mip chain would only cost
      // space.
      dds_mipmaps: Mipmaps::Disabled,
    })?;

    xrf_output::info!(
      output,
      "Wrote {}x{} region from {}:{} of {} to {}",
      result.width,
      result.height,
      x,
      y,
      source.display(),
      output_path.display()
    );

    Ok(())
  }
}

use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_dds::{DdsFile, DdsFormat, DdsMetadata};
use xrf_output::OutputOptions;

use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct InfoDdsCommand;

impl GenericCommand for InfoDdsCommand {
  fn operation(&self) -> &'static str {
    "info-dds"
  }

  /// Create command for printing texture info.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to print information about provided dds file")
      .arg(
        Arg::new("path")
          .help("Path to dds file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
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

  /// Print information about dds file.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid path to be provided");

    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    xrf_output::info!(output, "Read dds file {}", path.display());

    let metadata: DdsMetadata = DdsFile::read_from_path(path)?.metadata();

    xrf_output::info!(
      output,
      "File size: {} ({})",
      metadata.file_size,
      xrf_utils::format_bytes(metadata.file_size)
    );
    xrf_output::info!(output, "Metadata size: {} ", metadata.metadata_size);
    xrf_output::info!(
      output,
      "Data size: {} ({})",
      metadata.data_size,
      xrf_utils::format_bytes(metadata.data_size as u64)
    );
    xrf_output::info!(output, "Size: {} x {}", metadata.width, metadata.height);
    xrf_output::info!(
      output,
      "Mipmap: {} - {}",
      metadata.mipmap_levels,
      metadata.minimum_mipmap_size,
    );

    if let Some(depth) = metadata.depth {
      xrf_output::info!(output, "Depth: {}", depth);
    }

    if let Some(pitch) = metadata.pitch {
      xrf_output::info!(output, "Pitch: {}", pitch);
    }

    if let Some(linear_size) = metadata.linear_size {
      xrf_output::info!(output, "Linear size: {}", linear_size);
    }

    if let Some(block_size) = metadata.block_size {
      xrf_output::info!(output, "Block size: {}", block_size);
    }

    if let Some(bits_per_pixel) = metadata.bits_per_pixel {
      xrf_output::info!(output, "Bits per pixel: {}", bits_per_pixel);
    }

    if let Some(four_cc) = metadata.four_cc {
      xrf_output::info!(output, "Four CC: {}", four_cc);
    }

    if !metadata.has_data_format {
      xrf_output::info!(output, "Format: unknown");
    }

    match metadata.format {
      DdsFormat::D3d(format) => xrf_output::info!(output, "D3D format: {:?}", format),
      DdsFormat::Dxgi(format) => xrf_output::info!(output, "DXGI format: {:?}", format),
      _ => {}
    }

    Ok(())
  }
}

use std::path::PathBuf;

use clap::{ArgMatches, Command};
use xrf_dds::{DdsFile, DdsFormat, DdsMetadata};
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use super::report::DdsInfoReport;
use crate::core::arguments::InputArgument;
use crate::core::command::{CommandContext, CommandInput, CommandResult, GenericCommand};

#[derive(Default)]
pub(crate) struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  fn input(&self) -> CommandInput {
    CommandInput::Path("Path to dds file")
  }

  fn declare(&self, command: Command) -> Command {
    command.about("Print what a DDS file holds")
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = InputArgument::of(matches);

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Read dds file {}", format_path(path));

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

    context.set_result(|| DdsInfoReport::new(&metadata))?;

    Ok(())
  }
}

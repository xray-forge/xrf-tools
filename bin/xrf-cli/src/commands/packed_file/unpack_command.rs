use std::marker::PhantomData;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::{ArgMatches, Command};
use xrf_output::OutputOptions;
use xrf_utils::{format_duration, format_path};

use crate::commands::packed_file::{PackedFileArguments, PackedFileDestination, PackedFileFormat};
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::reports::FileConversionReport;

/// Unpacks one file into a folder of text files.
pub struct UnpackCommand<F>(PhantomData<F>);

impl<F> Default for UnpackCommand<F> {
  fn default() -> Self {
    Self(PhantomData)
  }
}

impl<F: PackedFileFormat> GenericCommand for UnpackCommand<F> {
  fn operation(&self) -> &'static str {
    "unpack"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command.about(format!("Unpack a {} into a folder of text files", F::FILE_NAME));
    let command: Command = PackedFileArguments::<F>::declare_path(command, format!("Packed {}", F::FILE_NAME));
    let command: Command =
      PackedFileArguments::<F>::declare_dest(command, format!("Folder the unpacked {} is written to", F::NOUN));

    PackedFileArguments::<F>::declare_force(command)
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = PackedFileArguments::<F>::path_of(matches);
    let dest: &PathBuf = PackedFileArguments::<F>::dest_of(matches);
    let output: OutputOptions = context.get_output().clone();

    PackedFileDestination::require_free(dest, PackedFileArguments::<F>::is_forced(matches))?;

    xrf_output::info!(output, "Unpacking {} into {}", format_path(path), format_path(dest));

    let started_at: Instant = Instant::now();
    let file: F = F::read(path)?;
    let read_duration: Duration = started_at.elapsed();

    PackedFileDestination::write_folder(dest, |folder| file.export(folder))?;

    let write_duration: Duration = started_at.elapsed() - read_duration;

    xrf_output::verbose!(output, "Read in {}", format_duration(read_duration));
    xrf_output::verbose!(output, "Exported in {}", format_duration(write_duration));
    xrf_output::success!(
      output,
      "Unpacked {} in {}",
      format_path(dest),
      format_duration(started_at.elapsed())
    );

    context.set_result(|| FileConversionReport::new(path, dest, read_duration, write_duration))
  }
}

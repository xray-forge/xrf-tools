use std::marker::PhantomData;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::{ArgMatches, Command};
use xrf_output::OutputOptions;
use xrf_utils::{format_duration, format_path};

use crate::commands::packed_file::{PackedFileArguments, PackedFileDestination, PackedFileFormat};
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::reports::FileConversionReport;

/// Packs an unpacked folder into one file.
pub struct PackCommand<F>(PhantomData<F>);

impl<F> Default for PackCommand<F> {
  fn default() -> Self {
    Self(PhantomData)
  }
}

impl<F: PackedFileFormat> GenericCommand for PackCommand<F> {
  fn operation(&self) -> &'static str {
    "pack"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command.about(format!("Pack an unpacked {} folder into one {}", F::NOUN, F::FILE_NAME));
    let command: Command = PackedFileArguments::declare_path(command, format!("Unpacked {} folder", F::NOUN));
    let command: Command = PackedFileArguments::declare_dest(command, format!("Packed {} written", F::FILE_NAME));

    PackedFileArguments::declare_force(command)
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = PackedFileArguments::path_of(matches);
    let dest: &PathBuf = PackedFileArguments::dest_of(matches);
    let output: OutputOptions = context.get_output().clone();

    PackedFileDestination::require_free(dest, PackedFileArguments::is_forced(matches))?;

    xrf_output::info!(output, "Packing {} into {}", format_path(path), format_path(dest));

    let started_at: Instant = Instant::now();
    let file: F = F::import(path)?;
    let read_duration: Duration = started_at.elapsed();

    file.write(dest)?;

    let write_duration: Duration = started_at.elapsed() - read_duration;

    xrf_output::verbose!(output, "Read in {}", format_duration(read_duration));
    xrf_output::verbose!(output, "Written in {}", format_duration(write_duration));
    xrf_output::success!(
      output,
      "Packed {} in {}",
      format_path(dest),
      format_duration(started_at.elapsed())
    );

    context.set_result(|| FileConversionReport::new(path, dest, read_duration, write_duration))
  }
}

use std::marker::PhantomData;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::{ArgMatches, Command};
use xrf_output::OutputOptions;
use xrf_utils::{format_duration, format_path};

use crate::commands::packed_file::{PackedFileArguments, PackedFileFormat};
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::reports::FileConversionReport;

/// Reads one packed file and writes it packed again, which proves the round trip.
pub struct RepackCommand<F>(PhantomData<F>);

impl<F> Default for RepackCommand<F> {
  fn default() -> Self {
    Self(PhantomData)
  }
}

impl<F: PackedFileFormat> GenericCommand for RepackCommand<F> {
  fn operation(&self) -> &'static str {
    "repack"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command.about(format!("Read a {} and write it packed again", F::FILE_NAME));
    let command: Command = PackedFileArguments::<F>::declare_path(command, format!("Packed {}", F::FILE_NAME));

    PackedFileArguments::<F>::declare_dest(command, format!("Repacked {} written", F::FILE_NAME))
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = PackedFileArguments::<F>::path_of(matches);
    let dest: &PathBuf = PackedFileArguments::<F>::dest_of(matches);
    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Repacking {} into {}", format_path(path), format_path(dest));

    let started_at: Instant = Instant::now();
    let file: F = F::read(path)?;
    let read_duration: Duration = started_at.elapsed();

    file.write(dest)?;

    let write_duration: Duration = started_at.elapsed() - read_duration;

    xrf_output::verbose!(output, "Read in {}", format_duration(read_duration));
    xrf_output::verbose!(output, "Written in {}", format_duration(write_duration));
    xrf_output::success!(
      output,
      "Repacked {} in {}",
      format_path(dest),
      format_duration(started_at.elapsed())
    );

    context.set_result(|| FileConversionReport::new(path, dest, read_duration, write_duration))
  }
}

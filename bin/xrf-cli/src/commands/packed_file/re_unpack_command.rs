use std::marker::PhantomData;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::{ArgMatches, Command};
use xrf_output::OutputOptions;
use xrf_utils::{format_duration, format_path};

use crate::commands::packed_file::{PackedFileArguments, PackedFileFormat};
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::reports::FileConversionReport;

/// Reads one unpacked folder and writes it unpacked again, which proves the text round trip.
pub struct ReUnpackCommand<F>(PhantomData<F>);

impl<F> Default for ReUnpackCommand<F> {
  fn default() -> Self {
    Self(PhantomData)
  }
}

impl<F: PackedFileFormat> GenericCommand for ReUnpackCommand<F> {
  fn operation(&self) -> &'static str {
    "re-unpack"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command.about(format!(
      "Read an unpacked {} folder and write it unpacked again",
      F::NOUN
    ));
    let command: Command = PackedFileArguments::<F>::declare_path(command, format!("Unpacked {} folder", F::NOUN));

    PackedFileArguments::<F>::declare_dest(command, format!("Folder the re-unpacked {} is written to", F::NOUN))
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = PackedFileArguments::<F>::path_of(matches);
    let dest: &PathBuf = PackedFileArguments::<F>::dest_of(matches);
    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Re-unpacking {} into {}", format_path(path), format_path(dest));

    let started_at: Instant = Instant::now();
    let file: F = F::import(path)?;
    let read_duration: Duration = started_at.elapsed();

    file.export(dest)?;

    let write_duration: Duration = started_at.elapsed() - read_duration;

    xrf_output::verbose!(output, "Imported in {}", format_duration(read_duration));
    xrf_output::verbose!(output, "Exported in {}", format_duration(write_duration));
    xrf_output::success!(
      output,
      "Re-unpacked {} in {}",
      format_path(dest),
      format_duration(started_at.elapsed())
    );

    context.set_result(|| FileConversionReport::new(path, dest, read_duration, write_duration))
  }
}

use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_chunk::XRayByteOrder;
use xrf_dds::{DdsFile, DdsMetadata};
use xrf_output::OutputOptions;
use xrf_texture::TextureEncodedDescriptor;
use xrf_thm::ThmFile;
use xrf_utils::{OutputFiles, format_path};

use super::report::ThmWriteReport;
use crate::commands::thm::thm_field_arguments::ThmFieldArguments;
use crate::commands::thm::thm_field_edit::ThmFieldEdit;
use crate::core::arguments::InputArgument;
use crate::core::command::{CommandContext, CommandInput, CommandResult, GenericCommand};

#[derive(Default)]
pub(crate) struct WriteCommand;

impl GenericCommand for WriteCommand {
  fn operation(&self) -> &'static str {
    "write"
  }

  fn input(&self) -> CommandInput {
    CommandInput::path("Path to the dds file to write a descriptor for")
  }

  fn declare(&self, command: Command) -> Command {
    ThmFieldArguments::declare(
      command
        .about(
          "Write a complete texture descriptor for a DDS file, at the SDK's new-file defaults with the size and format the \
           file states, then the fields named",
        )
        .arg(
          Arg::new("dest")
            .help("Path of the thm file to write, defaults to the one beside the dds file")
            .short('d')
            .long("dest")
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(
          Arg::new("force")
            .help("Replace the thm file where it already exists; without it, one there is refused before reading")
            .short('f')
            .long("force")
            .action(ArgAction::SetTrue),
        )
        .arg(
          Arg::new("dry-run")
            .help("Report the descriptor without writing it")
            .long("dry-run")
            .action(ArgAction::SetTrue),
        ),
    )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let source: &PathBuf = InputArgument::of(matches);
    let destination: PathBuf = matches
      .get_one::<PathBuf>("dest")
      .cloned()
      .unwrap_or_else(|| source.with_extension("thm"));
    let is_dry_run: bool = matches.get_flag("dry-run");
    let edit: ThmFieldEdit = ThmFieldArguments::of(matches)?;
    let output: OutputOptions = context.get_output().clone();

    OutputFiles::of_path(&destination).assert_free(matches.get_flag("force"))?;

    let descriptor: ThmFile = Self::describe(source, &edit)?;

    if !is_dry_run {
      descriptor.write_to_path::<XRayByteOrder, _>(&destination)?;
    }

    context.set_result(|| ThmWriteReport::new(source, &destination, descriptor, is_dry_run))?;

    if is_dry_run {
      xrf_output::info!(
        output,
        "Dry run, nothing written, {} would describe {}",
        format_path(&destination),
        format_path(source)
      );
    } else {
      xrf_output::success!(
        output,
        "Wrote {} describing {}",
        format_path(&destination),
        format_path(source)
      );
    }

    Ok(())
  }
}

impl WriteCommand {
  /// The descriptor the texture at `source` is given, with the fields `edit` names set over it.
  fn describe(source: &Path, edit: &ThmFieldEdit) -> CommandResult<ThmFile> {
    let metadata: DdsMetadata = DdsFile::read_from_path(source)?.metadata();
    let mut descriptor: ThmFile = TextureEncodedDescriptor::of(&metadata, edit.format)?;

    edit.apply_to(&mut descriptor);

    Ok(descriptor)
  }
}

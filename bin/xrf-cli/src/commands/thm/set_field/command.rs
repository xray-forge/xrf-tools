use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command};
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_output::OutputOptions;
use xrf_thm::{ThmKind, ThmTextureEdit};
use xrf_utils::{StagedFile, format_path, format_portable_path};

use super::report::ThmSetFieldReport;
use crate::commands::thm::thm_field_arguments::ThmFieldArguments;
use crate::core::arguments::InputArgument;
use crate::core::command::{CommandContext, CommandInput, CommandResult, GenericCommand};
use crate::core::host_walk::HostWalk;

#[derive(Default)]
pub(crate) struct SetFieldCommand;

/// One descriptor's new bytes, held until every descriptor of the run has been read and edited.
struct ThmSetFieldChange {
  path: PathBuf,
  bytes: Vec<u8>,
}

/// What reading and editing one descriptor came to.
enum ThmSetFieldOutcome {
  Changed(ThmSetFieldChange),
  Unchanged,
  Skipped,
}

impl GenericCommand for SetFieldCommand {
  fn operation(&self) -> &'static str {
    "set-field"
  }

  fn input(&self) -> CommandInput {
    CommandInput::path("Path to a thm file, or to a directory whose texture descriptors are all edited")
  }

  fn declare(&self, command: Command) -> Command {
    ThmFieldArguments::declare(
      command
        .about(
          "Set named fields on texture descriptors in place, every other byte of each left as it was; a directory sets \
           them on every texture descriptor below it",
        )
        .arg(
          Arg::new("dry-run")
            .help("Report which descriptors would change without writing any")
            .long("dry-run")
            .action(ArgAction::SetTrue),
        ),
    )
    .mut_group(ThmFieldArguments::GROUP, |group| group.required(true))
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let root: &PathBuf = InputArgument::of(matches);
    let edit: ThmTextureEdit = ThmFieldArguments::of(matches)?;
    let is_dry_run: bool = matches.get_flag("dry-run");
    let output: OutputOptions = context.get_output().clone();
    let mut report: ThmSetFieldReport = ThmSetFieldReport {
      changed: Vec::new(),
      is_dry_run,
      skipped: 0,
      unchanged: 0,
    };
    let mut changes: Vec<ThmSetFieldChange> = Vec::new();

    // Every descriptor is read and edited before any is written, so one the run cannot read leaves all of them as
    // they were.
    for path in HostWalk::files_with(root, XrayExtension::Thm) {
      let path: PathBuf = path?;

      match Self::edit(&path, &edit)? {
        ThmSetFieldOutcome::Changed(change) => {
          report.changed.push(format_portable_path(&change.path));
          changes.push(change);
        }
        ThmSetFieldOutcome::Unchanged => report.unchanged += 1,
        // A directory holds thumbnails of every kind, but a file named on its own was meant to be a descriptor.
        ThmSetFieldOutcome::Skipped if root.is_file() => {
          return Err(
            XrfError::new_invalid_error(format!("{} is not a texture descriptor", format_path(&path))).into(),
          );
        }
        ThmSetFieldOutcome::Skipped => report.skipped += 1,
      }
    }

    if !is_dry_run {
      for change in &changes {
        StagedFile::write(&change.path, &change.bytes)?;
      }
    }

    context.set_result(|| &report)?;

    if is_dry_run {
      xrf_output::info!(
        output,
        "Dry run, nothing written; descriptors that would change: {}, already as named: {}, skipped: {}",
        report.changed.len(),
        report.unchanged,
        report.skipped
      );
    } else {
      xrf_output::success!(
        output,
        "Set fields; descriptors changed: {}, already as named: {}, skipped: {}",
        report.changed.len(),
        report.unchanged,
        report.skipped
      );
    }

    Ok(())
  }
}

impl SetFieldCommand {
  /// Reads the descriptor at `path` and sets the fields `edit` names on it, skipping a thumbnail that is not a texture
  /// descriptor and refusing one [`ThmTextureEdit::rewrite`] refuses.
  fn edit(path: &Path, edit: &ThmTextureEdit) -> XrfResult<ThmSetFieldOutcome> {
    let original: Vec<u8> = std::fs::read(path)?;

    if ThmKind::detect(original.clone())? != Some(ThmKind::Texture) {
      return Ok(ThmSetFieldOutcome::Skipped);
    }

    let bytes: Vec<u8> = edit.rewrite(&original)?.ok_or_else(|| {
      XrfError::new_verify_error(format!(
        "{} would not be written back byte for byte, so its other fields could not be kept as they are",
        format_path(path)
      ))
    })?;

    if bytes == original {
      return Ok(ThmSetFieldOutcome::Unchanged);
    }

    Ok(ThmSetFieldOutcome::Changed(ThmSetFieldChange {
      path: path.to_path_buf(),
      bytes,
    }))
  }
}

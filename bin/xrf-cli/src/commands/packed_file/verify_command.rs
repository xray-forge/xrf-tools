use std::marker::PhantomData;
use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgAction, ArgMatches, Command};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};
use xrf_utils::{format_path, to_portable_path_string};

use crate::commands::packed_file::{PackedFileArguments, PackedFileFormat, PackedFileVerifyReport};
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::report::ReportVerdict;

/// Reads one packed file, or its unpacked folder, and judges whether it reads.
pub struct VerifyCommand<F>(PhantomData<F>);

impl<F> Default for VerifyCommand<F> {
  fn default() -> Self {
    Self(PhantomData)
  }
}

impl<F> VerifyCommand<F> {
  const UNPACKED: &'static str = "unpacked";
}

impl<F: PackedFileFormat> GenericCommand for VerifyCommand<F> {
  fn operation(&self) -> &'static str {
    "verify"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command.about(format!("Verify that a {} reads", F::FILE_NAME));

    PackedFileArguments::declare_path(
      command,
      format!("Packed {}, or its unpacked folder with --unpacked", F::FILE_NAME),
    )
    .arg(
      Arg::new(Self::UNPACKED)
        .help(format!("Read the path as an unpacked {} folder", F::NOUN))
        .short('u')
        .long(Self::UNPACKED)
        .action(ArgAction::SetTrue),
    )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = PackedFileArguments::path_of(matches);
    let is_unpacked: bool = matches.get_flag(Self::UNPACKED);
    let output: OutputOptions = context.get_output().clone();
    let started_at: Instant = Instant::now();

    xrf_output::info!(output, "Verifying {} {}", F::NOUN, format_path(path));

    let read: XrfResult<F> = if is_unpacked { F::import(path) } else { F::read(path) };
    let findings: Vec<Finding> = match read {
      Ok(_) => Vec::new(),
      // An unreadable file is an execution failure, which judged nothing and so deposits nothing; only judged content
      // is a check failure.
      Err(error @ XrfError::Io { .. }) => return Err(error.into()),
      Err(error) => vec![Finding::new(
        RuleId::of_static(F::READ_CHECK),
        Some(to_portable_path_string(path)),
        error.to_string(),
      )],
    };
    let report: Report = Report::new(vec![CheckReport::new(
      CheckId::of_static(F::READ_CHECK),
      Status::from_is_valid(findings.is_empty()),
      Some(started_at.elapsed()),
      findings,
    )]);

    ReportVerdict::print_findings(&output, &report);

    // Deposited before the verdict becomes an outcome, so a caller reading `result` alone still learns what was wrong.
    context.set_result(|| PackedFileVerifyReport::new(path, report.clone()))?;

    ReportVerdict::conclude(&output, &report, &format!("Verification of {}", format_path(path)))
  }
}

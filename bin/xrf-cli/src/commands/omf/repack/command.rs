use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_omf::OmfFile;
use xrf_output::OutputOptions;
use xrf_report::Status;
use xrf_spawn::XRayByteOrder;
use xrf_utils::{format_duration, format_path};

use crate::commands::omf::repack::{
  OmfRepackFindingReport, OmfRepackOutcome, OmfRepackStatistics, OmfRepackVerifyReport,
};
use crate::core::command::{CommandContext, CommandError, CommandResult, GenericCommand};
use crate::core::host_walk::HostWalk;
use crate::core::reports::FileConversionReport;

#[derive(Default)]
pub struct RepackCommand;

impl GenericCommand for RepackCommand {
  fn operation(&self) -> &'static str {
    "repack"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about("Repack an OMF file, or verify that OMF files repack into the bytes they were read from")
      .arg(
        Arg::new("path")
          .help("Omf file to repack, or with --verify an omf file or a directory of them")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Repacked omf file written")
          .short('d')
          .long("dest")
          .required_unless_present("verify")
          .conflicts_with("verify")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("verify")
          .help("Verify that repacked bytes match the source instead of writing them")
          .long("verify")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    if matches.get_flag("verify") {
      return Self::verify(context, path);
    }

    if path.is_dir() {
      return Err(XrfError::new_invalid_error("A directory of omf files is only verified: pass --verify").into());
    }

    Self::repack_file(
      context,
      path,
      matches
        .get_one::<PathBuf>("dest")
        .expect("Expected a destination outside --verify"),
    )
  }
}

impl RepackCommand {
  /// Verifies that every omf file `path` names, itself or below it, serializes back into identical bytes.
  fn verify(context: &mut CommandContext, path: &Path) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();
    let mut statistics: OmfRepackStatistics = OmfRepackStatistics::default();
    let mut findings: Vec<OmfRepackFindingReport> = Vec::new();

    for file in HostWalk::files_with(path, XrayExtension::Omf) {
      let outcome: OmfRepackOutcome = match file {
        Ok(file) => Self::verify_single(&output, &file, &mut findings),
        // A directory the walk cannot read is a failure of the run: its files were never judged.
        Err(error) => {
          xrf_output::error!(output, "{error}");
          findings.push(OmfRepackFindingReport::new(path, error.to_string()));

          OmfRepackOutcome::Failed
        }
      };

      statistics.register(outcome);
    }

    xrf_output::info!(
      output,
      "Repacked omf files, {}/{} byte identical, {} malformed, {} failed to read",
      statistics.identical(),
      statistics.checked(),
      statistics.malformed(),
      statistics.failed()
    );

    // Deposited before the verdict becomes an outcome, so a failing sweep still reports the files that explain it.
    context.set_result(|| OmfRepackVerifyReport::new(&statistics, findings))?;

    // A file that could not be read, or no file at all, is the run failing; a file that would not parse or came back
    // different is a verdict on content.
    match statistics.status() {
      Status::Skipped => {
        Err(XrfError::new_verify_error(format!("No omf file found under {}", format_path(path))).into())
      }
      Status::Error => Err(
        XrfError::new_verify_error(format!(
          "Omf repack verification could not read {} of {} files",
          statistics.failed(),
          statistics.checked()
        ))
        .into(),
      ),
      Status::Failed => Err(CommandError::new_check_failed(statistics.judged_defective() as usize)),
      Status::Passed | Status::Incomplete => {
        xrf_output::success!(output, "Every omf file repacked into identical bytes");

        Ok(())
      }
    }
  }

  /// Reads `path` and writes it repacked to `destination`.
  fn repack_file(context: &mut CommandContext, path: &Path, destination: &Path) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(
      output,
      "Repacking {} into {}",
      format_path(path),
      format_path(destination)
    );

    let started_at: Instant = Instant::now();
    let omf_file: OmfFile = OmfFile::read_from_path::<XRayByteOrder, _>(&path)?;
    let read_duration: Duration = started_at.elapsed();

    omf_file.write_to_path::<XRayByteOrder, _>(&destination)?;

    let write_duration: Duration = started_at.elapsed() - read_duration;

    xrf_output::success!(
      output,
      "Repacked {} in {}",
      format_path(destination),
      format_duration(started_at.elapsed())
    );

    context.set_result(|| FileConversionReport::new(path, destination, read_duration, write_duration))
  }

  /// Reads and re-serializes one omf file, comparing the result with its bytes.
  fn verify_single(
    output: &OutputOptions,
    path: &Path,
    findings: &mut Vec<OmfRepackFindingReport>,
  ) -> OmfRepackOutcome {
    match Self::repack_into_buffer(path) {
      Ok((original, repacked)) if original == repacked => {
        xrf_output::verbose!(output, "Byte identical: {}", format_path(path));

        OmfRepackOutcome::Identical
      }
      Ok((original, repacked)) => {
        let message: String = format!(
          "Repacked bytes differ, {} bytes original, {} bytes repacked",
          original.len(),
          repacked.len()
        );

        xrf_output::failure!(output, "{}: {message}", format_path(path));
        findings.push(OmfRepackFindingReport::new(path, message));

        OmfRepackOutcome::Mismatched
      }
      // A file that could not be read is the run's own failure; one read and refused is judged content.
      Err(error @ XrfError::Io { .. }) => {
        xrf_output::error!(output, "Failed to read {}: {error}", format_path(path));
        findings.push(OmfRepackFindingReport::new(path, error.to_string()));

        OmfRepackOutcome::Failed
      }
      Err(error) => {
        xrf_output::failure!(output, "{}: {error}", format_path(path));
        findings.push(OmfRepackFindingReport::new(path, error.to_string()));

        OmfRepackOutcome::Malformed
      }
    }
  }

  /// One omf file's bytes, and the same file read from them and written back.
  fn repack_into_buffer(path: &Path) -> XrfResult<(Vec<u8>, Vec<u8>)> {
    let original: Vec<u8> = fs::read(path)?;
    let omf_file: OmfFile = OmfFile::read_from_bytes::<XRayByteOrder>(original.clone())?;
    let mut repacked: Vec<u8> = Vec::with_capacity(original.len());

    omf_file.write_to::<XRayByteOrder>(&mut repacked)?;

    Ok((original, repacked))
  }
}

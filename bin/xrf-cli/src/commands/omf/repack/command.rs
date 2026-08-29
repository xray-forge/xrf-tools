use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use walkdir::WalkDir;
use xrf_db::{OmfFile, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use super::report::{OmfRepackFindingReport, OmfRepackVerifyReport};
use super::statistics::{RepackOmfOutcome, RepackOmfStatistics};
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::reports::FileConversionReport;

#[derive(Default)]
pub struct RepackCommand;

impl GenericCommand for RepackCommand {
  fn operation(&self) -> &'static str {
    "repack"
  }

  /// Create command for repack of omf file.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to repack provided omf file or directory of omf files")
      .arg(
        Arg::new("path")
          .help("Path to omf file or directory containing omf files")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to resulting omf file, not applicable when verifying a directory")
          .short('d')
          .long("dest")
          .required(false)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("verify")
          .help("Verify that repacked bytes match the source file instead of writing output")
          .long("verify")
          .action(ArgAction::SetTrue),
      )
  }

  /// Repack provided omf file or verify omf files in provided directory.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let destination: Option<&PathBuf> = matches.get_one::<PathBuf>("dest");

    if path.is_dir() {
      Self::verify_directory(context, path, destination)
    } else if matches.get_flag("verify") {
      Self::verify_file(context, path)
    } else {
      Self::repack_file(context, path, destination)
    }
  }
}

impl RepackCommand {
  /// Verify that every omf file in provided directory is serialized back into identical bytes.
  fn verify_directory(context: &mut CommandContext, path: &Path, destination: Option<&PathBuf>) -> CommandResult {
    if destination.is_some() {
      return Err(XrfError::new_invalid_error("Destination path is not applicable when repacking a directory").into());
    }

    let output: OutputOptions = context.get_output().clone();

    let mut statistics: RepackOmfStatistics = RepackOmfStatistics::default();
    let mut findings: Vec<OmfRepackFindingReport> = Vec::new();

    for path in Self::list_omf_files(path) {
      statistics.register(Self::verify_single(&output, &path, &mut findings));
    }

    xrf_output::info!(
      output,
      "Repacked omf files, {}/{} byte identical, {} failed to read or write",
      statistics.identical(),
      statistics.checked(),
      statistics.failed()
    );

    // Deposited before the verdict becomes an outcome, so a failing sweep still reports the files
    // that explain it.
    context.set_result(|| OmfRepackVerifyReport::new(&statistics, findings))?;

    if !statistics.is_valid() {
      return Err(
        XrfError::new_verify_error(format!(
          "Omf repack verification failed, {} mismatched and {} errored of {} files",
          statistics.mismatched(),
          statistics.failed(),
          statistics.checked()
        ))
        .into(),
      );
    }

    Ok(())
  }

  /// Verify that provided omf file is serialized back into identical bytes.
  fn verify_file(context: &mut CommandContext, path: &Path) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();

    let mut statistics: RepackOmfStatistics = RepackOmfStatistics::default();
    let mut findings: Vec<OmfRepackFindingReport> = Vec::new();

    statistics.register(Self::verify_single(&output, path, &mut findings));

    // The census of one: a caller pointing at a file reads the same payload as one pointing at a
    // directory, and reads it whether the verdict passed or failed.
    context.set_result(|| OmfRepackVerifyReport::new(&statistics, findings))?;

    if statistics.is_valid() {
      return Ok(());
    }

    Err(XrfError::new_verify_error(format!("Omf repack verification failed for {}", format_path(path))).into())
  }

  /// Read provided omf file and write it into destination path.
  fn repack_file(context: &mut CommandContext, path: &Path, destination: Option<&PathBuf>) -> CommandResult {
    let destination: &PathBuf =
      destination.ok_or_else(|| XrfError::new_invalid_error("Destination path is required when not verifying"))?;

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Repack omf file {}", format_path(path));

    let started_at: Instant = Instant::now();
    let omf_file: Box<OmfFile> = Box::new(OmfFile::read_from_path::<XRayByteOrder, _>(&path)?);
    let read_duration: Duration = started_at.elapsed();

    omf_file.write_to_path::<XRayByteOrder, _>(destination)?;

    let write_duration: Duration = started_at.elapsed() - read_duration;

    xrf_output::info!(output, "Omf file repacked into {}", format_path(destination));

    context.set_result(|| FileConversionReport::new(path, destination, read_duration, write_duration))?;

    Ok(())
  }

  /// Collect omf files stored in provided directory and its nested directories.
  fn list_omf_files(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
      .into_iter()
      .filter_map(Result::ok)
      .map(|it| it.into_path())
      .filter(|it| it.is_file() && it.extension().and_then(|it| it.to_str()) == Some("omf"))
      .collect()
  }

  /// Read and re-serialize single omf file, comparing the result with source bytes.
  ///
  /// A file that did not come back is appended to `findings` as well as counted, so the report says
  /// which file failed and how, not only how many did.
  fn verify_single(
    output: &OutputOptions,
    path: &Path,
    findings: &mut Vec<OmfRepackFindingReport>,
  ) -> RepackOmfOutcome {
    match Self::repack_into_buffer(path) {
      Ok((original, repacked)) => {
        if original == repacked {
          xrf_output::verbose!(output, "Byte identical: {}", format_path(path));

          RepackOmfOutcome::Identical
        } else {
          xrf_output::error!(
            output,
            "Repacked bytes differ: {}, {} bytes original, {} bytes repacked",
            format_path(path),
            original.len(),
            repacked.len()
          );

          findings.push(OmfRepackFindingReport::new(
            path,
            format!(
              "Repacked bytes differ, {} bytes original, {} bytes repacked",
              original.len(),
              repacked.len()
            ),
          ));

          RepackOmfOutcome::Mismatched
        }
      }
      Err(error) => {
        xrf_output::error!(output, "Failed to repack {}: {}", format_path(path), error);

        findings.push(OmfRepackFindingReport::new(path, error.to_string()));

        RepackOmfOutcome::Failed
      }
    }
  }

  /// Read omf file and write it back into memory buffer.
  fn repack_into_buffer(path: &Path) -> XrfResult<(Vec<u8>, Vec<u8>)> {
    let original: Vec<u8> = fs::read(path)?;
    let omf_file: Box<OmfFile> = Box::new(OmfFile::read_from_path::<XRayByteOrder, _>(&path)?);

    let mut repacked: Vec<u8> = Vec::with_capacity(original.len());

    omf_file.write_to::<XRayByteOrder>(&mut repacked)?;

    Ok((original, repacked))
  }
}

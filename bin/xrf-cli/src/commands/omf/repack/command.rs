use std::fs;
use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use walkdir::WalkDir;
use xrf_db::{OmfFile, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;

use super::statistics::{RepackOmfOutcome, RepackOmfStatistics};
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

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
      .arg(
        Arg::new("silent")
          .help("Disable any logging")
          .short('s')
          .long("silent")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("verbose")
          .help("Turn on verbose logging")
          .short('v')
          .long("verbose")
          .action(ArgAction::SetTrue),
      )
  }

  /// Repack provided omf file or verify omf files in provided directory.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let destination: Option<&PathBuf> = matches.get_one::<PathBuf>("dest");

    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    if path.is_dir() {
      Self::verify_directory(&output, path, destination)?;
    } else if matches.get_flag("verify") {
      Self::verify_file(&output, path)?;
    } else {
      Self::repack_file(&output, path, destination)?;
    }

    Ok(())
  }
}

impl RepackCommand {
  /// Verify that every omf file in provided directory is serialized back into identical bytes.
  fn verify_directory(output: &OutputOptions, path: &Path, destination: Option<&PathBuf>) -> XrfResult {
    if destination.is_some() {
      return Err(XrfError::new_invalid_error(
        "Destination path is not applicable when repacking a directory",
      ));
    }

    let mut statistics: RepackOmfStatistics = RepackOmfStatistics::default();

    for path in Self::list_omf_files(path) {
      statistics.register(Self::verify_single(output, &path));
    }

    xrf_output::info!(
      output,
      "Repacked omf files, {}/{} byte identical, {} failed to read or write",
      statistics.identical(),
      statistics.checked(),
      statistics.failed()
    );

    if !statistics.is_valid() {
      return Err(XrfError::new_verify_error(format!(
        "Omf repack verification failed, {} mismatched and {} errored of {} files",
        statistics.mismatched(),
        statistics.failed(),
        statistics.checked()
      )));
    }

    Ok(())
  }

  /// Verify that provided omf file is serialized back into identical bytes.
  fn verify_file(output: &OutputOptions, path: &Path) -> XrfResult {
    if Self::verify_single(output, path) == RepackOmfOutcome::Identical {
      return Ok(());
    }

    Err(XrfError::new_verify_error(format!(
      "Omf repack verification failed for {}",
      path.display()
    )))
  }

  /// Read provided omf file and write it into destination path.
  fn repack_file(output: &OutputOptions, path: &Path, destination: Option<&PathBuf>) -> XrfResult {
    let destination: &PathBuf =
      destination.ok_or_else(|| XrfError::new_invalid_error("Destination path is required when not verifying"))?;

    xrf_output::info!(output, "Repack omf file {}", path.display());

    OmfFile::read_from_path::<XRayByteOrder, _>(&path)?.write_to_path::<XRayByteOrder, _>(destination)?;

    xrf_output::info!(output, "Omf file repacked into {}", destination.display());

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
  fn verify_single(output: &OutputOptions, path: &Path) -> RepackOmfOutcome {
    match Self::repack_into_buffer(path) {
      Ok((original, repacked)) => {
        if original == repacked {
          xrf_output::verbose!(output, "Byte identical: {}", path.display());

          RepackOmfOutcome::Identical
        } else {
          xrf_output::error!(
            output,
            "Repacked bytes differ: {}, {} bytes original, {} bytes repacked",
            path.display(),
            original.len(),
            repacked.len()
          );

          RepackOmfOutcome::Mismatched
        }
      }
      Err(error) => {
        xrf_output::error!(output, "Failed to repack {}: {}", path.display(), error);

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

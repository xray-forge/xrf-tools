use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_db::{OmfFile, OmfMotionsProcessor, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;

use super::report::OmfRenameMotionsReport;
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct RenameMotionsCommand;

impl GenericCommand for RenameMotionsCommand {
  fn operation(&self) -> &'static str {
    "rename-motions"
  }

  /// Create command for renaming omf motions.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to rename motions of provided omf file")
      .arg(
        Arg::new("path")
          .help("Path to omf file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to resulting omf file")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("map")
          .help("Path to JSON object mapping existing motion names to new ones")
          .short('m')
          .long("map")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("strict")
          .help("Require every motion in the file to be covered by the map")
          .long("strict")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("dry-run")
          .help("Validate the change and report the result without writing any file")
          .long("dry-run")
          .action(ArgAction::SetTrue),
      )
  }

  /// Rename motions of provided omf file.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let map_path: &PathBuf = matches
      .get_one::<PathBuf>("map")
      .expect("Expected valid map path to be provided");

    let destination: &PathBuf = matches
      .get_one::<PathBuf>("dest")
      .expect("Expected valid output path to be provided");

    Self::rename_file(
      context,
      path,
      destination,
      map_path,
      matches.get_flag("strict"),
      matches.get_flag("dry-run"),
    )?;

    Ok(())
  }
}

impl RenameMotionsCommand {
  /// Rename motions of single omf file and write the result into destination.
  fn rename_file(
    context: &mut CommandContext,
    path: &Path,
    destination: &Path,
    map_path: &Path,
    is_strict: bool,
    is_dry_run: bool,
  ) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();

    let renames: HashMap<String, String> = Self::read_map(map_path)?;
    let mut omf_file: Box<OmfFile> = Box::new(OmfFile::read_from_path::<XRayByteOrder, _>(&path)?);

    if is_strict && let Err(error) = Self::assert_map_covers_all_motions(path, &omf_file, &renames) {
      // The guard refuses before anything is renamed, but the names the file holds are exactly what
      // a caller reconciles its map against, so they are worth depositing on the way out.
      context.set_result(|| OmfRenameMotionsReport::new(path, destination, &omf_file, 0, is_dry_run))?;

      return Err(error.into());
    }

    let renamed_count: usize = OmfMotionsProcessor::rename_motions(&mut omf_file, &renames)?;

    // Deposited before the refusal below and before the dry run returns, so a run that wrote
    // nothing still reports the names the map produced.
    context.set_result(|| OmfRenameMotionsReport::new(path, destination, &omf_file, renamed_count, is_dry_run))?;

    if renamed_count == 0 {
      return Err(
        XrfError::new_invalid_error(format!(
          "Refused to rename {}, no motion matched the provided map",
          path.display()
        ))
        .into(),
      );
    }

    xrf_output::info!(
      output,
      "Renamed omf motions {}, {renamed_count}/{} renamed",
      path.display(),
      omf_file.motions.motions.len()
    );

    xrf_output::verbose!(output, "Resulting motions: {}", omf_file.get_motion_names().join(","));

    if is_dry_run {
      xrf_output::info!(
        output,
        "Dry run, nothing written, {} would receive {renamed_count} renamed motions",
        destination.display()
      );

      return Ok(());
    }

    omf_file.write_to_path::<XRayByteOrder, _>(&destination)?;

    xrf_output::info!(output, "Renamed omf file written into {}", destination.display());

    Ok(())
  }

  /// Read the old name to new name map from provided JSON file.
  fn read_map(map_path: &Path) -> XrfResult<HashMap<String, String>> {
    let content: String = fs::read_to_string(map_path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Motions rename map was not read: {}, error: {error}",
        map_path.display()
      ))
    })?;

    serde_json::from_str(&content).map_err(|error| {
      XrfError::new_parsing_error(format!(
        "Motions rename map is not a valid JSON object of string to string: {}, error: {error}",
        map_path.display()
      ))
    })
  }

  /// Guard that every motion in the file has an entry in the map.
  fn assert_map_covers_all_motions(path: &Path, omf_file: &OmfFile, renames: &HashMap<String, String>) -> XrfResult {
    let uncovered: Vec<&str> = omf_file
      .get_motion_names()
      .into_iter()
      .filter(|it| !renames.contains_key(*it))
      .collect();

    if !uncovered.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "Refused to rename {} in strict mode, {} motions are missing from the map: {}",
        path.display(),
        uncovered.len(),
        uncovered.join(",")
      )));
    }

    Ok(())
  }
}

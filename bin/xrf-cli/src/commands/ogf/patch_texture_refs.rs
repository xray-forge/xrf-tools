use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_db::{OgfFile, OgfRefsPatchReport, OgfTextureRefsProcessor, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_output::OutputOptions;

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct PatchTextureRefsCommand;

impl GenericCommand for PatchTextureRefsCommand {
  fn operation(&self) -> &'static str {
    "patch-texture-refs"
  }

  /// Create command for renaming ogf texture refs.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to rename a texture reference of provided ogf file")
      .arg(
        Arg::new("path")
          .help("Path to ogf file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to resulting ogf file, defaults to in place rewrite of the source file")
          .short('d')
          .long("dest")
          .required(false)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("from")
          .help("Texture reference to rename, matched exactly")
          .long("from")
          .required(true),
      )
      .arg(
        Arg::new("to")
          .help("Texture reference to write in its place")
          .long("to")
          .required(true),
      )
      .arg(
        Arg::new("dry-run")
          .help("Validate the change and report the result without writing any file")
          .long("dry-run")
          .action(ArgAction::SetTrue),
      )
  }

  /// Rename a texture reference of provided ogf file.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let from: &String = matches
      .get_one::<String>("from")
      .expect("Expected valid source texture reference to be provided");
    let to: &String = matches
      .get_one::<String>("to")
      .expect("Expected valid target texture reference to be provided");

    let destination: &Path = matches
      .get_one::<PathBuf>("dest")
      .map_or(path.as_path(), |it| it.as_path());

    let output: OutputOptions = context.get_output().clone();

    Self::patch_file(&output, path, destination, from, to, matches.get_flag("dry-run"))?;

    Ok(())
  }
}

impl PatchTextureRefsCommand {
  /// Rename a texture reference of single ogf file, preserving all other chunks byte for byte.
  ///
  /// The rename itself and every guard around it live in [`OgfTextureRefsProcessor`], so this only
  /// reports what it did.
  fn patch_file(
    output: &OutputOptions,
    path: &Path,
    destination: &Path,
    from: &str,
    to: &str,
    is_dry_run: bool,
  ) -> XrfResult {
    let existing: Vec<String> = OgfFile::read_texture_refs_from_path::<XRayByteOrder, _>(&path)?;

    xrf_output::info!(
      output,
      "Patch ogf texture refs {}, '{}' -> '{}', existing {:?}",
      path.display(),
      from,
      to,
      existing
    );

    let report: OgfRefsPatchReport =
      OgfTextureRefsProcessor::patch_texture_refs_to_path::<XRayByteOrder>(path, destination, from, to, is_dry_run)?;

    if report.is_dry_run {
      xrf_output::info!(
        output,
        "Dry run, nothing written, {} references would be renamed and {} would receive {} bytes instead of {}",
        report.patched_count,
        destination.display(),
        report.patched_size,
        report.original_size
      );

      return Ok(());
    }

    xrf_output::info!(
      output,
      "Renamed {} ogf texture references, written into {}",
      report.patched_count,
      destination.display()
    );

    Ok(())
  }
}

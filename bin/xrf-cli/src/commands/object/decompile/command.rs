use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_build_info::{BuildInfo, build_info};
use xrf_chunk::XRayByteOrder;
use xrf_editable_object::{EditableMesh, EditableSurface};
use xrf_error::{XrfError, XrfResult};
use xrf_ogf::OgfFile;
use xrf_output::OutputOptions;
use xrf_rawdata::{ObjectDecompile, ObjectDecompiler, ObjectProvenance};
use xrf_utils::{StagedFile, format_path};

use crate::commands::object::decompile::report::ObjectDecompileReport;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

/// Decompiles an OGF into the SDK editable object it was exported from, with a provenance sidecar beside it.
#[derive(Default)]
pub struct DecompileCommand;

impl GenericCommand for DecompileCommand {
  fn operation(&self) -> &'static str {
    "decompile"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about("Decompile a skeleton OGF into an SDK .object, with a .object.xrf.json provenance sidecar beside it")
      .arg(
        Arg::new("path")
          .help("Path to the OGF to decompile")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path of the .object to write; its sidecar is written beside it")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("force")
          .help("Overwrite an existing .object and sidecar")
          .short('f')
          .long("force")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let source: &PathBuf = matches.get_one::<PathBuf>("path").expect("the path is required");
    let destination: &PathBuf = matches.get_one::<PathBuf>("dest").expect("the destination is required");
    let output: OutputOptions = context.get_output().clone();

    let report: ObjectDecompileReport = Self::decompile(&output, source, destination, matches.get_flag("force"))?;

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl DecompileCommand {
  fn decompile(
    output: &OutputOptions,
    source: &Path,
    destination: &Path,
    is_forced: bool,
  ) -> XrfResult<ObjectDecompileReport> {
    let sidecar: PathBuf = ObjectProvenance::sidecar_path(destination);

    if !is_forced && let Some(existing) = [destination, sidecar.as_path()].into_iter().find(|path| path.exists()) {
      return Err(XrfError::new_invalid_error(format!(
        "{} already exists; pass --force to overwrite it",
        format_path(existing)
      )));
    }

    let bytes: Vec<u8> = std::fs::read(source)
      .map_err(|error| XrfError::new_read_error(format!("{} was not read: {error}", format_path(source))))?;
    let decompile: ObjectDecompile =
      ObjectDecompiler::decompile(&OgfFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?)?;
    let build: BuildInfo = build_info!();
    let provenance: ObjectProvenance = ObjectProvenance::new(
      format_path(source).to_string(),
      &bytes,
      "xrf-cli object decompile",
      match build.short_commit() {
        Some(commit) => format!("{} ({commit})", build.version),
        None => build.version.to_string(),
      },
      decompile.inferred.clone(),
    );

    // The sidecar first: an object that failed to publish leaves a stray sidecar, never a decompile taken for an
    // authored file.
    StagedFile::write_creating_directories(&sidecar, provenance.to_json()?.as_bytes()).map_err(|error| {
      XrfError::new_io_error(
        format!("{} was not written: {error}", format_path(&sidecar)),
        error.kind(),
      )
    })?;
    decompile.object.write_to_path::<XRayByteOrder, _>(&destination)?;

    let mesh: Option<&EditableMesh> = decompile.object.meshes.iter().flatten().next();
    let surfaces: &[EditableSurface] = decompile.object.surfaces.as_deref().unwrap_or_default();
    let report: ObjectDecompileReport = ObjectDecompileReport {
      source: format_path(source).to_string(),
      destination: format_path(destination).to_string(),
      sidecar: format_path(&sidecar).to_string(),
      surfaces: surfaces.len(),
      points: mesh.map_or(0, |mesh| mesh.points.len()),
      faces: mesh.map_or(0, |mesh| mesh.faces.len()),
      two_sided_faces: mesh.map_or(0, |mesh| {
        mesh
          .surface_faces
          .iter()
          .filter(|faces| surfaces.iter().any(|it| it.name == faces.surface && it.is_two_sided()))
          .map(|faces| faces.faces.len())
          .sum()
      }),
      bones: decompile.object.bones.as_ref().map_or(0, Vec::len),
      inferred: decompile.inferred,
    };

    xrf_output::success!(
      output,
      "Decompiled {} into {}: {} surfaces, {} points, {} faces ({} two-sided), {} bones; provenance in {}",
      report.source,
      report.destination,
      report.surfaces,
      report.points,
      report.faces,
      report.two_sided_faces,
      report.bones,
      report.sidecar
    );

    Ok(report)
  }
}

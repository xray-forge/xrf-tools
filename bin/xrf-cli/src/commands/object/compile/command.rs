use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_build_info::{BuildInfo, build_info};
use xrf_chunk::XRayByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_output::OutputOptions;
use xrf_rawdata::{
  DetailParameters, ObjectCompile, ObjectCompileOptions, ObjectCompileSource, ObjectCompiler, ObjectProvenance,
  TangentMethod,
};
use xrf_utils::{StagedFile, format_path};

use crate::commands::object::compile::report::ObjectCompileReport;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

/// Compiles an SDK editable object into the OGF the engine loads, as the Actor Editor's export does, or into a detail
/// model as the Level Editor's `EDetail` exports one.
#[derive(Default)]
pub struct CompileCommand;

impl GenericCommand for CompileCommand {
  fn operation(&self) -> &'static str {
    "compile"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about(
        "Compile an SDK .object into the OGF the engine loads (a skeleton with bones, else a static visual), or into a \
         .dm detail model where the destination is one",
      )
      .arg(
        Arg::new("path")
          .help("Path to the .object to compile")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help(
            "Path of the OGF to write, or of a .dm, whose flags and scale range come from the sidecar of an object \
             decompiled from one and are the editor's defaults otherwise",
          )
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("force")
          .help("Overwrite an existing OGF")
          .short('f')
          .long("force")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("tangents")
          .help(
            "Tangent method of a skeleton: auto follows the provenance sidecar beside a decompiled object \
             (MikkTSpace for a model exported from Blender) and the Actor Editor's otherwise; actor-editor or \
             mikktspace forces one. A static visual stores no tangents",
          )
          .long("tangents")
          .value_parser([Self::AUTO, Self::ACTOR_EDITOR, Self::MIKKTSPACE])
          .default_value(Self::AUTO),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let source: &PathBuf = matches.get_one::<PathBuf>("path").expect("the path is required");
    let destination: &PathBuf = matches.get_one::<PathBuf>("dest").expect("the destination is required");
    let tangents: &str = matches
      .get_one::<String>("tangents")
      .map(String::as_str)
      .expect("the tangent method has a default");
    let output: OutputOptions = context.get_output().clone();

    let report: ObjectCompileReport = Self::compile(&output, source, destination, matches.get_flag("force"), tangents)?;

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl CompileCommand {
  const AUTO: &'static str = "auto";
  const ACTOR_EDITOR: &'static str = "actor-editor";
  const MIKKTSPACE: &'static str = "mikktspace";

  /// The options `tangents` asks for: `auto` follows the object's provenance sidecar, if it has one.
  pub(super) fn get_options(source: &Path, tangents: &str) -> XrfResult<ObjectCompileOptions> {
    Ok(match tangents {
      Self::ACTOR_EDITOR => ObjectCompileOptions {
        tangents: TangentMethod::ActorEditor,
      },
      Self::MIKKTSPACE => ObjectCompileOptions {
        tangents: TangentMethod::MikkTSpace,
      },
      // `auto`, the default and the one value the parser leaves.
      _ => ObjectCompileOptions::following(ObjectProvenance::read_beside(source)?.as_ref()),
    })
  }

  fn compile(
    output: &OutputOptions,
    source: &Path,
    destination: &Path,
    is_forced: bool,
    tangents: &str,
  ) -> XrfResult<ObjectCompileReport> {
    if !is_forced && destination.exists() {
      return Err(XrfError::new_invalid_error(format!(
        "{} already exists; pass --force to overwrite it",
        format_path(destination)
      )));
    }

    let object: EditableObjectFile = EditableObjectFile::read_from_path::<XRayByteOrder, _>(&source)?;
    // A `.dm` destination is a detail model, whose flags and scale range a decompiled object's sidecar keeps.
    let compiled: ObjectCompile = if XrayExtension::Dm.matches_path(destination) {
      let parameters: DetailParameters = ObjectProvenance::read_beside(source)?
        .and_then(|provenance| provenance.detail)
        .unwrap_or_default();

      ObjectCompiler::compile_detail(&object, &parameters)?
    } else {
      let build: BuildInfo = build_info!();

      ObjectCompiler::compile_with(
        &object,
        &ObjectCompileSource {
          source_file: format_path(source).to_string(),
          convertor: match build.short_commit() {
            Some(commit) => format!("xrf-cli object compile {} ({commit})", build.version),
            None => format!("xrf-cli object compile {}", build.version),
          },
          built_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs()),
        },
        &Self::get_options(source, tangents)?,
      )?
    };

    StagedFile::write_creating_directories(destination, &compiled.bytes).map_err(|error| {
      XrfError::new_io_error(
        format!("{} was not written: {error}", format_path(destination)),
        error.kind(),
      )
    })?;

    let report: ObjectCompileReport = ObjectCompileReport {
      source: format_path(source).to_string(),
      destination: format_path(destination).to_string(),
      children: compiled.children,
      vertices: compiled.vertices,
      faces: compiled.faces,
      dropped_faces: compiled.dropped_faces,
      bones: object.bones.as_ref().map_or(0, Vec::len),
      bytes: compiled.bytes.len(),
      tangents: compiled.tangents,
    };

    xrf_output::success!(
      output,
      "Compiled {} into {}: {} children, {} vertices, {} faces ({} dropped), {} bones, {}",
      report.source,
      report.destination,
      report.children,
      report.vertices,
      report.faces,
      report.dropped_faces,
      report.bones,
      match report.tangents {
        Some(TangentMethod::ActorEditor) => "Actor Editor tangents",
        Some(TangentMethod::MikkTSpace) => "MikkTSpace tangents",
        None => "no tangents",
      }
    );

    Ok(report)
  }
}

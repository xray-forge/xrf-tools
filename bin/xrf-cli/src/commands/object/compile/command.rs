use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_build_info::{BuildInfo, build_info};
use xrf_output::OutputOptions;
use xrf_rawdata::{ObjectCompileReport, ObjectCompileRequest, ObjectCompiler, TangentMethod};

use crate::core::arguments::InputArgument;
use crate::core::command::{CommandContext, CommandInput, CommandResult, GenericCommand};

/// Compiles an SDK editable object into the OGF the engine loads, as the Actor Editor's export does, or into a detail
/// model as the Level Editor's `EDetail` exports one.
#[derive(Default)]
pub(crate) struct CompileCommand;

impl GenericCommand for CompileCommand {
  fn operation(&self) -> &'static str {
    "compile"
  }

  fn input(&self) -> CommandInput {
    CommandInput::Path("Path to the .object to compile")
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about(
        "Compile an SDK .object into the OGF the engine loads (a skeleton with bones, else a static visual), or into a \
         .dm detail model where the destination is one",
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
    let request: ObjectCompileRequest = ObjectCompileRequest {
      source: InputArgument::of(matches).clone(),
      destination: matches
        .get_one::<PathBuf>("dest")
        .expect("the destination is required")
        .clone(),
      tangents: Self::get_tangents(
        matches
          .get_one::<String>("tangents")
          .map(String::as_str)
          .expect("the tangent method has a default"),
      ),
      is_forced: matches.get_flag("force"),
    };
    let output: OutputOptions = context.get_output().clone();
    let build: BuildInfo = build_info!();
    let report: ObjectCompileReport =
      ObjectCompiler::compile_to_path(&request, "xrf-cli object compile", &build.get_tool_version())?;

    Self::print(&output, &report);
    context.set_result(|| &report)?;

    Ok(())
  }
}

impl CompileCommand {
  const AUTO: &'static str = "auto";
  const ACTOR_EDITOR: &'static str = "actor-editor";
  const MIKKTSPACE: &'static str = "mikktspace";

  /// The tangent method `tangents` asks for; none for `auto`, which follows the object's provenance sidecar.
  fn get_tangents(tangents: &str) -> Option<TangentMethod> {
    match tangents {
      Self::ACTOR_EDITOR => Some(TangentMethod::ActorEditor),
      Self::MIKKTSPACE => Some(TangentMethod::MikkTSpace),
      // `auto`, the default and the one value the parser leaves.
      _ => None,
    }
  }

  fn print(output: &OutputOptions, report: &ObjectCompileReport) {
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
  }
}

use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_build_info::{BuildInfo, build_info};
use xrf_output::OutputOptions;
use xrf_rawdata::{ObjectDecompileReport, ObjectDecompileRequest, ObjectDecompiler};

use crate::core::command::{CommandContext, CommandResult, GenericCommand};

/// Decompiles an OGF into the SDK editable object it was exported from, with a provenance sidecar beside it.
#[derive(Default)]
pub(crate) struct DecompileCommand;

impl GenericCommand for DecompileCommand {
  fn operation(&self) -> &'static str {
    "decompile"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about(
        "Decompile a skeleton or static OGF, or a .dm detail model, into an SDK .object, with a .object.xrf.json \
         provenance sidecar beside it",
      )
      .arg(
        Arg::new("path")
          .help("Path to the OGF or .dm to decompile")
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
    let request: ObjectDecompileRequest = ObjectDecompileRequest {
      source: matches
        .get_one::<PathBuf>("path")
        .expect("the path is required")
        .clone(),
      destination: matches
        .get_one::<PathBuf>("dest")
        .expect("the destination is required")
        .clone(),
      is_forced: matches.get_flag("force"),
    };
    let output: OutputOptions = context.get_output().clone();
    let build: BuildInfo = build_info!();
    let report: ObjectDecompileReport =
      ObjectDecompiler::decompile_to_path(&request, "xrf-cli object decompile", &build.get_tool_version())?;

    Self::print(&output, &report);
    context.set_result(|| &report)?;

    Ok(())
  }
}

impl DecompileCommand {
  fn print(output: &OutputOptions, report: &ObjectDecompileReport) {
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
  }
}

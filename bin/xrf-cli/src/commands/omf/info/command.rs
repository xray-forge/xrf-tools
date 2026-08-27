use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_db::{OmfFile, XRayByteOrder};
use xrf_output::OutputOptions;

use super::report::OmfInfoReport;
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  /// Create command for printing omf file info.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to print information about provided omf file")
      .arg(
        Arg::new("path")
          .help("Path to ogf file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  /// Print information about ogf file.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid path to be provided");

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Read omf file {}", path.display());

    let omf_file: Box<OmfFile> = Box::new(OmfFile::read_from_path::<XRayByteOrder, _>(path)?);

    xrf_output::info!(output, "Omf file information");

    xrf_output::info!(output, "Version: {}", omf_file.parameters.version);

    let motion_names: Vec<&str> = omf_file.get_motion_names();

    // Names come from the motion definitions; a payload's leading label is not what the engine resolves.
    xrf_output::info!(output, "Motions: {} {}", motion_names.len(), motion_names.join(","));

    let diverging_labels_count: usize = omf_file.get_diverging_labels_count();

    // Worth saying and not worth printing: release playback ignores the label, and on the banks that have this the
    // stored bytes are not text at all.
    if diverging_labels_count > 0 {
      xrf_output::info!(
        output,
        "Payload labels differing from motion names: {} of {}",
        diverging_labels_count,
        motion_names.len()
      );
    }

    // Keyframe count and playback speed together give effective duration.
    for (definition, motion) in omf_file.get_motions() {
      xrf_output::verbose!(
        output,
        "Motion '{}': keyframes {}, flags {:#04b}, speed {}, power {}, accrue {}, falloff {}{}",
        definition.name,
        motion.count,
        definition.flags,
        definition.speed,
        definition.power,
        definition.accrue,
        definition.falloff,
        if motion.has_label_matching(&definition.name) {
          ""
        } else {
          ", payload label differs"
        }
      );
    }

    xrf_output::info!(output, "Bones total: {}", omf_file.parameters.get_bones_count());
    xrf_output::info!(
      output,
      "Parts: {}",
      omf_file
        .parameters
        .parts
        .iter()
        .map(|it| it.name.as_str())
        .collect::<Vec<_>>()
        .join(",")
    );

    for part in &omf_file.parameters.parts {
      xrf_output::info!(output, "Part '{}' bones: {}", part.name, part.get_bones().join(","));
    }

    context.set_result(|| OmfInfoReport::new(&omf_file))?;

    Ok(())
  }
}

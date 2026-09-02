use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_dds::ImageFormat;
use xrf_output::OutputOptions;
use xrf_texture::{PackDescriptionOptions, PackDescriptionProcessor};
use xrf_utils::format_path;

use super::report::SpriteDescriptionReport;
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::progress::new_logging_job;

#[derive(Default)]
pub struct PackDescriptionCommand;

impl GenericCommand for PackDescriptionCommand {
  fn operation(&self) -> &'static str {
    "pack-description"
  }

  /// Create command for packing of texture description file.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to pack the sprites a texture description xml declares")
      .arg(
        Arg::new("description")
          .help("Path to XML file describing textures")
          .long("description")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("base")
          .help("Path to base where search for described files")
          .long("base")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("output")
          .help("Path to directory where output dds files")
          .long("output")
          .required(false)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("file")
          .help("Name of a described file to pack, repeatable; packs every described file if omitted")
          .long("file")
          .required(false)
          .action(ArgAction::Append),
      )
      .arg(
        Arg::new("strict")
          .help("Turn on strict unpack mode")
          .long("strict")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Pack texture descriptions file as single dds sprite.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let description: &PathBuf = matches
      .get_one::<PathBuf>("description")
      .expect("Expected valid path to be provided for texture description file or folder");

    let base: &PathBuf = matches
      .get_one::<PathBuf>("base")
      .expect("Expected valid base path to be provided");

    let output: &PathBuf = matches.get_one::<PathBuf>("output").unwrap_or(base);

    let files: Vec<String> = matches
      .get_many::<String>("file")
      .map(|values| values.cloned().collect())
      .unwrap_or_default();

    let is_strict: bool = matches.get_flag("strict");
    let output_options: OutputOptions = context.get_output().clone();

    let options: PackDescriptionOptions = PackDescriptionOptions {
      job: new_logging_job(),
      description: description.clone(),
      base: base.clone(),
      output: output_options,
      output_path: output.clone(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files,
      is_strict,
    };

    log::info!("Packing texture descriptions from: {}", format_path(description));
    log::info!("Paths: base {}, output {}", format_path(base), format_path(output));
    log::info!("DDS format: {:?}", options.dds_compression_format);

    // The payload is what the run was pointed at, so it is deposited before the work rather than
    // after: a run that gives up part way through still says what it was doing.
    context.set_result(|| SpriteDescriptionReport::new(&options))?;

    PackDescriptionProcessor::pack_xml_descriptions(&options)?;

    Ok(())
  }
}

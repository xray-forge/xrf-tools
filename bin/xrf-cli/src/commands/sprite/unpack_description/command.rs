use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_dds::ImageFormat;
use xrf_output::OutputOptions;
use xrf_texture::{PackDescriptionOptions, UnpackDescriptionProcessor};
use xrf_utils::format_path;

use crate::commands::sprite::pack_description::report::SpriteDescriptionReport;
use crate::core::command_context::CommandContext;
use crate::core::execution::ExecutionArguments;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::progress::new_logging_job;

#[derive(Default)]
pub struct UnpackDescriptionCommand;

impl GenericCommand for UnpackDescriptionCommand {
  fn operation(&self) -> &'static str {
    "unpack-description"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to unpack the sprites a texture description xml declares")
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
          .help("Path to output folder for icons")
          .long("output")
          .required(false)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("file")
          .help("Name of a described file to unpack, repeatable; unpacks every described file if omitted")
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
      .with_jobs()
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let description: &PathBuf = matches
      .get_one::<PathBuf>("description")
      .expect("Expected valid path to be provided for texture description file or folder");

    let base: &PathBuf = matches
      .get_one::<PathBuf>("base")
      .expect("Expected valid base path to be provided");

    let output_path: &PathBuf = matches.get_one::<PathBuf>("output").unwrap_or(base);

    let files: Vec<String> = matches
      .get_many::<String>("file")
      .map(|values| values.cloned().collect())
      .unwrap_or_default();

    let is_strict: bool = matches.get_flag("strict");
    let output: OutputOptions = context.get_output().clone();

    let started_at: Instant = Instant::now();

    log::info!("Unpacking texture descriptions from: {}", format_path(description));
    log::info!("Paths: base {}, output {}", format_path(base), format_path(output_path));

    xrf_output::info!(
      output,
      "Unpacking texture descriptions: {}, from {} to {}",
      format_path(description),
      format_path(base),
      format_path(output_path)
    );

    let options: PackDescriptionOptions = PackDescriptionOptions {
      job: new_logging_job(),
      description: description.clone(),
      base: base.clone(),
      output,
      output_path: output_path.clone(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files,
      is_strict,
    };

    // The payload is what the run was pointed at, so it is deposited before the work rather than
    // after: a run that gives up part way through still says what it was doing.
    context.set_result(|| SpriteDescriptionReport::new(&options))?;

    UnpackDescriptionProcessor::unpack_xml_descriptions(options)?;

    log::info!(
      "Unpack texture descriptions took: {}",
      xrf_utils::format_duration(started_at.elapsed())
    );

    Ok(())
  }
}

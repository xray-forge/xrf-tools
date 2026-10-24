use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_dds::ImageFormat;
use xrf_output::OutputOptions;
use xrf_texture::{PackDescriptionProcessor, SpriteDescriptionOptions, SpriteDescriptionResult};
use xrf_utils::format_path;

use super::report::SpriteDescriptionReport;
use crate::core::arguments::LinkPolicyArgument;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

#[derive(Default)]
pub struct PackDescriptionCommand;

impl GenericCommand for PackDescriptionCommand {
  fn operation(&self) -> &'static str {
    "pack-description"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about("Pack the sprites a texture description xml declares into its sheets")
      .arg(
        Arg::new("description")
          .help("Path to the XML file describing the sheets")
          .long("description")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("base")
          .help("Folder the described sprites are read from")
          .long("base")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Folder the packed sheets are written to, defaults to the base folder")
          .short('d')
          .long("dest")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("file")
          .help("Name of a described file to pack, repeatable; packs every described file if omitted")
          .long("file")
          .action(ArgAction::Append),
      )
      .arg(
        Arg::new("strict")
          .help("Fail on a sprite that cannot be read instead of leaving its place empty")
          .long("strict")
          .action(ArgAction::SetTrue),
      );

    LinkPolicyArgument::declare(command)
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let description: &PathBuf = matches
      .get_one::<PathBuf>("description")
      .expect("Expected valid path to be provided for texture description file or folder");

    let base: &PathBuf = matches
      .get_one::<PathBuf>("base")
      .expect("Expected valid base path to be provided");

    let destination: &PathBuf = matches.get_one::<PathBuf>("dest").unwrap_or(base);

    let files: Vec<String> = matches
      .get_many::<String>("file")
      .map(|values| values.cloned().collect())
      .unwrap_or_default();

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(
      output,
      "Packing texture descriptions {}, from {} to {}",
      format_path(description),
      format_path(base),
      format_path(destination)
    );

    let options: SpriteDescriptionOptions = SpriteDescriptionOptions {
      job: CliJob::new_logging(),
      description: description.clone(),
      base: base.clone(),
      output: output.clone(),
      output_path: destination.clone(),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files,
      is_strict: matches.get_flag("strict"),
      link_policy: LinkPolicyArgument::of(matches),
    };

    let result: SpriteDescriptionResult = PackDescriptionProcessor::pack_xml_descriptions(&options)?;

    xrf_output::success!(
      output,
      "Packed {} sheet(s) with {} sprite(s) in {}",
      result.files,
      result.sprites,
      xrf_utils::format_duration(result.duration)
    );

    context.set_result(|| SpriteDescriptionReport::new(&options, &result))
  }
}

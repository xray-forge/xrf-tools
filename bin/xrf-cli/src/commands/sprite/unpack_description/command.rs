use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_dds::ImageFormat;
use xrf_output::OutputOptions;
use xrf_texture::{SpriteDescriptionOptions, SpriteDescriptionResult, UnpackDescriptionProcessor};
use xrf_utils::format_path;

use crate::commands::sprite::pack_description::report::SpriteDescriptionReport;
use crate::core::arguments::{JobsArgument, LinkPolicyArgument};
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

#[derive(Default)]
pub struct UnpackDescriptionCommand;

impl GenericCommand for UnpackDescriptionCommand {
  fn operation(&self) -> &'static str {
    "unpack-description"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about("Unpack the sprites a texture description xml declares out of its sheets")
      .arg(
        Arg::new("description")
          .help("Path to the XML file describing the sheets")
          .long("description")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("base")
          .help("Folder the described sheets are read from")
          .long("base")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Folder the sprites are written to, defaults to the base folder")
          .short('d')
          .long("dest")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("file")
          .help("Name of a described file to unpack, repeatable; unpacks every described file if omitted")
          .long("file")
          .action(ArgAction::Append),
      )
      .arg(
        Arg::new("strict")
          .help("Fail on a sheet that cannot be read or a sprite outside its sheet instead of skipping it")
          .long("strict")
          .action(ArgAction::SetTrue),
      );

    LinkPolicyArgument::declare(JobsArgument::declare(command))
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
      "Unpacking texture descriptions {}, from {} to {}",
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

    let result: SpriteDescriptionResult = UnpackDescriptionProcessor::unpack_xml_descriptions(&options)?;

    xrf_output::success!(
      output,
      "Unpacked {} sprite(s) from {} sheet(s) in {}",
      result.sprites,
      result.files,
      xrf_utils::format_duration(result.duration)
    );

    context.set_result(|| SpriteDescriptionReport::new(&options, &result))
  }
}

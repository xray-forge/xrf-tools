use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;
use xrf_output::OutputOptions;
use xrf_texture::{GenerateBumpGloss, GenerateBumpOptions, GenerateBumpProcessor, GenerateBumpResult, ImageFileReader};
use xrf_utils::format_path;

use super::report::DdsMakeBumpReport;
use crate::commands::dds::dds_encode_arguments::DdsEncodeArguments;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

#[derive(Default)]
pub struct MakeBumpCommand;

impl GenericCommand for MakeBumpCommand {
  fn operation(&self) -> &'static str {
    "make-bump"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about("Generate the `_bump` and `_bump#` pair a bumped surface binds, from a height map")
      .arg(
        Arg::new("path")
          .help("Path of the height map the relief is read from, averaged across its colour channels")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path of the texture the pair belongs to, without the `_bump` suffix or an extension")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("gloss-alpha")
          .help("Read the gloss from the height map's own alpha, where the SDK's bump source carries it")
          .long("gloss-alpha")
          .conflicts_with_all(["gloss", "gloss-constant"])
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("gloss")
          .help("Path of a gloss mask, averaged across its colour channels")
          .long("gloss")
          .conflicts_with("gloss-constant")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("gloss-constant")
          .help("One gloss level for the whole surface, from 0 to 1, for a texture with no mask")
          .long("gloss-constant")
          .value_parser(value_parser!(f32)),
      )
      .arg(
        Arg::new("normal-map")
          .help("Path of a normal map to use instead of deriving one from the height, of the same size")
          .long("normal-map")
          .value_parser(value_parser!(PathBuf)),
      );

    DdsEncodeArguments::declare(command, DdsEncodeArguments::BUMP_MIP_FILTER).arg(
      Arg::new("virtual-height")
        .help("Relief depth the normals are derived at, `bump_virtual_height` of the descriptor")
        .long("virtual-height")
        .default_value("0.05")
        .value_parser(value_parser!(f32)),
    )
  }

  /// Generate the pair and say what it came to.
  ///
  /// A gloss too dark to show a specular response is reported rather than refused, exactly as the SDK reports it and
  /// keeps the files: a matte surface is a thing somebody may have meant.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let height: &PathBuf = matches.get_one("path").expect("Expected valid height path");
    let destination: &PathBuf = matches.get_one("dest").expect("Expected valid destination path");
    let output: OutputOptions = context.get_output().clone();

    let options: GenerateBumpOptions = GenerateBumpOptions {
      job: JobHandle::inert(),
      bump: GenerateBumpProcessor::bump_of(destination),
      height: ImageFileReader::read_rgba(height)?,
      gloss: Self::get_gloss(matches)?,
      normal_map: match matches.get_one::<PathBuf>("normal-map") {
        Some(path) => Some(ImageFileReader::read_rgba(path)?),
        None => None,
      },
      virtual_height: *matches
        .get_one::<f32>("virtual-height")
        .expect("Expected valid virtual height"),
      mip_filter: DdsEncodeArguments::mip_filter_of(matches),
      quality: DdsEncodeArguments::quality_of(matches),
    };

    let result: GenerateBumpResult = GenerateBumpProcessor::generate(&options)?;

    xrf_output::success!(
      output,
      "Generated {} and {} from {}",
      format_path(&result.bump),
      format_path(&result.companion),
      format_path(height)
    );

    if result.is_gloss_too_dark() {
      xrf_output::warning!(
        output,
        "Gloss power is {:.3}, below {:.1}: the surface will show almost no specular response",
        result.gloss_power,
        GenerateBumpResult::MINIMUM_GLOSS_POWER
      );
    }

    context.set_result(|| DdsMakeBumpReport::new(height, &result))?;

    Ok(())
  }
}

impl MakeBumpCommand {
  /// The gloss the pair carries: the height's own alpha, a mask, one level everywhere, or the default where none was
  /// given.
  fn get_gloss(matches: &ArgMatches) -> XrfResult<GenerateBumpGloss> {
    if matches.get_flag("gloss-alpha") {
      return Ok(GenerateBumpGloss::HeightAlpha);
    }

    if let Some(path) = matches.get_one::<PathBuf>("gloss") {
      return Ok(GenerateBumpGloss::Mask(ImageFileReader::read_rgba(path)?));
    }

    let level: f32 = matches
      .get_one::<f32>("gloss-constant")
      .copied()
      .unwrap_or(GenerateBumpOptions::DEFAULT_GLOSS);

    if !(0.0..=1.0).contains(&level) {
      return Err(XrfError::new_invalid_error(format!(
        "Gloss level {level} is outside the 0 to 1 range a gloss is measured in"
      )));
    }

    Ok(GenerateBumpGloss::Constant(level))
  }
}

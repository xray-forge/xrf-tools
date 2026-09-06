use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;
use xrf_output::OutputOptions;
use xrf_texture::{
  GenerateBumpGloss, GenerateBumpOptions, GenerateBumpProcessor, GenerateBumpResult, read_image_as_rgba,
};
use xrf_utils::format_path;

use super::report::DdsMakeBumpReport;
use crate::commands::dds::dds_encode_arguments::{
  DEFAULT_BUMP_MIP_FILTER, get_mip_filter, get_quality, new_mip_filter_argument, new_quality_argument,
};
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct MakeBumpCommand;

impl GenericCommand for MakeBumpCommand {
  fn operation(&self) -> &'static str {
    "make-bump"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to generate the `_bump` and `_bump#` pair a bumped surface binds, from a height map")
      .arg(
        Arg::new("height")
          .help("Path of the image the relief is read from, averaged across its colour channels")
          .value_name("HEIGHT")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("destination")
          .help("Path of the texture the pair belongs to, without the `_bump` suffix or an extension")
          .value_name("DESTINATION")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
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
      )
      .arg(new_mip_filter_argument(DEFAULT_BUMP_MIP_FILTER))
      .arg(new_quality_argument())
      .arg(
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
    let height: &PathBuf = matches.get_one("height").expect("Expected valid height path");
    let destination: &PathBuf = matches.get_one("destination").expect("Expected valid destination path");
    let output: OutputOptions = context.get_output().clone();

    let options: GenerateBumpOptions = GenerateBumpOptions {
      job: JobHandle::inert(),
      destination: destination.clone(),
      height: read_image_as_rgba(height)?,
      gloss: Self::get_gloss(matches)?,
      normal_map: match matches.get_one::<PathBuf>("normal-map") {
        Some(path) => Some(read_image_as_rgba(path)?),
        None => None,
      },
      virtual_height: *matches
        .get_one::<f32>("virtual-height")
        .expect("Expected valid virtual height"),
      mip_filter: get_mip_filter(matches)?,
      quality: get_quality(matches)?,
    };

    let result: GenerateBumpResult = GenerateBumpProcessor::generate(&options)?;

    xrf_output::info!(
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
  /// The gloss the pair carries: a mask, one level everywhere, or the default where neither was given.
  fn get_gloss(matches: &ArgMatches) -> XrfResult<GenerateBumpGloss> {
    if let Some(path) = matches.get_one::<PathBuf>("gloss") {
      return Ok(GenerateBumpGloss::Mask(read_image_as_rgba(path)?));
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

use clap::{ArgMatches, Command};
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfError;
use xrf_level::LevelFile;
use xrf_material::XrayTextureScope;
use xrf_output::OutputOptions;
use xrf_vfs::XrayLogicalPath;

use super::level_albedo_survey::LevelAlbedoSurvey;
use super::report::{LevelAlbedoReport, LevelAlbedoSummary};
use crate::commands::level::albedo::albedo_texture::AlbedoOutcome;
use crate::commands::level::level_assets::LevelAssets;
use crate::commands::level::level_selection::LevelSelection;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

#[derive(Default)]
pub struct AlbedoCommand;

impl GenericCommand for AlbedoCommand {
  fn operation(&self) -> &'static str {
    "albedo"
  }

  fn declare(&self, command: Command) -> Command {
    LevelSelection::declare(command.about(
      "Report how the base textures of a level's lit surfaces sit as albedo, against the range real materials span",
    ))
  }

  /// Measures every base texture the level's albedo-writing surfaces dress with, decoded as the renderer decodes it.
  ///
  /// A report, not a check: a painted texture is the game's art as shipped, so outliers are counted, not failed.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let selection: LevelSelection = LevelSelection::of(matches)?;
    let output: OutputOptions = context.get_output().clone();
    let assets: LevelAssets = selection.open()?;

    // Textures are found as the engine finds them, beside the level and then in the shared tree, which only mounted
    // roots hold.
    let LevelAssets::Asset { search, .. } = &assets else {
      return Err(
        XrfError::new_invalid_error(
          "The albedo report finds a level's textures through mounted roots: name it with --root and --level",
        )
        .into(),
      );
    };

    let bundle: Vec<u8> = assets.read(LevelFile::FILE_NAME)?.ok_or_else(|| {
      XrfError::new_not_found_error(format!(
        "Level bundle was not found: {}",
        assets.describe_file(LevelFile::FILE_NAME)
      ))
    })?;
    let level: LevelFile = LevelFile::read_from_bytes::<XRayByteOrder>(bundle)?;
    let scope: XrayTextureScope = XrayTextureScope::of_level(XrayLogicalPath::new(&assets.describe())?);

    xrf_output::info!(output, "Measuring the albedo of compiled level {}", assets.describe());

    let survey: LevelAlbedoSurvey = LevelAlbedoSurvey::run(search, scope, &level);

    let summary: LevelAlbedoSummary = LevelAlbedoSummary::of(&survey);

    Self::print(&output, &survey, &summary);

    context.set_result(|| LevelAlbedoReport::new(assets.describe(), &survey, summary))?;

    Ok(())
  }
}

impl AlbedoCommand {
  fn print(output: &OutputOptions, survey: &LevelAlbedoSurvey, summary: &LevelAlbedoSummary) {
    xrf_output::info!(
      output,
      "{} surfaces, {} writing no albedo; {} base textures, {} measured",
      summary.surfaces,
      summary.unlit_surfaces,
      summary.textures,
      summary.measured
    );
    xrf_output::info!(
      output,
      "Against linear {} to {}: {} dark, {} within, {} bright; median albedo {:.3}, median coarse spread {:.2} stops",
      summary.floor,
      summary.ceiling,
      summary.dark,
      summary.within,
      summary.bright,
      summary.median_albedo,
      summary.median_coarse_spread
    );

    for texture in &survey.textures {
      match &texture.outcome {
        AlbedoOutcome::Measured { albedo } => xrf_output::verbose!(
          output,
          "{}: median {:.3} ({:.3} to {:.3}), {:.0}% dark, {:.0}% bright, spread {:.2} stops",
          texture.reference,
          albedo.median,
          albedo.low,
          albedo.high,
          albedo.dark_share * 100.0,
          albedo.bright_share * 100.0,
          albedo.coarse_spread
        ),
        AlbedoOutcome::Missing => xrf_output::warning!(output, "{}: no root holds it", texture.reference),
        AlbedoOutcome::Unread { reason } => xrf_output::warning!(output, "{}: {}", texture.reference, reason),
        AlbedoOutcome::Empty => {
          xrf_output::verbose!(output, "{}: its alpha test keeps no texel", texture.reference)
        }
      }
    }
  }
}

use clap::{ArgMatches, Command};
use xrf_output::OutputOptions;
use xrf_texture::BumpAlignmentVerdict;
use xrf_vfs::XraySearch;

use super::bump_alignment_survey::BumpAlignmentSurvey;
use super::report::{BumpAlignmentReport, BumpAlignmentSummary};
use crate::commands::thm::bump_alignment::bump_alignment_set::BumpAlignmentOutcome;
use crate::core::arguments::XrayRootsArguments;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

#[derive(Default)]
pub struct BumpAlignmentCommand;

impl GenericCommand for BumpAlignmentCommand {
  fn operation(&self) -> &'static str {
    "bump-alignment"
  }

  fn declare(&self, command: Command) -> Command {
    XrayRootsArguments::declare(
      command.about(
        "Report whether each texture's bump belongs to its diffuse, by how closely the bump's slopes follow the \
         diffuse's edges",
      ),
      "Root holding the textures and their descriptors. Repeat to layer roots, highest priority first",
      "Limit to the descriptors of one logical subtree, such as textures\\wpn",
    )
  }

  /// Compares every set a descriptor declares a bump for, each found as the renderer finds it outside a level.
  ///
  /// A report, not a check: the verdict is a measure's, and a soft or periodic texture can read as doubtful while
  /// belonging to its bump, so mismatches are counted, not failed.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();
    let search: XraySearch = XrayRootsArguments::of(matches)?.to_search_plan()?.open()?;
    let listed: XraySearch = search
      .clone()
      .with_optional_prefix(XrayRootsArguments::prefix_of(matches))?;

    xrf_output::info!(
      output,
      "Comparing the bumps texture descriptors declare with their diffuses"
    );

    let survey: BumpAlignmentSurvey = BumpAlignmentSurvey::run(&search, &listed);
    let summary: BumpAlignmentSummary = BumpAlignmentSummary::of(&survey);

    Self::print(&output, &survey, &summary);

    context.set_result(|| BumpAlignmentReport::new(&survey, summary))?;

    Ok(())
  }
}

impl BumpAlignmentCommand {
  fn print(output: &OutputOptions, survey: &BumpAlignmentSurvey, summary: &BumpAlignmentSummary) {
    xrf_output::info!(
      output,
      "{} descriptors, {} declaring a bump; {} sets compared, {} missing a diffuse, {} missing a bump, {} unread",
      summary.descriptors,
      summary.sets,
      summary.measured,
      summary.missing_diffuses,
      summary.missing_bumps,
      summary.unread
    );
    xrf_output::info!(
      output,
      "{} aligned, {} doubtful, {} mismatched ({} with their own bump), {} featureless; {} borrow another's bump",
      summary.aligned,
      summary.doubtful,
      summary.mismatched,
      summary.mismatched_own,
      summary.featureless,
      summary.borrowed
    );

    for set in &survey.sets {
      match &set.outcome {
        // A set's own bump not following it is what a pack installed in part leaves; a borrowed one often does not.
        BumpAlignmentOutcome::Measured { alignment }
          if alignment.verdict == BumpAlignmentVerdict::Mismatched && !set.is_borrowed =>
        {
          xrf_output::warning!(
            output,
            "{}: its own bump does not follow it, correlation {:.2} against chance {:.2}",
            set.descriptor,
            alignment.correlation.unwrap_or_default(),
            alignment.chance.unwrap_or_default()
          )
        }
        BumpAlignmentOutcome::Measured { alignment } => xrf_output::verbose!(
          output,
          "{}: {} with {}, lift {}",
          set.descriptor,
          alignment.verdict.label(),
          set.bump,
          alignment
            .lift
            .map_or_else(|| String::from("none"), |lift| format!("{lift:.2}"))
        ),
        BumpAlignmentOutcome::Unread { reason } => xrf_output::warning!(output, "{}: {}", set.descriptor, reason),
        BumpAlignmentOutcome::MissingDiffuse => {
          xrf_output::verbose!(output, "{}: no root holds its diffuse", set.descriptor)
        }
        BumpAlignmentOutcome::MissingBump => {
          xrf_output::verbose!(output, "{}: no root holds its bump {}", set.descriptor, set.bump)
        }
      }
    }
  }
}

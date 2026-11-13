use clap::{ArgMatches, Command};
use xrf_level::{LevelAiGraphVerifier, LevelAiGraphVerifyResult, LevelGeometryVerifier, LevelGeometryVerifyResult};
use xrf_math::Aabb;
use xrf_output::OutputOptions;
use xrf_report::{CheckReport, Report};

use crate::commands::level::level_assets::LevelAssets;
use crate::commands::level::level_selection::LevelSelection;
use crate::commands::level::verify::report::LevelVerifyReport;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::report::ReportVerdict;

#[derive(Default)]
pub(crate) struct VerifyCommand;

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  fn declare(&self, command: Command) -> Command {
    LevelSelection::declare(command.about(
      "Verify that a compiled level's geometry can be drawn the way the renderer addresses it, and that its AI map is \
       a graph the engine can walk",
    ))
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let selection: LevelSelection = LevelSelection::of(matches)?;
    let output: OutputOptions = context.get_output().clone();

    // The mount outlives the assets that borrow from it, so it is taken before the level is opened.
    let assets: LevelAssets = selection.open()?;

    xrf_output::info!(output, "Verifying compiled level {}", assets.describe());

    let result: LevelGeometryVerifyResult = LevelGeometryVerifier::new(&assets).run();
    let ai: LevelAiGraphVerifyResult = LevelAiGraphVerifier::verify_source(&assets);
    let report: Report = Report::new(
      result
        .report
        .checks()
        .iter()
        .chain(ai.report.checks())
        .cloned()
        .collect::<Vec<CheckReport>>(),
    );

    Self::print_census(&output, &result);
    Self::print_ai_census(&output, &ai);
    ReportVerdict::print_findings(&output, &report);

    // Deposited before the verdict becomes an outcome, so a failing check still reports what explains it.
    context.set_result(|| LevelVerifyReport::new(&result.census, ai.census.as_ref(), &report))?;

    ReportVerdict::conclude(&output, &report, "Verification of the level")
  }
}

impl VerifyCommand {
  fn print_ai_census(output: &OutputOptions, result: &LevelAiGraphVerifyResult) {
    let Some(census) = &result.census else {
      xrf_output::info!(output, "No AI map was checked");

      return;
    };

    xrf_output::info!(
      output,
      "AI map of {} nodes and {} links in {}: {} one-way to {} nodes, {} to themselves; {} parts, the largest {} nodes",
      census.nodes,
      census.links,
      xrf_utils::format_duration(result.duration),
      census.one_way_links,
      census.one_way_nodes,
      census.self_links,
      census.components,
      census.largest_component
    );
  }

  fn print_census(output: &OutputOptions, result: &LevelGeometryVerifyResult) {
    xrf_output::info!(
      output,
      "Read {} visuals in {}, {} drawable and {} with a fast path",
      result.census.visuals,
      xrf_utils::format_duration(result.duration),
      result.census.drawable_visuals,
      result.census.fastpath_visuals
    );
    xrf_output::info!(
      output,
      "Addressed {} vertices and {} indices against a {} entry shader table",
      result.census.vertices,
      result.census.indices,
      result.census.shader_entries
    );

    xrf_output::info!(
      output,
      "{} sectors joined by {} portals, named {} times between them",
      result.census.sectors,
      result.census.portals,
      result.census.sector_portal_references
    );

    xrf_output::info!(
      output,
      "Sectors reach {} visuals, {} drawables reached by more than one, {} reached by none",
      result.census.sector_visuals,
      result.census.shared_drawables,
      result.census.orphaned_drawables
    );

    xrf_output::info!(
      output,
      "{} static lights{}",
      result.census.lights,
      if result.census.has_sun {
        ", one of them the sun"
      } else {
        ", none of them a sun"
      }
    );

    if let Some(Aabb { min, max }) = &result.census.bounds {
      xrf_output::info!(
        output,
        "Decoded geometry spans ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1}), widest texture coordinate {:.1} tiles",
        min.x,
        min.y,
        min.z,
        max.x,
        max.y,
        max.z,
        result.census.widest_coordinate
      );
    }

    if result.census.stray_normals > 0 {
      xrf_output::warning!(
        output,
        "{} decoded normals are not unit length, which a misread normal element would explain",
        result.census.stray_normals
      );
    }

    for (layout, count) in &result.census.layouts {
      xrf_output::verbose!(output, "Layout {layout}: {count} visuals");
    }

    for (model_type, count) in &result.census.model_types {
      xrf_output::verbose!(output, "{model_type}: {count} visuals");
    }
  }
}

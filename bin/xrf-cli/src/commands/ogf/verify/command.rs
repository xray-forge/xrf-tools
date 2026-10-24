use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use crate::commands::ogf::verify::ogf_verifier::{OgfVerificationCensus, OgfVerificationResult, OgfVerifier};
use crate::commands::ogf::verify::report::OgfVerifyReport;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::report::ReportVerdict;

#[derive(Default)]
pub struct VerifyCommand;

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about("Verify that OGF visuals can be packed for rendering")
      .arg(
        Arg::new("path")
          .help("Path to an ogf file or a directory to sweep")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("root")
          .help("Additional root searched for textures after the visual's own tree, repeatable")
          .long("root")
          .action(ArgAction::Append)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");
    let roots: Vec<PathBuf> = matches
      .get_many::<PathBuf>("root")
      .map(|values| values.cloned().collect())
      .unwrap_or_default();

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Verifying ogf visuals in {}", format_path(path));

    for root in &roots {
      xrf_output::info!(output, "Searching textures in {} as well", format_path(root));
    }

    let result: OgfVerificationResult = OgfVerifier::new(path, roots.clone()).run();

    Self::print_census(&output, &result);
    ReportVerdict::print_findings(&output, &result.report);

    // Deposited before the verdict becomes an outcome, so a failing check still reports the findings that explain it.
    context.set_result(|| OgfVerifyReport::new(&result.census, &result.report))?;

    ReportVerdict::conclude(&output, &result.report, "Verification of ogf visuals")
  }
}

impl VerifyCommand {
  fn print_census(output: &OutputOptions, result: &OgfVerificationResult) {
    let census: &OgfVerificationCensus = &result.census;

    xrf_output::info!(
      output,
      "Swept {} visuals in {}",
      census.files,
      xrf_utils::format_duration(result.duration)
    );
    xrf_output::info!(
      output,
      "Submeshes: {} total, {} packed, {} unsupported, {} malformed",
      census.submeshes,
      census.packed_submeshes,
      census.unsupported_submeshes,
      census.malformed_submeshes
    );
    xrf_output::info!(
      output,
      "Progressive submeshes: {}, of which {} draw only part of their index buffer",
      census.progressive_submeshes,
      census.progressive_submeshes_drawing_part_of_the_buffer
    );
    xrf_output::info!(
      output,
      "Skinned submeshes: {}, vertices whose weights do not sum to one: {}",
      census.skinned_submeshes,
      census.vertices_with_stray_skin_weights
    );
    xrf_output::info!(
      output,
      "Unreadable files: {}, files with nothing drawable: {}, bounds disagreements: {}",
      census.unreadable_files,
      census.files_without_geometry,
      census.bounds_disagreements
    );

    xrf_output::info!(
      output,
      "Texture references: {} total, {} resolved, {} missing, {} unreadable, {} visuals with no root",
      census.texture_references,
      census.resolved_texture_references,
      census.missing_texture_references,
      census.unreadable_textures,
      census.visuals_without_root
    );
    xrf_output::info!(
      output,
      "Distinct textures: {}, of which {} carry no mip chain; {} references resolved inside archives and were not read",
      census.distinct_textures,
      census.textures_without_mipmaps,
      census.located_texture_references
    );

    Self::print_distribution(output, "Header versions", &census.versions);
    Self::print_distribution(output, "Root model types", &census.root_model_types);
    Self::print_distribution(output, "Submesh model types", &census.submesh_model_types);
    Self::print_distribution(output, "Vertex formats", &census.vertex_formats);
    Self::print_distribution(output, "Texture formats", &census.texture_formats);
    Self::print_distribution(output, "Textures resolved by", &census.texture_steps);
    Self::print_distribution(output, "Texture sizes", &census.texture_sizes);
  }

  /// Prints one distribution, or nothing when it counted nothing.
  fn print_distribution<K: std::fmt::Display>(output: &OutputOptions, label: &str, counts: &BTreeMap<K, usize>) {
    if let Some(rendered) = ReportVerdict::format_distribution(counts) {
      xrf_output::info!(output, "{label}: {rendered}");
    }
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::VerifyCommand;
  use crate::core::command::GenericCommand;

  fn roots(arguments: &[&str]) -> Option<Vec<PathBuf>> {
    VerifyCommand
      .init()
      .try_get_matches_from(arguments)
      .expect("expect the arguments to parse")
      .get_many::<PathBuf>("root")
      .map(|values| values.cloned().collect())
  }

  #[test]
  fn takes_every_root_it_is_given_in_order() {
    // Repeatable and ordered: an overlay is layered over a base, and sometimes over a base and the installation
    // behind it, which is the order the engine reads them in.
    assert_eq!(
      roots(&["verify", "--path", "meshes", "--root", "base", "--root", "install"]),
      Some(vec![PathBuf::from("base"), PathBuf::from("install")])
    );
  }

  #[test]
  fn sweeps_one_tree_when_no_root_is_named() {
    assert_eq!(roots(&["verify", "--path", "meshes"]), None);
  }
}

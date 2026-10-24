use clap::{Arg, ArgAction, ArgMatches, Command};
use xrf_output::OutputOptions;
use xrf_report::Status;
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::commands::dialog::dialog_project_opening::DialogProjectOpening;
use crate::commands::dialog::info::dialog_sweep::{DialogSweep, DialogSweepCensus, DialogSweepResult};
use crate::commands::dialog::info::report::DialogInfoReport;
use crate::core::arguments::XrayRootsArguments;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::report::ReportVerdict;

#[derive(Default)]
pub struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  fn declare(&self, command: Command) -> Command {
    DialogProjectOpening::declare_roots(command.about("Read dialog XML and report what it holds")).arg(
      Arg::new("strict")
        .help("Answer with a check failure when anything was unreadable or off schema")
        .long("strict")
        .action(ArgAction::SetTrue),
    )
  }

  /// Read every dialog file under the provided path and report the census and findings.
  ///
  /// Reporting is the default and answers success: a sweep over reference trees exists to produce a
  /// tally, and a tally that also fails the build cannot be run casually. `--strict` is the mode that
  /// judges, and it is the one a CI step uses.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let is_strict: bool = matches.get_flag("strict");

    let output: OutputOptions = context.get_output().clone();
    let source: XrayMountMode = XrayRootsArguments::source_of(matches)?;
    let roots: XrayRoots = XrayRootsArguments::of(matches)?;

    xrf_output::info!(output, "Reading dialogs in {} ({source})", roots.describe());

    // A roots that cannot be mounted is an execution failure, which the mount itself answers with, so
    // no separate existence check is needed here.
    let result: DialogSweepResult = DialogSweep::new(&roots, XrayRootsArguments::prefix_of(matches)).run()?;

    Self::print_census(&output, &result);
    ReportVerdict::print_findings(&output, &result.report);

    // Deposited before the verdict becomes an outcome, so a failing check still reports the findings that explain it.
    context.set_result(|| DialogInfoReport::new(&result.census, &result.report))?;

    // Findings are reported and answered as success by default: a tally that also fails the build cannot be run
    // casually. `--strict` is the mode that judges, and it is what a CI step uses. A sweep that reached no verdict
    // fails either way.
    if !is_strict && result.report.status() == Status::Failed {
      xrf_output::success!(output, "Read {} files, status: {}", result.census.files, Status::Failed);

      return Ok(());
    }

    ReportVerdict::conclude(
      &output,
      &result.report,
      &format!("Reading dialogs in {}", roots.describe()),
    )
  }
}

impl InfoCommand {
  fn print_census(output: &OutputOptions, result: &DialogSweepResult) {
    let census: &DialogSweepCensus = &result.census;

    xrf_output::info!(
      output,
      "Swept {} files in {}, {} unreadable, {} archived",
      census.files,
      xrf_utils::format_duration(result.duration),
      census.unreadable_files,
      census.archived_files
    );
    xrf_output::info!(
      output,
      "Dialogs: {} total, {} with no phrases, {} with a priority",
      census.dialogs,
      census.dialogs_without_phrases,
      census.dialogs_with_priority
    );
    xrf_output::info!(
      output,
      "Phrases: {} total, {} links, {} final, {} without text, {} outside a phrase list",
      census.phrases,
      census.links,
      census.final_phrases,
      census.phrases_without_text,
      census.phrases_outside_phrase_list
    );

    if let Some(id) = &census.largest_dialog_id {
      xrf_output::info!(
        output,
        "Largest dialog: {} with {} phrases",
        id,
        census.largest_dialog_phrases
      );
    }

    for (label, counts) in [
      ("Encodings", &census.encodings),
      ("Dialog elements", &census.dialog_elements),
      ("Phrase elements", &census.phrase_elements),
    ] {
      if let Some(rendered) = ReportVerdict::format_distribution(counts) {
        xrf_output::info!(output, "{label}: {rendered}");
      }
    }
  }
}

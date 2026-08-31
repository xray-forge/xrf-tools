use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_report::Status;
use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots};

use crate::commands::dialog::info::dialog_sweep::{
  DialogSweep, DialogSweepCensus, DialogSweepResult, list_distribution, sum_findings,
};
use crate::core::command_context::CommandContext;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to read dialog xml and report what it holds")
      .arg(
        Arg::new("path")
          .help("Root holding dialog xml. Repeat to layer roots, highest priority first")
          .short('p')
          .long("path")
          .required(true)
          // Both spellings layer: repeat the flag, or list several values after one of them.
          .action(ArgAction::Append)
          .num_args(1..)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("source")
          .help(
            "How to read the path: auto treats it as an installation only when it declares one, directory ignores any declaration, volumes mounts every archive volume beneath it, installation requires one, containing-installation searches parent directories for one",
          )
          .long("source")
          .default_value("containing-installation")
          .value_parser(["auto", "directory", "volumes", "installation", "containing-installation"]),
      )
      .arg(
        Arg::new("prefix")
          .help("Limit to one logical subtree, such as configs\\gameplay")
          .long("prefix")
          .value_parser(value_parser!(String)),
      )
      .arg(
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
    let paths: Vec<&PathBuf> = matches
      .get_many::<PathBuf>("path")
      .expect("Expected at least one path to be provided")
      .collect();
    let is_strict: bool = matches.get_flag("strict");

    let output: OutputOptions = context.get_output().clone();

    let source: XrayMountMode = XrayMountMode::try_from(
      matches
        .get_one::<String>("source")
        .expect("Expected source mode to default")
        .as_str(),
    )?;
    let prefix: Option<&String> = matches.get_one::<_>("prefix");

    // One vocabulary for naming roots, so repeating `--path` layers a tree in front of an
    // installation exactly as the desktop app does it.
    let roots: XrayRoots = XrayRoots::new(
      paths
        .iter()
        .map(|path| XrayRoot::new(path.display().to_string(), source)),
    );

    xrf_output::info!(output, "Reading dialogs in {} ({:?})", roots.describe(), source);

    // A roots that cannot be mounted is an execution failure, which the mount itself answers with, so
    // no separate existence check is needed here.
    let result: DialogSweepResult = DialogSweep::new(&roots, prefix.map(String::as_str)).run()?;

    Self::print_census(&output, &result);
    Self::print_findings(&output, &result);

    // Deposited before the verdict becomes an outcome, so a failing check still reports the findings that explain it.
    context.set_result(|| &result.report)?;

    let status: Status = result.report.status();

    match status {
      Status::Passed => {
        xrf_output::success!(output, "Read {} files, status: {}", result.census.files, status);

        Ok(())
      }
      // Findings are reported and answered as success by default: a tally that also fails the build
      // cannot be run casually. `--strict` is the mode that judges, and it is what a CI step uses.
      Status::Failed if is_strict => Err(CommandError::new_check_failed(sum_findings(&result.report).max(1))),
      Status::Failed => {
        xrf_output::success!(output, "Read {} files, status: {}", result.census.files, status);

        Ok(())
      }
      // The sweep reached no verdict, so this is an execution failure rather than a check result,
      // whether or not `--strict` was asked for.
      Status::Error | Status::Incomplete | Status::Skipped => Err(
        XrfError::new_verify_error(format!(
          "No dialog files were read under {}, status: {status}",
          roots.describe()
        ))
        .into(),
      ),
    }
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

    xrf_output::info!(output, "Encodings: {}", list_distribution(&census.encodings).join(", "));
    xrf_output::info!(
      output,
      "Dialog elements: {}",
      list_distribution(&census.dialog_elements).join(", ")
    );
    xrf_output::info!(
      output,
      "Phrase elements: {}",
      list_distribution(&census.phrase_elements).join(", ")
    );
  }

  /// List findings under verbose output only: a sweep of a whole tree can produce more of them than a
  /// terminal is useful for, and the json report is the artifact meant for comparison.
  fn print_findings(output: &OutputOptions, result: &DialogSweepResult) {
    for check in result.report.checks() {
      if check.findings().is_empty() {
        continue;
      }

      xrf_output::info!(
        output,
        "Check '{}' is {} with {} findings",
        check.id(),
        check.status(),
        check.findings().len()
      );

      for finding in check.findings() {
        xrf_output::verbose!(
          output,
          "  [{}] {}: {}",
          finding.rule_id(),
          finding.subject().unwrap_or("-"),
          finding.message()
        );
      }
    }
  }
}

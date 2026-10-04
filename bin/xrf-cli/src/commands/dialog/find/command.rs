use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command, value_parser};
use xrf_dialog::{DialogElement, DialogElementKind, DialogProject, DialogReferenceDescriptor};
use xrf_output::OutputOptions;

use super::report::DialogFindReport;
use crate::commands::dialog::dialog_project_open::open_dialog_project;
use crate::commands::dialog::dialog_roots::DialogRootsArguments;
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

const INFO_ARGUMENT: &str = "info";
const FUNCTOR_ARGUMENT: &str = "functor";

#[derive(Default)]
pub struct FindCommand;

impl GenericCommand for FindCommand {
  fn operation(&self) -> &'static str {
    "find"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Find every dialog and phrase naming an info portion or a script function, and what each does with it")
      .with_dialog_roots()
      .arg(
        Arg::new(INFO_ARGUMENT)
          .help("Info portion to find where dialogs check, give or take it. Repeat to find several")
          .long(INFO_ARGUMENT)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new(FUNCTOR_ARGUMENT)
          .help(
            "Script function to find where dialogs call it, as module.function or a bare function name in any module. Repeat to find several",
          )
          .long(FUNCTOR_ARGUMENT)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .group(
        ArgGroup::new("query")
          .args([INFO_ARGUMENT, FUNCTOR_ARGUMENT])
          .required(true)
          .multiple(true),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();
    let infos: Vec<String> = Self::requested(matches, INFO_ARGUMENT);
    let functors: Vec<String> = Self::requested(matches, FUNCTOR_ARGUMENT);
    let project: DialogProject = open_dialog_project(matches)?;

    let report: DialogFindReport = DialogFindReport {
      references: project.list_references(|element| {
        Self::find_info(&infos, element).is_some() || Self::find_functor(&functors, element).is_some()
      }),
      infos,
      functors,
    };

    Self::render(&report, &output);

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl FindCommand {
  fn requested(matches: &ArgMatches, argument: &str) -> Vec<String> {
    matches
      .get_many::<String>(argument)
      .map(|values| values.cloned().collect())
      .unwrap_or_default()
  }

  /// The searched info portion an element names, if it names one.
  fn find_info<'a>(infos: &'a [String], element: &DialogElement) -> Option<&'a String> {
    if !element.get_kind().is_info_portion() {
      return None;
    }

    infos.iter().find(|info| *info == element.get_value().trim())
  }

  /// The searched function an element calls, if it calls one.
  fn find_functor<'a>(functors: &'a [String], element: &DialogElement) -> Option<&'a String> {
    if !element.get_kind().is_script_call() {
      return None;
    }

    let value: &str = element.get_value().trim();

    functors.iter().find(|functor| Self::is_functor_match(functor, value))
  }

  /// Whether a query names a call: exactly with a module, or by its function part without one.
  fn is_functor_match(query: &str, value: &str) -> bool {
    value == query || (!query.contains('.') && value.rsplit_once('.').is_some_and(|(_, function)| function == query))
  }

  fn render(report: &DialogFindReport, output: &OutputOptions) {
    for info in &report.infos {
      let found: Vec<&DialogReferenceDescriptor> = report
        .references
        .iter()
        .filter(|reference| reference.element.kind.is_info_portion() && reference.element.value.trim() == info)
        .collect();
      let count = |kinds: &[DialogElementKind]| {
        found
          .iter()
          .filter(|reference| !reference.is_ignored && kinds.contains(&reference.element.kind))
          .count()
      };
      let given: usize = count(&[DialogElementKind::GiveInfo]);
      let read: usize = count(&[DialogElementKind::HasInfo, DialogElementKind::DontHasInfo]);

      xrf_output::heading!(
        output,
        "Info portion '{info}': given {given}, taken {}, checked {read}",
        count(&[DialogElementKind::DisableInfo])
      );
      Self::render_references(&found, output);

      if read > 0 && given == 0 {
        xrf_output::warning!(
          output,
          "No dialog gives '{info}'; scripts, logic and tasks may still give it"
        );
      }
    }

    for functor in &report.functors {
      let found: Vec<&DialogReferenceDescriptor> = report
        .references
        .iter()
        .filter(|reference| {
          reference.element.kind.is_script_call() && Self::is_functor_match(functor, reference.element.value.trim())
        })
        .collect();

      xrf_output::heading!(output, "Function '{functor}': {} references", found.len());
      Self::render_references(&found, output);
    }

    xrf_output::success!(output, "Found {} references", report.references.len());
  }

  fn render_references(references: &[&DialogReferenceDescriptor], output: &OutputOptions) {
    for reference in references {
      let place: String = match &reference.phrase_id {
        Some(phrase) => format!("[{phrase}]"),
        None => String::from("dialog"),
      };
      let note: &str = if reference.is_ignored {
        " (never read: the entry phrase's own condition)"
      } else {
        ""
      };

      xrf_output::info!(
        output,
        "  {} {} {place} {} {}{note}",
        reference.logical_path,
        reference.dialog_id,
        reference.element.name,
        reference.element.value
      );
    }
  }
}

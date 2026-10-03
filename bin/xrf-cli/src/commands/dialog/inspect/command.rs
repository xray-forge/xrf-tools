use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_dialog::{
  DialogDescriptor, DialogElementDescriptor, DialogElementKind, DialogPhraseDescriptor, DialogProject,
  DialogProjectFile,
};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;

use super::report::DialogInspectReport;
use crate::commands::dialog::dialog_project_open::{
  DialogProjectArguments, open_dialog_project, requested_dialog_language,
};
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct InspectCommand;

impl GenericCommand for InspectCommand {
  fn operation(&self) -> &'static str {
    "inspect"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Explain one dialog: its conditions and every phrase, with what each says, gives and runs")
      .with_dialog_project()
      .arg(
        Arg::new("dialog")
          .help("Id of the dialog to explain")
          .required(true)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("file")
          .help("Logical path of the file to read the dialog from, when more than one declares it")
          .long("file")
          .value_parser(value_parser!(String)),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let id: &String = matches
      .get_one::<String>("dialog")
      .expect("Expected a dialog id to be provided");

    let output: OutputOptions = context.get_output().clone();
    let project: DialogProject = open_dialog_project(matches)?;
    let declaring: Vec<&DialogProjectFile> = project.list_files_declaring(id);
    let file: &DialogProjectFile = Self::locate(&project, &declaring, id, matches.get_one::<String>("file"))?;

    let Some(dialog) = project.describe_dialog(file.get_logical_path(), id, requested_dialog_language(matches)) else {
      return Err(
        XrfError::new_read_error(format!(
          "Dialog '{id}' is declared in '{}' but could not be read back",
          file.get_logical_path()
        ))
        .into(),
      );
    };

    let report: DialogInspectReport = DialogInspectReport {
      dialog,
      also_declared_in: declaring
        .iter()
        .filter(|other| other.get_logical_path() != file.get_logical_path())
        .map(|other| other.get_logical_path().to_owned())
        .collect(),
    };

    Self::render(&report, &output);

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl InspectCommand {
  /// The file to read a dialog from: the one `--file` names, or else the first declaring it.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when no file declares the id, or when `--file` names one that does not.
  fn locate<'a>(
    project: &DialogProject,
    declaring: &[&'a DialogProjectFile],
    id: &str,
    requested: Option<&String>,
  ) -> XrfResult<&'a DialogProjectFile> {
    let found: Option<&&DialogProjectFile> = match requested {
      Some(requested) => declaring
        .iter()
        .find(|file| file.get_logical_path().eq_ignore_ascii_case(requested)),
      None => declaring.first(),
    };

    found.copied().ok_or_else(|| match requested {
      Some(requested) => XrfError::new_not_found_error(format!("Dialog '{id}' is not declared in '{requested}'")),
      None => XrfError::new_not_found_error(format!(
        "No file under '{}' declares dialog '{id}'",
        project.get_dialogs_prefix()
      )),
    })
  }

  fn render(report: &DialogInspectReport, output: &OutputOptions) {
    let dialog: &DialogDescriptor = &report.dialog;

    xrf_output::heading!(output, "Dialog '{}' in {}", dialog.id, dialog.logical_path);

    for element in &dialog.elements {
      xrf_output::info!(output, "  {}: {}", element.name, element.value);
    }

    for phrase in &dialog.phrases {
      xrf_output::info!(output, "{}", Self::describe(phrase));
    }

    if !report.also_declared_in.is_empty() {
      xrf_output::warning!(
        output,
        "Also declared in {}, ask with --file to read another",
        report.also_declared_in.join(", ")
      );
    }

    xrf_output::success!(output, "Explained {} phrases", dialog.phrases.len());
  }

  /// One phrase as a line: its id and line, then what follows it and what saying it does.
  fn describe(phrase: &DialogPhraseDescriptor) -> String {
    let line: &str = phrase.text.as_deref().or(phrase.text_key.as_deref()).unwrap_or("-");
    let effects: Vec<String> = phrase
      .elements
      .iter()
      .filter(|element| !matches!(element.kind, DialogElementKind::Text | DialogElementKind::Next))
      .map(|element: &DialogElementDescriptor| format!("{} {}", element.name, element.value))
      .collect();

    let mut described: String = format!("  [{}] {line}", phrase.id);

    if !phrase.next.is_empty() {
      described.push_str(&format!(" -> {}", phrase.next.join(", ")));
    }

    if !effects.is_empty() {
      described.push_str(&format!(" | {}", effects.join("; ")));
    }

    described
  }
}

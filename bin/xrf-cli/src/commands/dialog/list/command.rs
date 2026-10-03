use clap::{Arg, ArgMatches, Command, value_parser};
use indexmap::IndexMap;
use xrf_dialog::{
  Dialog, DialogCharacter, DialogFinding, DialogOffer, DialogPartners, DialogProject, DialogProjectFile,
  DialogTextLanguage,
};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;

use super::report::{DialogListEntry, DialogListProfile, DialogListReport};
use crate::commands::dialog::dialog_project_open::{
  DialogProjectArguments, open_dialog_project, requested_dialog_text,
};
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct ListCommand;

impl GenericCommand for ListCommand {
  fn operation(&self) -> &'static str {
    "list"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("List the dialogs a tree holds, or the ones reaching the actor from an NPC of one profile")
      .with_dialog_project()
      .arg(
        Arg::new("profile")
          .help("NPC profile, as profile_name() answers it, to list only the dialogs the actor gets talking to one")
          .long("profile")
          .value_parser(value_parser!(String)),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();
    let project: DialogProject = open_dialog_project(matches)?;
    let text: Option<DialogTextLanguage<'_>> = requested_dialog_text(matches, &project);

    let report: DialogListReport = match matches.get_one::<String>("profile") {
      Some(profile) => Self::list_offered(&project, profile, text)?,
      None => Self::list_all(&project, text),
    };

    for entry in &report.dialogs {
      xrf_output::info!(output, "{}", Self::describe(entry));
    }

    for finding in &report.findings {
      xrf_output::verbose!(
        output,
        "  [{}] {}: {}",
        finding.rule,
        finding.subject.as_deref().unwrap_or("-"),
        finding.message
      );
    }

    match &report.profile {
      Some(profile) => xrf_output::success!(
        output,
        "Profile '{}' ({}) offers {} dialogs",
        profile.id,
        profile.characters.join(", "),
        report.dialogs.len()
      ),
      None => xrf_output::success!(output, "Listed {} dialogs", report.dialogs.len()),
    }

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl ListCommand {
  /// Every dialog the tree holds, file by file.
  fn list_all(project: &DialogProject, text: Option<DialogTextLanguage<'_>>) -> DialogListReport {
    DialogListReport {
      language: text.map(|text| text.get_language().to_owned()),
      profile: None,
      dialogs: project
        .get_files()
        .iter()
        .flat_map(|file| {
          file
            .get_file()
            .get_dialogs()
            .iter()
            .map(move |dialog| DialogListEntry::new(file, dialog, text, Vec::new()))
        })
        .collect(),
      findings: project.get_findings().to_vec(),
    }
  }

  /// The dialogs reaching the actor from an NPC of one profile, each with every way it does.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when no profile has the id, or when it resolves to no character: the engine refuses to
  /// spawn an NPC of either.
  fn list_offered(
    project: &DialogProject,
    profile: &str,
    text: Option<DialogTextLanguage<'_>>,
  ) -> XrfResult<DialogListReport> {
    let partners: DialogPartners = DialogPartners::from_vfs(project.get_vfs(), project.get_dialogs_prefix())?;
    let mut findings: Vec<DialogFinding> = project.get_findings().to_vec();

    findings.extend_from_slice(partners.get_findings());

    let Some(profile) = partners.find_profile(profile) else {
      return Err(XrfError::new_not_found_error(format!(
        "No profile '{profile}' under '{}' in {}{}",
        project.get_dialogs_prefix(),
        project.get_roots().describe(),
        Self::describe_unread(&partners)
      )));
    };

    let characters: Vec<&DialogCharacter> = partners.resolve_profile(profile);

    if characters.is_empty() {
      return Err(XrfError::new_not_found_error(format!(
        "Profile '{}' resolves to no character{}",
        profile.get_id(),
        Self::describe_unread(&partners)
      )));
    }

    let offers: IndexMap<String, Vec<DialogOffer>> = partners.list_offers(profile);
    let mut dialogs: Vec<DialogListEntry> = Vec::with_capacity(offers.len());

    for (id, offers) in offers {
      match Self::find_dialog(project, &id) {
        Some((file, dialog)) => dialogs.push(DialogListEntry::new(file, dialog, text, offers)),
        // The engine asserts on loading a dialog no file declares, so this is a defect of the tree, not of the list.
        None => findings.push(DialogFinding::new(
          "dialog.offer-missing",
          Some(id.clone()),
          format!("Dialog '{id}' is offered, but no dialog file declares it"),
        )),
      }
    }

    Ok(DialogListReport {
      language: text.map(|text| text.get_language().to_owned()),
      profile: Some(DialogListProfile {
        id: profile.get_id().to_owned(),
        characters: characters
          .iter()
          .map(|character| character.get_id().to_owned())
          .collect(),
      }),
      dialogs,
      findings,
    })
  }

  /// Which partner files could not be read, since a missing profile or character is most often declared in one.
  ///
  /// The list goes nowhere else: a refused profile deposits no report to carry its findings.
  fn describe_unread(partners: &DialogPartners) -> String {
    let unread: Vec<String> = partners
      .get_findings()
      .iter()
      .filter(|finding| finding.rule == DialogPartners::UNREADABLE_RULE)
      .map(|finding| format!("{} ({})", finding.subject.as_deref().unwrap_or("-"), finding.message))
      .collect();

    if unread.is_empty() {
      String::new()
    } else {
      format!(
        ", and {} partner files could not be read: {}",
        unread.len(),
        unread.join("; ")
      )
    }
  }

  /// The first declaration of a dialog id, as the engine reads it.
  fn find_dialog<'a>(project: &'a DialogProject, id: &str) -> Option<(&'a DialogProjectFile, &'a Dialog)> {
    let file: &DialogProjectFile = project.list_files_declaring(id).into_iter().next()?;

    file.get_file().find_dialog(id).map(|dialog| (file, dialog))
  }

  /// One dialog as a line: its id, where it is declared, how it opens, and how it reaches the actor.
  fn describe(entry: &DialogListEntry) -> String {
    let caption: &str = entry.caption.as_deref().or(entry.caption_key.as_deref()).unwrap_or("-");
    let offers: Vec<String> = entry
      .offers
      .iter()
      .map(|offer| match offer {
        DialogOffer::Start { character } => format!("start of {character}"),
        DialogOffer::Actor { character } => format!("actor dialog of {character}"),
        DialogOffer::Info { info } => format!("with info {info}"),
      })
      .collect();

    if offers.is_empty() {
      format!(
        "{} ({}, {} phrases): {caption}",
        entry.id, entry.logical_path, entry.phrases
      )
    } else {
      format!(
        "{} ({}, {} phrases, {}): {caption}",
        entry.id,
        entry.logical_path,
        entry.phrases,
        offers.join(", ")
      )
    }
  }
}

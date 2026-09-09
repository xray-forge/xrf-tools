use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_ltx::{LtxDocumentSource, LtxProject};
use xrf_ltx_inspect::{LtxInventory, LtxInventoryFile, LtxInventoryReader, LtxInventoryRole};
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use crate::commands::ltx::ltx_project_open::open_ltx_project;
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::ltx_dialect::LtxDialectArguments;

#[derive(Default)]
pub struct ListCommand;

impl GenericCommand for ListCommand {
  fn operation(&self) -> &'static str {
    "list"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("List the LTX configs a project holds, and the role each one plays")
      .arg(
        Arg::new("path")
          .help("Path to a folder with ltx files, or to a game installation root holding fsgame.ltx")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .with_ltx_dialect()
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let output: OutputOptions = context.get_output().clone();

    xrf_output::heading!(output, "List path: {}", format_path(path));

    let project: LtxProject = open_ltx_project(path, matches, &output)?;
    let source = project.document_source();
    let inventory: LtxInventory = LtxInventoryReader::new(&project, &source as &dyn LtxDocumentSource).read()?;

    for file in &inventory.files {
      xrf_output::info!(output, "{}", Self::describe(file, matches.get_flag("verbose")));
    }

    xrf_output::success!(output, "{}", Self::summarize(&inventory));

    context.set_result(|| &inventory)?;

    Ok(())
  }
}

impl ListCommand {
  /// One config as a line: what it is called, what it is to the project, and whether an editor could ever write it.
  fn describe(file: &LtxInventoryFile, is_verbose: bool) -> String {
    let role: String = match &file.role {
      LtxInventoryRole::EntryPoint => String::from("entry point"),
      LtxInventoryRole::SchemeFile => String::from("scheme"),
      LtxInventoryRole::Attachment => String::from("attachment"),
      LtxInventoryRole::Included { by } if by.is_empty() => String::from("included"),
      LtxInventoryRole::Included { by } => format!("included by {}", by.join(", ")),
    };

    // Said only where it is true, and said as a fact about writing rather than about storage: an archived config reads
    // exactly like a loose one and differs only in that nothing can edit it in place.
    let archived: &str = if file.is_physical { "" } else { " [archived]" };

    if is_verbose {
      format!("{} - {role}{archived} ({})", file.path, file.source)
    } else {
      format!("{} - {role}{archived}", file.path)
    }
  }

  /// The counts, so a large tree says what it is made of without being read line by line.
  fn summarize(inventory: &LtxInventory) -> String {
    let mut entries: usize = 0;
    let mut included: usize = 0;
    let mut schemes: usize = 0;
    let mut attachments: usize = 0;
    let mut archived: usize = 0;

    for file in &inventory.files {
      match file.role {
        LtxInventoryRole::EntryPoint => entries += 1,
        LtxInventoryRole::Included { .. } => included += 1,
        LtxInventoryRole::SchemeFile => schemes += 1,
        LtxInventoryRole::Attachment => attachments += 1,
      }

      if !file.is_physical {
        archived += 1;
      }
    }

    format!(
      "{} configs: {entries} entry points, {included} included, {schemes} schemes, {attachments} attachments, \
       {archived} archived",
      inventory.files.len()
    )
  }
}

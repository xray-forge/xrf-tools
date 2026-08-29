use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command, value_parser};
use xrf_archive::ArchiveProject;
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use super::report::ArchiveFindReport;
use crate::commands::archive::list::ListCommand;
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct FindCommand;

impl GenericCommand for FindCommand {
  fn operation(&self) -> &'static str {
    "find"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Find archive entries whose logical path contains text")
      .arg(
        Arg::new("path")
          .help("Path to an archive volume or a directory containing volumes")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("query")
          .help("Case-insensitive text to find in an entry's logical path")
          .short('q')
          .long("query")
          .required(true)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("files")
          .help("Search files only, excluding directory records")
          .long("files")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("directories")
          .help("Search directory records only, excluding files")
          .long("directories")
          .action(ArgAction::SetTrue),
      )
      .group(ArgGroup::new("entry-kind").args(["files", "directories"]))
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let started_at: Instant = Instant::now();
    let path: &PathBuf = matches
      .get_one("path")
      .expect("Expected an archive path to be provided");
    let query: &String = matches.get_one("query").expect("Expected a query to be provided");
    let output: OutputOptions = context.get_output().clone();

    let project: ArchiveProject = ArchiveProject::new(path)?;
    let query: String = query.to_ascii_lowercase();
    let entries = ListCommand::entries(&project, ListCommand::selection(matches));
    let entries: Vec<_> = entries
      .into_iter()
      .filter(|entry| entry.name.to_ascii_lowercase().contains(&query))
      .collect();

    for entry in &entries {
      let size: String = xrf_utils::format_bytes(u64::from(entry.size_real));

      if matches.get_flag("verbose") {
        let (compressed, unpacked): (String, String) =
          xrf_utils::format_bytes_pair(u64::from(entry.size_compressed), u64::from(entry.size_real));

        xrf_output::info!(
          output,
          "{} [{size}, {compressed} stored, {unpacked} unpacked, {}]",
          entry.name,
          format_path(&entry.source),
        );
      } else {
        xrf_output::info!(output, "{} [{size}]", entry.name);
      }
    }

    xrf_output::success!(
      output,
      "Found {} match{} in {}",
      entries.len(),
      if entries.len() == 1 { "" } else { "es" },
      xrf_utils::format_duration(started_at.elapsed())
    );

    context.set_result(|| ArchiveFindReport::new(&query, &entries))?;

    Ok(())
  }
}

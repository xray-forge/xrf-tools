use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_archive::ArchiveProject;
use xrf_output::OutputOptions;

use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Describe an X-Ray archive volume or volume set")
      .arg(
        Arg::new("path")
          .help("Path to an archive volume or a directory containing volumes")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("silent")
          .help("Disable any logging")
          .short('s')
          .long("silent")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("verbose")
          .help("Show every volume in the set")
          .short('v')
          .long("verbose")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let started_at: Instant = Instant::now();
    let path: &PathBuf = matches
      .get_one("path")
      .expect("Expected an archive path to be provided");
    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    let project: ArchiveProject = ArchiveProject::new(path)?;
    let file_count: usize = project.files.values().filter(|entry| !entry.is_directory).count();
    let directory_count: usize = project.files.len() - file_count;

    let (compressed, unpacked): (String, String) =
      xrf_utils::format_bytes_pair(project.get_compressed_size(), project.get_real_size());

    xrf_output::info!(output, "Archive root: {}", project.root.display());
    xrf_output::info!(output, "Volumes: {}", project.archives.len());
    xrf_output::info!(
      output,
      "Entries: {file_count} file(s), {directory_count} directory record(s)"
    );
    xrf_output::info!(output, "Size: {compressed} compressed, {unpacked} unpacked");

    if matches.get_flag("verbose") {
      for archive in &project.archives {
        let (compressed, unpacked): (String, String) =
          xrf_utils::format_bytes_pair(archive.get_compressed_size(), archive.get_real_size());

        xrf_output::info!(
          output,
          "  {}: {} entry(s), {compressed} compressed, {unpacked} unpacked, root {}",
          archive.path.display(),
          archive.files.len(),
          archive.output_root_path.display(),
        );
      }
    }

    xrf_output::success!(
      output,
      "Read archive information in {}",
      xrf_utils::format_duration(started_at.elapsed())
    );

    Ok(())
  }
}

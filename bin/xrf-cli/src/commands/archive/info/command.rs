use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_archive::ArchiveProject;
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use super::report::ArchiveInfoReport;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

#[derive(Default)]
pub struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  fn declare(&self, command: Command) -> Command {
    command.about("Describe an X-Ray archive volume or volume set").arg(
      Arg::new("path")
        .help("Path to an archive volume or a directory containing volumes")
        .short('p')
        .long("path")
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let started_at: Instant = Instant::now();
    let path: &PathBuf = matches
      .get_one("path")
      .expect("Expected an archive path to be provided");
    let output: OutputOptions = context.get_output().clone();

    let project: ArchiveProject = ArchiveProject::new(path)?;
    let (compressed, unpacked): (String, String) =
      xrf_utils::format_bytes_pair(project.get_compressed_size(), project.get_real_size());

    xrf_output::info!(output, "Archive root: {}", format_path(&project.root));
    xrf_output::info!(output, "Volumes: {}", project.archives.len());
    xrf_output::info!(
      output,
      "Entries: {} file(s), {} directory record(s)",
      project.count_files(),
      project.count_directories()
    );
    xrf_output::info!(output, "Size: {compressed} compressed, {unpacked} unpacked");

    for archive in &project.archives {
      let (compressed, unpacked): (String, String) =
        xrf_utils::format_bytes_pair(archive.get_compressed_size(), archive.get_real_size());

      xrf_output::verbose!(
        output,
        "  {}: {} entry(s), {compressed} compressed, {unpacked} unpacked, root {}",
        format_path(&archive.path),
        archive.entries,
        format_path(&archive.output_root_path),
      );
    }

    xrf_output::success!(
      output,
      "Read archive information in {}",
      xrf_utils::format_duration(started_at.elapsed())
    );

    context.set_result(|| ArchiveInfoReport::new(&project))?;

    Ok(())
  }
}

use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_db::{OmfFile, OmfMotionsProcessor, XRayByteOrder};
use xrf_output::OutputOptions;

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct DuplicateMotionCommand;

impl GenericCommand for DuplicateMotionCommand {
  fn operation(&self) -> &'static str {
    "duplicate-motion"
  }

  /// Create command for duplicating a motion of an omf file.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to copy a motion of provided omf file under a new name")
      .arg(
        Arg::new("path")
          .help("Path to omf file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to resulting omf file, defaults to in place rewrite of the source file")
          .short('d')
          .long("dest")
          .required(false)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("from")
          .help("Motion to copy, matched exactly")
          .long("from")
          .required(true),
      )
      .arg(Arg::new("to").help("Name to give the copy").long("to").required(true))
      .arg(
        Arg::new("play-once")
          .help("Clear looping on the copy so it plays once and ends")
          .long("play-once")
          .action(ArgAction::SetTrue),
      )
  }

  /// Copy a motion of an omf file under a new name.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let from: &String = matches
      .get_one::<String>("from")
      .expect("Expected valid source motion name to be provided");
    let to: &String = matches
      .get_one::<String>("to")
      .expect("Expected valid target motion name to be provided");

    let destination: &Path = matches
      .get_one::<PathBuf>("dest")
      .map_or(path.as_path(), |it| it.as_path());

    let play_once: bool = matches.get_flag("play-once");
    let output: OutputOptions = context.get_output().clone();

    let mut omf_file: OmfFile = OmfFile::read_from_path::<XRayByteOrder, _>(path)?;

    OmfMotionsProcessor::duplicate_motion(&mut omf_file, from, to, play_once)?;

    omf_file.write_to_path::<XRayByteOrder, _>(&destination)?;

    xrf_output::info!(
      output,
      "Copied motion '{}' to '{}'{}, written into {}",
      from,
      to,
      if play_once { ", playing once" } else { "" },
      destination.display()
    );

    Ok(())
  }
}

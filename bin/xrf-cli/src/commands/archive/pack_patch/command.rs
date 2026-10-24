use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_pack::{ArchivePatchResult, ArchivePatcher};

use crate::commands::archive::ArchiveVolumeArguments;
use crate::commands::archive::pack_patch::archive_patch_arguments::ArchivePatchArguments;
use crate::commands::archive::pack_patch::archive_patch_summary::ArchivePatchSummary;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

/// Options a configuration file also carries, so naming both would leave the winner unstated.
const SELECTION_ARGUMENTS: [&str; 4] = ["include", "ignore", "exclude-extension", "header"];

#[derive(Default)]
pub struct PackPatchCommand;

impl GenericCommand for PackPatchCommand {
  fn operation(&self) -> &'static str {
    "pack-patch"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about("Pack what a gamedata tree changes about an installation into overriding *.db archive volumes")
      .arg(
        Arg::new("path")
          .help("What to patch: an installation, a directory of volumes, or a gamedata tree")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("target")
          .help("Tree the patch delivers; omit to use the loose gamedata of the input itself")
          .long("target")
          .required(false)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dry-run")
          .help("Report the difference and write no volumes")
          .long("dry-run")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("config")
          .help("Path to a patching configuration describing the comparison scope and header, as *.ltx or *.json")
          .long("config")
          .required(false)
          .conflicts_with_all(SELECTION_ARGUMENTS)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("include")
          .help("Logical prefix the comparison is restricted to, such as configs, repeatable")
          .long("include")
          .required(false)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("ignore")
          .help("Logical prefix dropped from the comparison, repeatable")
          .long("ignore")
          .required(false)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("exclude-extension")
          .help("Extension pattern that keeps a file out of the comparison, such as *.txt, repeatable")
          .long("exclude-extension")
          .required(false)
          .action(ArgAction::Append)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("verify-payload")
          .help("Confirm every checksum match by comparing the payloads themselves")
          .long("verify-payload")
          .required(false)
          .action(ArgAction::SetTrue),
      );

    ArchiveVolumeArguments::declare(command, "patch")
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();
    let arguments: ArchivePatchArguments = ArchivePatchArguments::of(matches, output.clone())?;

    ArchivePatchSummary::describe_inputs(&output, &arguments.config, arguments.is_dry_run);

    // Which door runs is this flag's whole meaning, so it is spelled here rather than carried into the crate: the
    // patcher makes "does this write" a method name.
    let result: ArchivePatchResult = if arguments.is_dry_run {
      ArchivePatcher::compare_opt(&arguments.config, arguments.options)?
    } else {
      ArchivePatcher::patch_opt(&arguments.config, arguments.options)?
    };

    ArchivePatchSummary::describe_result(&output, &result);

    context.set_result(|| &result)?;

    Ok(())
  }
}

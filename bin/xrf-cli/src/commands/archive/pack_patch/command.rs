use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_pack::{ArchivePatchResult, ArchivePatcher, VOLUME_SIZE_MAX, VOLUME_SIZE_MIN};

use crate::commands::archive::pack_patch::archive_patch_arguments::ArchivePatchArguments;
use crate::commands::archive::pack_patch::archive_patch_summary::{describe_inputs, describe_result};
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

/// Options a configuration file also carries, so naming both would leave the winner unstated.
const SELECTION_ARGUMENTS: [&str; 4] = ["include", "ignore", "exclude-extension", "header"];

#[derive(Default)]
pub struct PackPatchCommand;

impl GenericCommand for PackPatchCommand {
  fn operation(&self) -> &'static str {
    "pack-patch"
  }

  /// Create command to pack the difference between two worlds as an overriding volume set.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to pack what a gamedata tree changes about an installation into overriding *.db archive volumes")
      .arg(
        Arg::new("source")
          .help("What to patch: an installation, a directory of volumes, or a gamedata tree")
          .required(true)
          .value_name("SOURCE")
          .num_args(1)
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
        Arg::new("dest")
          .help("Path to folder for writing the volumes")
          .short('d')
          .long("dest")
          .default_value("packed")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("name")
          .help("Base name of the volumes, written as <name>.db0, <name>.db1 and so on")
          .short('n')
          .long("name")
          .default_value("patch")
          .value_parser(value_parser!(String)),
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
        Arg::new("header")
          .help("Header entry written into the archive as <key>=<value>, repeatable, merged over the default header")
          .long("header")
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
      )
      .arg(
        Arg::new("store")
          .help("Store every file instead of compressing what the engine expects compressed")
          .long("store")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("max-size")
          .help(format!(
            "Maximum volume size in megabytes, from {} to {}",
            VOLUME_SIZE_MIN / xrf_utils::BYTES_PER_MEGABYTE,
            VOLUME_SIZE_MAX / xrf_utils::BYTES_PER_MEGABYTE
          ))
          .long("max-size")
          .required(false)
          .value_parser(value_parser!(u64).range(1..)),
      )
      .arg(
        Arg::new("oversized-volumes")
          .help(format!(
            "Let --max-size exceed {} MB, which only an engine fork that raised XRP_MAX_SIZE can mount",
            VOLUME_SIZE_MAX / xrf_utils::BYTES_PER_MEGABYTE
          ))
          .long("oversized-volumes")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("xdb")
          .help("Write volumes with the xdb extension")
          .long("xdb")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("force")
          .help("Replace volumes of the same set the destination already holds")
          .short('f')
          .long("force")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Compare two worlds and publish what changed.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();
    let arguments: ArchivePatchArguments = ArchivePatchArguments::of(matches, output.clone())?;

    describe_inputs(&output, &arguments.config, arguments.is_dry_run);

    // Which door runs is this flag's whole meaning, so it is spelled here rather than carried into the crate: the
    // patcher makes "does this write" a method name.
    let result: ArchivePatchResult = if arguments.is_dry_run {
      ArchivePatcher::compare_opt(&arguments.config, arguments.options)?
    } else {
      ArchivePatcher::patch_opt(&arguments.config, arguments.options)?
    };

    describe_result(&output, &result);

    context.set_result(|| &result)?;

    Ok(())
  }
}

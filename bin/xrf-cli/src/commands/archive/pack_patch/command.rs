use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_pack::{ArchivePatchResult, ArchivePatcher};

use crate::commands::archive::pack_patch::archive_patch_arguments::ArchivePatchArguments;
use crate::commands::archive::pack_patch::archive_patch_summary::{describe_inputs, describe_result};
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

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
        Arg::new("input")
          .help("What to patch: an installation, a directory of volumes, or a gamedata tree")
          .short('i')
          .long("input")
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
        Arg::new("release")
          .help("Read both sides as complete releases, so entries the target dropped are reported")
          .long("release")
          .required(false)
          .action(ArgAction::SetTrue),
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
          .help("Header entry written into the archive as <key>=<value>, repeatable, replacing the default header")
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
        Arg::new("strict")
          .help("Fail when the input holds entries the target dropped, which a patch cannot express; needs --release")
          .long("strict")
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
        Arg::new("xdb")
          .help("Write volumes with the *.xdb extension")
          .long("xdb")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("max-size")
          .help("Maximum volume size in megabytes, from 1 to 1900")
          .long("max-size")
          .required(false)
          .value_parser(value_parser!(u64)),
      )
      .arg(
        Arg::new("oversized-volumes")
          .help("Allow volumes past the size an unmodified engine mounts")
          .long("oversized-volumes")
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

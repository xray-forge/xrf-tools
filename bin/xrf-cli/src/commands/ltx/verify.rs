use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_ltx::{LtxProject, LtxProjectOptions, LtxProjectVerifyResult, LtxVerifyOptions};
use xrf_output::OutputOptions;
use xrf_vfs::XrayLookupScope;

use crate::commands::ltx::ltx_installation::mount_installation;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct VerifyCommand;

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  /// Add command for verifying of ltx files.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command for verification of ltx and ini files")
      .arg(
        Arg::new("path")
          .help("Path to a folder with ltx files, or to a game installation root holding fsgame.ltx")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("silent")
          .help("Turn off logging")
          .long("silent")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("verbose")
          .help("Turn on verbose logging")
          .short('v')
          .long("verbose")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Verify ltx file or folder based on provided arguments.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    if !path.is_dir() {
      return Err(
        XrfError::new_read_error("Expected configs root directory path for validation as --path parameter").into(),
      );
    }

    log::info!("Verifying ltx folder: {}", path.display());

    let options: LtxProjectOptions = LtxProjectOptions {
      is_with_schemes_check: true,
      is_strict_check: true,
    };

    // Verification only reads, so an installation is verified over every declared source. Narrowing it to loose configs
    // would leave archived includes unresolved and report their sections as missing, so the scope a person picks is the
    // path they name rather than a flag.
    let project: Box<LtxProject> = Box::new(match mount_installation(path)? {
      Some(vfs) => {
        let project: LtxProject = LtxProject::open_at_scope_opt(path, vfs, XrayLookupScope::all(), options)?;

        Self::report_sources(&project, &output);

        project
      }
      None => LtxProject::open_at_path_opt(path, options)?,
    });

    let result: LtxProjectVerifyResult = project.verify_entries_opt(LtxVerifyOptions { output })?;

    if result.errors.is_empty() {
      Ok(())
    } else {
      Err(CommandError::new_check_failed(result.errors.len()))
    }
  }
}

impl VerifyCommand {
  /// Reports the sources an installation's verification searched.
  ///
  /// An installation keeps nearly every config in an archive volume, so naming only the game directory would leave a report
  /// silent about where thousands of verified files came from.
  fn report_sources(project: &LtxProject, output: &OutputOptions) {
    for mount in project.vfs().scoped(project.scope()).list_mounts() {
      xrf_output::info!(
        output,
        "Source: {:?} {} ({})",
        mount.get_kind(),
        mount.get_source().get_root_path().display(),
        mount.get_label()
      );
    }
  }
}

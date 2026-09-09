use std::path::Path;

use clap::ArgMatches;
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxProject, LtxProjectOptions};
use xrf_output::OutputOptions;
use xrf_utils::format_path;
use xrf_vfs::XrayLookupScope;

use crate::commands::ltx::ltx_installation::mount_installation;
use crate::core::ltx_dialect::requested_ltx_dialect;

/// Opens the LTX project a command was pointed at, directory or installation.
///
/// Shared so the reading commands cannot drift from `ltx verify` about what a tree contains: an installation keeps
/// nearly every config in an archive volume, and a command that only walked the filesystem would answer about a
/// different set of files than the one that checks them.
///
/// # Errors
///
/// Returns an error when `path` is not a directory, an installation cannot be mounted, or the project cannot be
/// assembled.
pub fn open_ltx_project(path: &Path, matches: &ArgMatches, output: &OutputOptions) -> XrfResult<LtxProject> {
  if !path.is_dir() {
    return Err(XrfError::new_read_error(
      "Expected configs root directory path as --path parameter",
    ));
  }

  let options: LtxProjectOptions = LtxProjectOptions {
    dialect: requested_ltx_dialect(matches),
    is_with_schemes_check: true,
    is_strict_check: true,
  };

  match mount_installation(path)? {
    Some(vfs) => {
      let project: LtxProject = LtxProject::open_at_scope_opt(path, vfs, XrayLookupScope::all(), options)?;

      report_sources(&project, output);

      Ok(project)
    }
    None => LtxProject::open_at_path_opt(path, options),
  }
}

/// Reports the sources an installation-backed project reads through.
///
/// An installation keeps nearly every config in an archive volume, so naming only the game directory would leave the
/// answer silent about where thousands of configs came from.
fn report_sources(project: &LtxProject, output: &OutputOptions) {
  for mount in project.vfs().scoped(project.scope()).list_mounts() {
    xrf_output::info!(
      output,
      "Source: {:?} {} ({})",
      mount.get_kind(),
      format_path(mount.get_source().get_root_path()),
      mount.get_label()
    );
  }
}

use std::path::PathBuf;
use std::time::Instant;

use xrf_error::XrfResult;
use xrf_vfs::require_writable_path;

use crate::project::ltx_files_formatter::LtxFilesFormatter;
use crate::project::ltx_project_format_result::LtxProjectFormatResult;
use crate::{Ltx, LtxFormatOptions, LtxProject};

impl LtxProject {
  /// Formats every project LTX file with explicit options.
  ///
  /// Returns an error instead of partially formatting a project when any config is archived.
  pub fn format_all_files_opt(&self, options: LtxFormatOptions) -> XrfResult<LtxProjectFormatResult> {
    LtxFilesFormatter::format_opt(&self.writable_files()?, options)
  }

  /// Checks every project LTX file with explicit options, reading each one through the project.
  ///
  /// Unlike [`Self::format_all_files_opt`] this accepts archived configs: a formatting verdict needs content, and only
  /// rewriting needs a file. Findings name the loose path where there is one and the engine identity otherwise, so an
  /// installation reports every config it holds rather than refusing the whole check on the first archived one.
  ///
  /// # Errors
  ///
  /// Returns an error when a config cannot be read or parsed.
  pub fn check_format_all_files_opt(&self, options: LtxFormatOptions) -> XrfResult<LtxProjectFormatResult> {
    let mut result: LtxProjectFormatResult = LtxProjectFormatResult::new();
    let started_at: Instant = Instant::now();

    xrf_output::heading!(options.output, "Checking {} file(s)", self.ltx_files.len());

    for logical_path in &self.ltx_files {
      let contents: Vec<u8> = self.vfs().scoped(self.scope()).read_bytes(logical_path.as_str())?;

      result.record_checked(self.path_of(logical_path), Ltx::is_formatted(&contents)?, &options);
    }

    result.duration = started_at.elapsed();

    LtxFilesFormatter::report_check(&result, &options);

    Ok(result)
  }

  /// Returns physical paths for every project file, refusing when one is not loose.
  ///
  /// Formatting rewrites a file in place, which an archived config cannot do. Refusing by name is deliberate: a project
  /// spanning an installation would otherwise format the loose handful and report success over thousands it never touched.
  /// A caller that needs the loose subset must select it explicitly.
  fn writable_files(&self) -> XrfResult<Vec<PathBuf>> {
    let mut writable: Vec<PathBuf> = Vec::with_capacity(self.ltx_files.len());

    for logical_path in &self.ltx_files {
      writable.push(require_writable_path(
        logical_path.as_str(),
        self.physical_path_of(logical_path),
      )?);
    }

    Ok(writable)
  }

  /// Formats every project LTX file with default options.
  pub fn format_all_files(&self) -> XrfResult<LtxProjectFormatResult> {
    self.format_all_files_opt(LtxFormatOptions::default())
  }

  /// Checks every project LTX file with default options.
  pub fn check_format_all_files(&self) -> XrfResult<LtxProjectFormatResult> {
    self.check_format_all_files_opt(LtxFormatOptions::default())
  }
}

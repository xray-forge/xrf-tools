use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_job::{JobOutcome, JobScope};
use xrf_utils::encode_w1251_bytes_to_string;
use xrf_vfs::require_writable_path;

use crate::project::ltx_files_formatter::LtxFilesFormatter;
use crate::project::ltx_format_options::LTX_PHASE_CHECK;
use crate::project::ltx_project_format_result::LtxProjectFormatResult;
use crate::{LtxFormatOptions, LtxProject};

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

    xrf_output::heading!(options.output, "Checking {} file(s)", self.ltx_files.len());

    // Its own loop rather than `LtxFilesFormatter::check_format_opt`, because it reads through the project's virtual
    // filesystem where that one reads host files. The two therefore have to be instrumented separately.
    result.startup_duration = options.job.elapsed();

    let checking: JobScope = options.job.enter(LTX_PHASE_CHECK, Some(self.ltx_files.len() as u64));

    for logical_path in &self.ltx_files {
      // A stopped check reports what it had judged so far, and says so: findings read as a complete verdict otherwise.
      if options.job.is_cancelled() {
        result.outcome = JobOutcome::Cancelled;

        break;
      }

      // Two reads of one config, and the second is not redundant: judging formatting means comparing the bytes as
      // authored against the canonical rendering, and only the bytes can answer the first half. The rendering comes
      // from the document the project already holds, so this pass parses nothing.
      let contents: Vec<u8> = self.read_counted_bytes(logical_path)?;
      let formatted: bool =
        encode_w1251_bytes_to_string(&contents)? == self.read_document(logical_path)?.to_formatted();

      result.record_checked(self.path_of(logical_path), formatted, &options);
      checking.advance();
    }

    result.duration = options.job.elapsed();

    LtxFilesFormatter::report_check(&result, &options);

    Ok(result)
  }

  /// Returns physical paths for every project file, refusing when one is not loose.
  ///
  /// Formatting rewrites a file in place, which an archived config cannot do. Refusing by name is intentional: a project
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

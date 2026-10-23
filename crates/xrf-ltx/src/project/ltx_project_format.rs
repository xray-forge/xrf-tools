use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_job::{JobOutcome, JobScope};
use xrf_utils::{encode_string_to_w1251_bytes, encode_w1251_bytes_to_string, format_path};
use xrf_vfs::{XrayTransaction, require_writable_path};

use crate::ltx::Ltx;
use crate::project::{
  LTX_PHASE_CHECK, LTX_PHASE_FORMAT, LtxFilesFormatter, LtxFormatOptions, LtxProject, LtxProjectFormatResult,
};

impl LtxProject {
  /// Formats every project LTX file with explicit options, rewriting the ones not formatted yet in one commit of the
  /// project's own snapshot, so the files hold all of the run or none of it.
  ///
  /// # Errors
  ///
  /// As [`Self::format_all_files_with`], or when the rewrite cannot be committed.
  pub fn format_all_files_opt(&self, options: LtxFormatOptions) -> XrfResult<LtxProjectFormatResult> {
    self.format_all_files_with(options, |rewrites| {
      let transaction: XrayTransaction = rewrites
        .into_iter()
        .fold(XrayTransaction::new(self.search()), |transaction, (file, bytes)| {
          transaction.with_file_write(file, bytes)
        });

      self.search().get_vfs().commit(&transaction).map(drop)
    })
  }

  /// Formats every project LTX file with explicit options, handing the rewrites to `write` at once, each as the file it
  /// replaces and its formatted bytes, where there are any, for a caller committing the run as one change of its own.
  /// A cancelled run hands over what it formatted before it stopped.
  ///
  /// # Errors
  ///
  /// Returns an error, with nothing handed over, when any config is archived (a project spanning an installation would
  /// otherwise format the loose handful and report success), cannot be read or will not parse; or what `write` answers.
  pub fn format_all_files_with(
    &self,
    options: LtxFormatOptions,
    write: impl FnOnce(Vec<(PathBuf, Vec<u8>)>) -> XrfResult,
  ) -> XrfResult<LtxProjectFormatResult> {
    let files: Vec<PathBuf> = self.writable_files()?;
    let mut result: LtxProjectFormatResult = LtxProjectFormatResult::new();
    let mut rewrites: Vec<(PathBuf, Vec<u8>)> = Vec::new();

    xrf_output::heading!(options.output, "Formatting {} file(s)", files.len());

    result.startup_duration = options.job.elapsed();

    let formatting: JobScope = options.job.enter(LTX_PHASE_FORMAT, Some(files.len() as u64));

    for (logical_path, file) in self.ltx_files.iter().zip(files) {
      // Between files, never inside one.
      if options.job.is_cancelled() {
        result.outcome = JobOutcome::Cancelled;

        break;
      }

      options.job.set_detail(Some(format_path(&file).to_string()));

      let existing: String = encode_w1251_bytes_to_string(&self.read_counted_bytes(logical_path)?)?;
      let formatted: String = Ltx::format_from_str(&existing)?;

      if existing == formatted {
        result.valid_files += 1;
      } else {
        xrf_output::info!(options.output, "Formatted: {}", format_path(&file));

        result.invalid_files += 1;
        result.to_format.push(file.clone());
        // The read decodes Windows-1251, so the rewrite goes back in that encoding.
        rewrites.push((file, encode_string_to_w1251_bytes(&formatted)?));
      }

      result.total_files += 1;
      formatting.advance();
    }

    options.job.set_detail(None);

    if !rewrites.is_empty() {
      write(rewrites)?;
    }

    result.duration = options.job.elapsed();

    xrf_output::info!(
      options.output,
      "Formatted {}/{} files in {}",
      result.invalid_files,
      result.total_files,
      xrf_utils::format_duration(result.duration)
    );

    Ok(result)
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

  /// Returns physical paths for every project file, in `ltx_files` order, refusing when one is not loose: formatting
  /// rewrites a file in place, which an archived config cannot do.
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

use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_job::{JobOutcome, JobScope};
use xrf_utils::format_path;

use crate::ltx::Ltx;
use crate::project::{LTX_PHASE_CHECK, LTX_PHASE_FORMAT, LtxFormatOptions, LtxProjectFormatResult};

/// Formatter of arbitrary sets of LTX files.
pub struct LtxFilesFormatter {}

impl LtxFilesFormatter {
  /// Format provided LTX files, rewriting the ones that are not formatted yet.
  ///
  /// # Errors
  ///
  /// Returns an error when a file cannot be read, parsed, or rewritten.
  pub fn format_opt(files: &[PathBuf], options: LtxFormatOptions) -> XrfResult<LtxProjectFormatResult> {
    let mut result: LtxProjectFormatResult = LtxProjectFormatResult::new();

    xrf_output::heading!(options.output, "Formatting {} file(s)", files.len());

    result.startup_duration = options.job.elapsed();

    let formatting: JobScope = options.job.enter(LTX_PHASE_FORMAT, Some(files.len() as u64));

    for file in files {
      // Between files, never inside one.
      if options.job.is_cancelled() {
        result.outcome = JobOutcome::Cancelled;

        break;
      }

      // Sequential, so naming the file being rewritten is meaningful rather than whichever one a worker happens to
      // hold.
      options.job.set_detail(Some(format_path(file).to_string()));

      if Ltx::format_file(file, true)? {
        result.invalid_files += 1;
        result.to_format.push(file.clone());

        xrf_output::info!(options.output, "Formatted: {}", format_path(file));
      } else {
        result.valid_files += 1;
      }

      result.total_files += 1;
      formatting.advance();
    }

    options.job.set_detail(None);

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

  /// Check format of provided LTX files without rewriting any of them.
  ///
  /// # Errors
  ///
  /// Returns an error when a file cannot be read or parsed.
  pub fn check_format_opt(files: &[PathBuf], options: LtxFormatOptions) -> XrfResult<LtxProjectFormatResult> {
    let mut result: LtxProjectFormatResult = LtxProjectFormatResult::new();

    xrf_output::heading!(options.output, "Checking {} file(s)", files.len());

    result.startup_duration = options.job.elapsed();

    let checking: JobScope = options.job.enter(LTX_PHASE_CHECK, Some(files.len() as u64));

    for file in files {
      // A stopped check reports what it had judged so far, and says so: findings read as a complete verdict otherwise.
      if options.job.is_cancelled() {
        result.outcome = JobOutcome::Cancelled;

        break;
      }

      result.record_checked(file.clone(), Ltx::is_formatted(&fs::read(file)?)?, &options);
      checking.advance();
    }

    result.duration = options.job.elapsed();

    Self::report_check(&result, &options);

    Ok(result)
  }

  /// Format provided LTX files with default options.
  pub fn format(files: &[PathBuf]) -> XrfResult<LtxProjectFormatResult> {
    Self::format_opt(files, LtxFormatOptions::default())
  }

  /// Check format of provided LTX files with default options.
  pub fn check_format(files: &[PathBuf]) -> XrfResult<LtxProjectFormatResult> {
    Self::check_format_opt(files, LtxFormatOptions::default())
  }
}

impl LtxFilesFormatter {
  /// Report resulting statistics of a check run.
  ///
  /// Shared with [`crate::LtxProject::check_format_all_files_opt`], which reads its configs through a VFS rather than from
  /// files but reports the same way.
  pub(crate) fn report_check(result: &LtxProjectFormatResult, options: &LtxFormatOptions) {
    let duration: String = xrf_utils::format_duration(result.duration);

    if result.invalid_files == 0 {
      xrf_output::success!(
        options.output,
        "All {} files are formatted, checked in {}",
        result.total_files,
        duration
      );
    } else {
      xrf_output::warning!(
        options.output,
        "Format issues with {}/{} files in {}",
        result.invalid_files,
        result.total_files,
        duration
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_error::XrfResult;
  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use crate::project::LtxFilesFormatter;
  use crate::project::LtxProjectFormatResult;

  fn create_root(name: &str) -> XrfResult<PathBuf> {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("format/{name}"));

    if root.exists() {
      fs::remove_dir_all(&root)?;
    }

    fs::create_dir_all(&root)?;

    Ok(root)
  }

  #[test]
  fn formats_only_provided_files() -> XrfResult {
    let root: PathBuf = create_root("provided")?;
    let first: PathBuf = root.join("first.ltx");
    let second: PathBuf = root.join("second.ltx");

    fs::write(&first, "[a]\nkey=value\n")?;
    fs::write(&second, "[b]\nkey=value\n")?;

    let result: LtxProjectFormatResult = LtxFilesFormatter::format(std::slice::from_ref(&first))?;

    assert_eq!(result.total_files, 1);
    assert_eq!(result.invalid_files, 1);
    assert_eq!(result.to_format, vec![first.clone()]);

    assert_eq!(fs::read_to_string(&first)?, "[a]\r\nkey = value\r\n");
    assert_eq!(fs::read_to_string(&second)?, "[b]\nkey=value\n");

    fs::remove_dir_all(root)?;

    Ok(())
  }

  #[test]
  fn reports_already_formatted_files_as_valid() -> XrfResult {
    let root: PathBuf = create_root("valid")?;
    let file: PathBuf = root.join("formatted.ltx");

    fs::write(&file, "[a]\r\nkey = value\r\n")?;

    let result: LtxProjectFormatResult = LtxFilesFormatter::format(&[file])?;

    assert_eq!(result.total_files, 1);
    assert_eq!(result.valid_files, 1);
    assert_eq!(result.invalid_files, 0);
    assert!(result.to_format.is_empty());

    fs::remove_dir_all(root)?;

    Ok(())
  }

  #[test]
  fn check_does_not_write_files() -> XrfResult {
    let root: PathBuf = create_root("check")?;
    let file: PathBuf = root.join("unformatted.ltx");

    fs::write(&file, "[a]\nkey=value\n")?;

    let result: LtxProjectFormatResult = LtxFilesFormatter::check_format(std::slice::from_ref(&file))?;

    assert_eq!(result.total_files, 1);
    assert_eq!(result.invalid_files, 1);
    assert_eq!(result.to_format, vec![file.clone()]);
    assert_eq!(fs::read_to_string(&file)?, "[a]\nkey=value\n");

    fs::remove_dir_all(root)?;

    Ok(())
  }

  #[test]
  fn formats_file_with_unresolvable_inherit_and_include() -> XrfResult {
    let root: PathBuf = create_root("standalone")?;
    let file: PathBuf = root.join("standalone.ltx");

    fs::write(
      &file,
      "#include \"missing\\absent.ltx\"\n[af_custom]:af_base\ncost=100\n",
    )?;

    let result: LtxProjectFormatResult = LtxFilesFormatter::format(std::slice::from_ref(&file))?;

    assert_eq!(result.total_files, 1);
    assert_eq!(
      fs::read_to_string(&file)?,
      "#include \"missing\\absent.ltx\"\r\n\r\n[af_custom]:af_base\r\ncost = 100\r\n"
    );

    fs::remove_dir_all(root)?;

    Ok(())
  }

  #[test]
  fn handles_empty_file_list() -> XrfResult {
    let result: LtxProjectFormatResult = LtxFilesFormatter::format(&[])?;

    assert_eq!(result.total_files, 0);
    assert_eq!(result.invalid_files, 0);
    assert!(result.to_format.is_empty());

    Ok(())
  }
}

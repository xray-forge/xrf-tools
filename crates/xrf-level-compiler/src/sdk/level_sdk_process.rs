use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant, SystemTime};

use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;
use xrf_utils::format_path;

use crate::sdk::level_sdk_program::LevelSdkProgram;
use crate::sdk::level_sdk_workspace::LevelSdkWorkspace;

/// One run of an SDK compiler in a workspace, watched until it exits: its log's last line is the job's detail, and a
/// cancelled job kills it.
pub(crate) struct LevelSdkProcess;

impl LevelSdkProcess {
  /// How often the run is looked at: its exit, the job's cancel, and its log.
  const POLL: Duration = Duration::from_millis(250);

  /// Runs `program` for the level `level` in `workspace` and waits for it.
  ///
  /// # Errors
  ///
  /// Returns an error when the program does not start, exits with a failure (its log's last line named), or the job is
  /// cancelled, in which case it is killed first.
  pub(crate) fn run(
    program: &LevelSdkProgram,
    level: &str,
    workspace: &LevelSdkWorkspace,
    job: &JobHandle,
  ) -> XrfResult<Duration> {
    let started: Instant = Instant::now();
    let since: SystemTime = SystemTime::now();
    let mut command: Command = Command::new(&program.path);

    command
      .args(program.get_arguments(level))
      .current_dir(&workspace.directory)
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null());

    let mut child: Child = command.spawn().map_err(|error| {
      XrfError::new_io_error(
        format!("{} did not start: {error}", format_path(&program.path)),
        error.kind(),
      )
    })?;

    log::info!("Running {} -f {level}", format_path(&program.path));

    let status: ExitStatus = loop {
      if let Some(status) = child.try_wait().map_err(|error| {
        XrfError::new_io_error(
          format!("{} was not waited for: {error}", format_path(&program.path)),
          error.kind(),
        )
      })? {
        break status;
      }

      if job.is_cancelled() {
        // The compiler stops where it stands; what it wrote so far stays, as an SDK run stopped by hand leaves it.
        let _ = child.kill();
        let _ = child.wait();

        return Err(XrfError::new_cancelled_error(format!(
          "{} was stopped",
          format_path(&program.path)
        )));
      }

      job.set_detail(workspace.get_last_log_line(program.log_prefix, since));
      job.report_if_due();
      std::thread::sleep(Self::POLL);
    };

    if !status.success() {
      return Err(XrfError::new_invalid_error(format!(
        "{} failed ({status}); its log ends: {}",
        format_path(&program.path),
        workspace
          .get_last_log_line(program.log_prefix, since)
          .unwrap_or_else(|| String::from("(no log)"))
      )));
    }

    Ok(started.elapsed())
  }
}

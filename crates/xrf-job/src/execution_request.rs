use std::num::NonZeroUsize;
use std::str::FromStr;
use std::thread;

use xrf_error::{XrfError, XrfResult};

use crate::execution_origin::ExecutionOrigin;
use crate::execution_plan::ExecutionPlan;

/// How much of the machine a caller asked for, before the machine has been consulted.
///
/// Kept apart from [`ExecutionPlan`] because the two answer different questions and are known at different moments. A
/// request is what a command line said and can be parsed, printed and tested without a host; a plan is what that means
/// here, and cannot exist until the host has been asked. Resolving early would make `auto` a number nobody could trace
/// back to the word that produced it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionRequest {
  /// Whatever this host offers.
  Auto,
  /// Exactly this many workers.
  Workers(NonZeroUsize),
  /// This percentage of what the host offers, so one setting suits machines of different sizes.
  Share(NonZeroUsize),
}

impl ExecutionRequest {
  /// The largest worker count a caller may name.
  ///
  /// A guard against a typo, not against a machine: it exists so `--jobs 100000` is a usage error rather than an
  /// attempt to start a hundred thousand threads. Deliberately a fixed number rather than a multiple of what the host
  /// reports, because a host-derived limit would make one command line valid on one machine and a usage error on
  /// another, which breaks a reproducible bug report and a recorded test alike.
  ///
  /// [`Self::Auto`] and [`Self::Share`] are not subject to it. Neither can exceed the machine, so neither can be the
  /// typo this guards against, and capping them would silently under-use a host larger than the constant.
  pub const MAX_WORKERS: usize = 1024;

  /// Decides what this request means on this host.
  pub fn resolve(&self) -> ExecutionPlan {
    match self {
      Self::Auto => ExecutionPlan::new(Self::available_parallelism(), ExecutionOrigin::Auto),
      Self::Workers(workers) => ExecutionPlan::new(*workers, ExecutionOrigin::Requested),
      Self::Share(percent) => ExecutionPlan::new(Self::share_of_host(*percent), ExecutionOrigin::Requested),
    }
  }

  /// What the host offers, or one worker where it will not say.
  ///
  /// Uncapped, and the same answer Rayon reaches on its own, so asking for `auto` changes how work is bounded without
  /// changing how wide it runs.
  fn available_parallelism() -> NonZeroUsize {
    thread::available_parallelism().unwrap_or(NonZeroUsize::MIN)
  }

  /// A percentage of the host, rounded down, never rounded away, and never larger than the host.
  ///
  /// Down rather than to nearest, so a share is a promise not to exceed a fraction of the machine. The floor of one
  /// keeps `1%` of a four-core host a runnable plan instead of a pool with no workers in it.
  ///
  /// The ceiling of one host is what makes a share mean the same thing however it was built. Reading a command line
  /// rejects anything above `100%` with a message, but the variant is constructible directly, and a share that resolved
  /// to several times the machine would be oversubscription nobody asked for. Naming the count outright is how a caller
  /// asks for that.
  fn share_of_host(percent: NonZeroUsize) -> NonZeroUsize {
    let host: NonZeroUsize = Self::available_parallelism();
    let workers: usize = (host.get().saturating_mul(percent.get()) / 100).min(host.get());

    NonZeroUsize::new(workers).unwrap_or(NonZeroUsize::MIN)
  }

  fn parse_workers(text: &str) -> XrfResult<Self> {
    match text.parse::<usize>() {
      Ok(0) => Err(XrfError::new_invalid_error("worker count must be at least 1")),
      Ok(workers) if workers > Self::MAX_WORKERS => Err(XrfError::new_invalid_error(format!(
        "worker count must be at most {}, got {workers}",
        Self::MAX_WORKERS
      ))),
      Ok(workers) => Ok(Self::Workers(
        NonZeroUsize::new(workers).expect("zero is rejected above"),
      )),
      Err(_) => Err(Self::unreadable(text)),
    }
  }

  fn parse_share(text: &str) -> XrfResult<Self> {
    match text.parse::<usize>() {
      Ok(0) => Err(XrfError::new_invalid_error("share must be at least 1%")),
      Ok(percent) if percent > 100 => Err(XrfError::new_invalid_error(format!(
        "share must be at most 100%, got {percent}%"
      ))),
      Ok(percent) => Ok(Self::Share(NonZeroUsize::new(percent).expect("zero is rejected above"))),
      Err(_) => Err(Self::unreadable(text)),
    }
  }

  fn unreadable(text: &str) -> XrfError {
    XrfError::new_invalid_error(format!(
      "expected `auto`, a worker count such as `4`, or a share such as `50%`, got `{text}`"
    ))
  }
}

impl FromStr for ExecutionRequest {
  type Err = XrfError;

  /// Reads `auto`, a worker count, or a percentage.
  ///
  /// There is no `sequential` spelling. It could only mean a one-worker pool, which is what `1` already says, and a
  /// second spelling of one plan would leave a report having to pick which of them to print back.
  ///
  /// # Errors
  ///
  /// Returns an error naming what was accepted and what arrived, because this is read from a command line where the
  /// next thing the reader does is retype it.
  fn from_str(text: &str) -> XrfResult<Self> {
    let text: &str = text.trim();

    if text.eq_ignore_ascii_case("auto") {
      return Ok(Self::Auto);
    }

    match text.strip_suffix('%') {
      Some(percent) => Self::parse_share(percent.trim_end()),
      None => Self::parse_workers(text),
    }
  }
}

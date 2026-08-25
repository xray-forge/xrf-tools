use std::fmt::Display;
use std::sync::Arc;

use crate::{NoopOutput, Output, OutputChannel, OutputRecord};

/// Controls which live workflow messages are rendered.
///
/// Verbosity gates chatter only: `Silent` mutes headings, successes, warnings, info, and verbose
/// detail, while `error` and `failure` always reach the output so a failing run is never silent.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OutputVerbosity {
  #[default]
  Silent,
  Normal,
  Verbose,
}

/// Runtime output configuration for a workflow.
#[derive(Clone)]
pub struct OutputOptions {
  output: Arc<dyn Output>,
  verbosity: OutputVerbosity,
}

impl OutputOptions {
  pub fn new(output: Arc<dyn Output>, verbosity: OutputVerbosity) -> Self {
    Self { output, verbosity }
  }

  /// The same verbosity, writing into `output` instead.
  ///
  /// Gating stays with the verbosity rather than the sink, so a message muted by the original is
  /// muted here too and is never rendered at all.
  pub fn with_output(&self, output: Arc<dyn Output>) -> Self {
    Self::new(output, self.verbosity)
  }

  pub fn heading(&self, message: impl Display) {
    self.write(OutputChannel::Heading, &message);
  }

  pub fn success(&self, message: impl Display) {
    self.write(OutputChannel::Success, &message);
  }

  pub fn warning(&self, message: impl Display) {
    self.write(OutputChannel::Warning, &message);
  }

  pub fn failure(&self, message: impl Display) {
    self.write(OutputChannel::Failure, &message);
  }

  pub fn info(&self, message: impl Display) {
    self.write(OutputChannel::Info, &message);
  }

  pub fn error(&self, message: impl Display) {
    self.write(OutputChannel::Error, &message);
  }

  pub fn verbose(&self, message: impl Display) {
    self.write(OutputChannel::Verbose, &message);
  }

  /// Sends one message on `channel`, when this verbosity renders that channel at all.
  pub fn write(&self, channel: OutputChannel, message: &dyn Display) {
    if channel.is_visible_at(self.verbosity) {
      self.output.write(channel, message);
    }
  }

  /// Sends a message captured earlier.
  ///
  /// Replaying a record does not re-decide whether it is visible: it was recorded through the same
  /// verbosity that renders it, so anything muted was never recorded in the first place.
  pub fn write_record(&self, record: &OutputRecord) {
    self.output.write(record.get_channel(), &record.get_message());
  }

  pub const fn get_verbosity(&self) -> OutputVerbosity {
    self.verbosity
  }
}

impl Default for OutputOptions {
  fn default() -> Self {
    Self::new(Arc::new(NoopOutput), OutputVerbosity::Silent)
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use super::{OutputOptions, OutputVerbosity};
  use crate::{OutputChannel, OutputRecord, RecordingOutput};

  fn messages(output: &RecordingOutput) -> Vec<String> {
    output
      .list_records()
      .into_iter()
      .map(|record| format!("{:?}:{}", record.get_channel(), record.get_message()))
      .collect()
  }

  #[test]
  fn filters_messages_by_verbosity() {
    let output: Arc<RecordingOutput> = Arc::new(RecordingOutput::default());
    let options: OutputOptions = OutputOptions::new(output.clone(), OutputVerbosity::Normal);

    options.heading("heading");
    options.success("success");
    options.warning("warning");
    options.failure("failure");
    options.info("normal");
    options.error("error");
    options.verbose("verbose");

    assert_eq!(
      messages(&output),
      vec![
        String::from("Heading:heading"),
        String::from("Success:success"),
        String::from("Warning:warning"),
        String::from("Failure:failure"),
        String::from("Info:normal"),
        String::from("Error:error"),
      ]
    );
  }

  #[test]
  fn silent_verbosity_mutes_chatter_but_never_failures() {
    let output: Arc<RecordingOutput> = Arc::new(RecordingOutput::default());
    let options: OutputOptions = OutputOptions::new(output.clone(), OutputVerbosity::Silent);

    options.heading("heading");
    options.success("success");
    options.warning("warning");
    options.failure("failure");
    options.info("normal");
    options.error("error");
    options.verbose("verbose");

    assert_eq!(
      messages(&output),
      vec![String::from("Failure:failure"), String::from("Error:error")]
    );
  }

  #[test]
  fn forwards_every_message_at_verbose_verbosity() {
    let output: Arc<RecordingOutput> = Arc::new(RecordingOutput::default());
    let options: OutputOptions = OutputOptions::new(output.clone(), OutputVerbosity::Verbose);

    options.info("normal");
    options.error("error");
    options.verbose("verbose");

    assert_eq!(
      messages(&output),
      vec![
        String::from("Info:normal"),
        String::from("Error:error"),
        String::from("Verbose:verbose"),
      ]
    );
  }

  #[test]
  fn replays_a_record_without_re_deciding_visibility() {
    let output: Arc<RecordingOutput> = Arc::new(RecordingOutput::default());
    let options: OutputOptions = OutputOptions::new(output.clone(), OutputVerbosity::Silent);

    options.write_record(&OutputRecord::new(OutputChannel::Verbose, "detail"));

    assert_eq!(messages(&output), vec![String::from("Verbose:detail")]);
  }

  #[test]
  fn keeps_its_verbosity_when_redirected_to_another_sink() {
    let first: Arc<RecordingOutput> = Arc::new(RecordingOutput::default());
    let second: Arc<RecordingOutput> = Arc::new(RecordingOutput::default());
    let options: OutputOptions = OutputOptions::new(first.clone(), OutputVerbosity::Normal);
    let redirected: OutputOptions = options.with_output(second.clone());

    redirected.info("normal");
    redirected.verbose("verbose");

    assert_eq!(redirected.get_verbosity(), OutputVerbosity::Normal);
    assert!(messages(&first).is_empty());
    assert_eq!(messages(&second), vec![String::from("Info:normal")]);
  }
}

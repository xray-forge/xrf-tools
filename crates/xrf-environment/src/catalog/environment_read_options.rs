use xrf_job::JobHandle;

/// How an environment read behaves.
#[derive(Clone, Default)]
pub struct EnvironmentReadOptions {
  /// Where cancellation comes from.
  pub job: JobHandle,
  /// Whether each section records where its keys' values came from, for an editor to write an edit back to. Resolves
  /// every config afresh rather than through the project's cache, so a caller that only checks leaves it off.
  pub is_explained: bool,
}

impl EnvironmentReadOptions {
  pub fn with_job(mut self, job: JobHandle) -> Self {
    self.job = job;
    self
  }

  pub fn with_explained(mut self, is_explained: bool) -> Self {
    self.is_explained = is_explained;
    self
  }
}

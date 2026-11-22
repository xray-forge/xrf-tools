use xrf_app_core::TauriResult;
use xrf_app_core::logging::UiLog;

use crate::ui_log::UiLogRecord;

/// A batch of the webview's records in the order it logged them, never more than [`Self::MAX_RECORDS`].
pub(crate) struct UiLogBatch {
  records: Vec<UiLogRecord>,
}

impl UiLogBatch {
  /// The most records one call carries: the webview batches its log so a burst costs a few calls, not one a line.
  pub(crate) const MAX_RECORDS: usize = 512;

  /// The batch of `records`.
  ///
  /// # Errors
  ///
  /// Refuses a batch over [`Self::MAX_RECORDS`] whole.
  pub(crate) fn new(records: Vec<UiLogRecord>) -> TauriResult<Self> {
    match records.len() > Self::MAX_RECORDS {
      true => Err(format!(
        "A UI log batch of {} records is over the {} one call takes",
        records.len(),
        Self::MAX_RECORDS
      )),
      false => Ok(Self { records }),
    }
  }

  /// Writes the records to `log` in their order, each filtered again as it is written: the webview's own filtering
  /// only saves the call.
  ///
  /// # Errors
  ///
  /// Returns the file's error where it refuses a line, the records before it written.
  pub(crate) fn write_to(&self, log: &UiLog) -> TauriResult {
    for record in &self.records {
      log
        .write(&record.as_entry())
        .map_err(|error| format!("Could not write the UI log: {error}"))?;
    }

    Ok(())
  }
}

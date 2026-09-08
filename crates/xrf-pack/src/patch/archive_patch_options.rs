use xrf_job::JobHandle;
use xrf_output::OutputOptions;

/// Phase a patch reports while it mounts both sides and decides what differs.
pub const PATCH_PHASE_COMPARE: &str = "compare";

/// Phase a patch reports while it writes the difference into volumes.
///
/// Its own name rather than the pack's `write`, because the two are denominated differently: comparing counts the
/// target's entries and packing counts only the changed ones, so a single bar over both would race at the handover.
pub const PATCH_PHASE_PACK: &str = "pack";

/// How one patch run should behave, beyond the configuration describing what to compare.
///
/// Whether the run writes is deliberately absent. That is the difference between
/// [`ArchivePatcher::compare`](crate::ArchivePatcher::compare) and
/// [`ArchivePatcher::patch`](crate::ArchivePatcher::patch), and it belongs in the name a caller reads at the call
/// site rather than in a flag they have to look up — the same reason `TranslationFormatter` splits `check_format` from
/// `format` instead of carrying an `is_check` field.
#[derive(Default)]
pub struct ArchivePatchOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
  /// Where the run says what it decided, one verbose line per classified entry and per volume.
  pub output: OutputOptions,
  /// Whether the run may publish over volumes of its set that the destination already holds.
  pub is_forced: bool,
  /// Turn what the format cannot express into a failure.
  ///
  /// A `.db` patch cannot delete: `CLocatorAPI::Register` overwrites a descriptor and never removes one. Entries the
  /// base holds and the target does not are always reported; this decides whether a release gate may pass with them
  /// outstanding.
  pub is_strict: bool,
  /// Confirm a pair the checksums agreed on by comparing its payloads.
  ///
  /// Equal size and equal CRC32 is what the engine itself trusts on every decompression, so this is off by default.
  /// It exists for a release someone wants certainty about, and for archives built by a tool whose recorded checksums
  /// are not known to be honest.
  pub is_verifying_payload: bool,
}

impl ArchivePatchOptions {
  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(self, job: JobHandle) -> Self {
    Self { job, ..self }
  }

  /// The same options, saying what the run decides through `output`.
  pub fn with_output(self, output: OutputOptions) -> Self {
    Self { output, ..self }
  }

  /// The same options, allowed to replace a set the destination already holds.
  pub fn with_force(self, is_forced: bool) -> Self {
    Self { is_forced, ..self }
  }

  /// The same options, failing a run whose base holds entries the target does not.
  pub fn with_strict(self, is_strict: bool) -> Self {
    Self { is_strict, ..self }
  }

  /// The same options, proving every checksum match by its payload.
  pub fn with_verified_payloads(self, is_verifying_payload: bool) -> Self {
    Self {
      is_verifying_payload,
      ..self
    }
  }
}

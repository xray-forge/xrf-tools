use xrf_animation_envelope::AnimationEnvelope;
use xrf_error::XrfResult;
use xrf_vfs::XrayAssetType;

use crate::GamedataVerificationRule;

/// A format of animation envelopes the animations check reads, writes back, evaluates and plays as the engine does, one
/// implementation per format.
pub(crate) trait EnvelopeAnimation: Sized {
  /// The kind of asset its files are, which is how they are enumerated.
  const ASSET_TYPE: XrayAssetType;

  /// What one file is called in a finding.
  const NAME: &'static str;

  /// What its files are called in a summary.
  const PLURAL_NAME: &'static str;

  /// The rule a file that does not read is reported under.
  const READ_RULE: GamedataVerificationRule;

  /// The rule a file that does not write back to the bytes it was read from is reported under.
  const ROUND_TRIP_RULE: GamedataVerificationRule;

  /// The rule a channel evaluating to something other than a finite number is reported under.
  const VALUE_RULE: GamedataVerificationRule;

  /// Reads a whole file.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a file of this format.
  fn read_file(bytes: Vec<u8>) -> XrfResult<Self>;

  /// The whole file it writes back as, or `None` for a version the writer does not lay out.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses what was read.
  fn write_file(&self) -> XrfResult<Option<Vec<u8>>>;

  /// Every envelope it carries under the name an author knows its channel by, in the order the file stores them.
  fn get_channels(&self) -> Vec<(String, &AnimationEnvelope)>;

  /// What the engine refuses while it plays the file, each under its own rule and message: faults a file can carry
  /// even when it reads, writes back and evaluates to finite numbers.
  fn find_playback_refusals(&self) -> Vec<(GamedataVerificationRule, String)>;
}

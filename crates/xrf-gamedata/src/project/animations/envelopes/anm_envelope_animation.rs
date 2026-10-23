use xrf_animation_envelope::AnimationEnvelope;
use xrf_anm::{AnmChannel, AnmFile};
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_vfs::XrayAssetType;

use crate::GamedataVerificationRule;
use crate::project::animations::envelopes::envelope_animation::EnvelopeAnimation;

impl EnvelopeAnimation for AnmFile {
  const ASSET_TYPE: XrayAssetType = XrayAssetType::Anm;

  const NAME: &'static str = "object motion";

  const PLURAL_NAME: &'static str = "object motions";

  const READ_RULE: GamedataVerificationRule = GamedataVerificationRule::AnimationsObjectMotionRead;

  const ROUND_TRIP_RULE: GamedataVerificationRule = GamedataVerificationRule::AnimationsObjectMotionRoundTrip;

  const VALUE_RULE: GamedataVerificationRule = GamedataVerificationRule::AnimationsObjectMotionValue;

  fn read_file(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_bytes::<XRayByteOrder>(bytes)
  }

  // The motion chunk alone: `COMotion::LoadMotion` finds that chunk and reads nothing else, so any other a file carries
  // is not written back.
  fn write_file(&self) -> XrfResult<Option<Vec<u8>>> {
    if !Self::WRITABLE_VERSIONS.contains(&self.version) {
      return Ok(None);
    }

    self.write_to_bytes::<XRayByteOrder>().map(Some)
  }

  fn get_channels(&self) -> Vec<(String, &AnimationEnvelope)> {
    AnmChannel::ALL
      .into_iter()
      .map(|channel| (channel.get_name().to_owned(), self.get_channel(channel)))
      .collect()
  }

  // `CObjectAnimator` plays whatever finite transform a motion evaluates to.
  fn find_playback_refusals(&self) -> Vec<(GamedataVerificationRule, String)> {
    Vec::new()
  }
}

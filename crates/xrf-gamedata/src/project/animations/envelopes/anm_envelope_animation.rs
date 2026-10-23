use xrf_animation_envelope::AnimationEnvelope;
use xrf_anm::{ANM_CHANNELS, AnmFile};
use xrf_chunk::{ChunkWriter, XRayByteOrder};
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

    let mut writer: ChunkWriter = ChunkWriter::new();

    self.write::<XRayByteOrder>(&mut writer)?;

    writer
      .flush_chunk_into_buffer::<XRayByteOrder>(Self::CHUNK_ID)
      .map(Some)
  }

  fn get_channels(&self) -> Vec<(String, &AnimationEnvelope)> {
    ANM_CHANNELS
      .iter()
      .zip(&self.channels)
      .map(|(channel, envelope)| ((*channel).to_owned(), envelope))
      .collect()
  }
}

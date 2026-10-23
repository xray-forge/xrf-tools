use xrf_animation_envelope::AnimationEnvelope;
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_ppe::PpeFile;
use xrf_vfs::XrayAssetType;

use crate::GamedataVerificationRule;
use crate::project::animations::envelopes::envelope_animation::EnvelopeAnimation;

impl EnvelopeAnimation for PpeFile {
  const ASSET_TYPE: XrayAssetType = XrayAssetType::Ppe;

  const NAME: &'static str = "post-process effect";

  const PLURAL_NAME: &'static str = "post-process effects";

  const READ_RULE: GamedataVerificationRule = GamedataVerificationRule::AnimationsPostProcessRead;

  const ROUND_TRIP_RULE: GamedataVerificationRule = GamedataVerificationRule::AnimationsPostProcessRoundTrip;

  const VALUE_RULE: GamedataVerificationRule = GamedataVerificationRule::AnimationsPostProcessValue;

  fn read_file(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_bytes::<XRayByteOrder>(bytes)
  }

  // Every version this reads is one it writes, and an effect is flat: the writer's buffer is the whole file.
  fn write_file(&self) -> XrfResult<Option<Vec<u8>>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    self.write::<XRayByteOrder>(&mut writer)?;

    Ok(Some(writer.buffer))
  }

  fn get_channels(&self) -> Vec<(String, &AnimationEnvelope)> {
    self.get_named_envelopes()
  }
}

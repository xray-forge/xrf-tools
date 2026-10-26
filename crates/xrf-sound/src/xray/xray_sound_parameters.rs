use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

use crate::vorbis::vorbis_packet_reader::VorbisPacketReader;
use crate::xray::xray_sound_comment_version::XRaySoundCommentVersion;
use crate::xray::xray_sound_game_type::XRaySoundGameType;

/// How the engine plays a sound as a source in the world, `SoundSourceInfo` (`xrSound/SoundRender_Source.h`), and
/// what the SDK's sound editor edits as `ESoundThumbnail`. By default the engine's own, which it plays a sound with when
/// the sound carries none.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XRaySoundParameters {
  /// Metres within which the sound plays at full volume.
  pub min_distance: f32,
  /// Metres past which the sound is not heard.
  pub max_distance: f32,
  /// The volume the sound plays at before distance, where 1 is as recorded.
  pub base_volume: f32,
  pub game_type: XRaySoundGameType,
  /// Metres past which the AI does not hear the sound.
  pub max_ai_distance: f32,
}

impl XRaySoundParameters {
  /// The shortest maximum distance, and maximum AI distance, `CSoundRender_Source::LoadWave` loads a sound with.
  pub const MINIMUM_MAX_DISTANCE: f32 = 0.1;

  /// The minimum distances the SDK's sound editor accepts, `ESoundThumbnail::FillProp`.
  pub const MIN_DISTANCE_RANGE: RangeInclusive<f32> = 0.01..=1000.0;

  /// The maximum distances the SDK's sound editor accepts; the maximum AI distance also stays within the maximum one.
  pub const MAX_DISTANCE_RANGE: RangeInclusive<f32> = Self::MINIMUM_MAX_DISTANCE..=1000.0;

  /// The base volumes the SDK's sound editor accepts.
  pub const BASE_VOLUME_RANGE: RangeInclusive<f32> = 0.0..=2.0;

  /// Reads the parameter block of the first comment as `CSoundRender_Source::LoadWave` does: a `u32` version, then that
  /// version's fields. Version 1 stores no base volume, which reads as 1, and versions 1 and 2 no maximum AI distance,
  /// which reads as the maximum distance. None for a version the engine does not read.
  pub(crate) fn read(comment: &[u8]) -> Result<Option<(XRaySoundCommentVersion, Self)>, String> {
    let mut reader: VorbisPacketReader = VorbisPacketReader::new(comment);
    let Some(version) = XRaySoundCommentVersion::from_code(reader.read_u32("X-Ray comment version")?) else {
      return Ok(None);
    };

    let min_distance: f32 = reader.read_f32("minimum distance")?;
    let max_distance: f32 = reader.read_f32("maximum distance")?;
    let base_volume: f32 = match version {
      XRaySoundCommentVersion::V1 => 1.0,
      XRaySoundCommentVersion::V2 | XRaySoundCommentVersion::V3 => reader.read_f32("base volume")?,
    };
    let game_type: XRaySoundGameType = XRaySoundGameType::from(reader.read_u32("game type")?);
    let max_ai_distance: f32 = match version {
      XRaySoundCommentVersion::V1 | XRaySoundCommentVersion::V2 => max_distance,
      XRaySoundCommentVersion::V3 => reader.read_f32("maximum AI distance")?,
    };
    let parameters: Self = Self {
      min_distance,
      max_distance,
      base_volume,
      game_type,
      max_ai_distance,
    };

    match parameters.get_defect() {
      Some(defect) => Err(String::from(defect)),
      None => Ok(Some((version, parameters))),
    }
  }

  /// The comment `CSoundManager::MakeGameSound` writes: a version 3 parameter block.
  ///
  /// # Errors
  ///
  /// Returns an error for parameters the engine refuses to load a sound with, see [`Self::validate`].
  pub fn write(&self) -> XrfResult<Vec<u8>> {
    self.validate()?;

    let mut comment: Vec<u8> = Vec::with_capacity(24);

    comment.extend_from_slice(&XRaySoundCommentVersion::CURRENT.get_code().to_le_bytes());
    comment.extend_from_slice(&self.min_distance.to_le_bytes());
    comment.extend_from_slice(&self.max_distance.to_le_bytes());
    comment.extend_from_slice(&self.base_volume.to_le_bytes());
    comment.extend_from_slice(&u32::from(self.game_type).to_le_bytes());
    comment.extend_from_slice(&self.max_ai_distance.to_le_bytes());

    Ok(comment)
  }

  /// Checks the parameters are ones the engine loads a sound with.
  ///
  /// # Errors
  ///
  /// Returns an error for a parameter that is not a finite number, or a maximum or maximum AI distance below
  /// [`Self::MINIMUM_MAX_DISTANCE`], where `LoadWave` refuses the sound.
  pub fn validate(&self) -> XrfResult {
    match self.get_defect() {
      Some(defect) => Err(XrfError::new_invalid_error(defect)),
      None => Ok(()),
    }
  }

  /// Brings every parameter within the range the SDK's sound editor holds it to, the maximum AI distance within the
  /// maximum distance as `ESoundThumbnail::OnMaxDistChange` clamps it.
  pub fn clamp(&mut self) {
    self.min_distance = Self::clamp_value(self.min_distance, &Self::MIN_DISTANCE_RANGE);
    self.max_distance = Self::clamp_value(self.max_distance, &Self::MAX_DISTANCE_RANGE);
    self.base_volume = Self::clamp_value(self.base_volume, &Self::BASE_VOLUME_RANGE);
    self.max_ai_distance = Self::clamp_value(
      self.max_ai_distance,
      &(*Self::MAX_DISTANCE_RANGE.start()..=self.max_distance),
    );
  }

  /// Why the engine refuses the parameters, read and written alike.
  fn get_defect(&self) -> Option<&'static str> {
    if ![
      self.min_distance,
      self.max_distance,
      self.base_volume,
      self.max_ai_distance,
    ]
    .iter()
    .all(|value| value.is_finite())
    {
      return Some("X-Ray sound parameters must be finite numbers");
    }

    if self.max_distance < Self::MINIMUM_MAX_DISTANCE || self.max_ai_distance < Self::MINIMUM_MAX_DISTANCE {
      return Some("X-Ray sound maximum and maximum AI distances must be at least 0.1");
    }

    None
  }

  fn clamp_value(value: f32, range: &RangeInclusive<f32>) -> f32 {
    value.clamp(*range.start(), *range.end())
  }
}

impl Default for XRaySoundParameters {
  /// `SoundSourceInfo`'s defaults, which `ESoundThumbnail` starts a sound with as well.
  fn default() -> Self {
    Self {
      min_distance: 1.0,
      max_distance: 300.0,
      base_volume: 1.0,
      game_type: XRaySoundGameType::default(),
      max_ai_distance: 300.0,
    }
  }
}

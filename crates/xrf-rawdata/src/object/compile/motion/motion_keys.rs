use byteorder::{ByteOrder, WriteBytesExt};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Quaternion, Vector3d};

/// One bone's run of keys in an OGF motion, as `CExportSkeleton::ExportMotionKeys` writes it: the rotation of every
/// frame quantised to 16 bits, or one key where it never changes, then the translation quantised within its range to 8
/// or 16 bits, or the mean where it never leaves a millimetre of it.
pub(crate) struct MotionKeys;

impl MotionKeys {
  /// `KEY_Quant`: a rotation component's step.
  const ROTATION_QUANT: f32 = 32767.0;

  /// A translation whose box's half extent is longer than this takes 16 bit keys.
  const WIDE_TRANSLATION: f32 = 1.5;

  /// `EPS_L`: a translation this close to the mean on every axis does not move.
  const STILL_TRANSLATION: f32 = 0.001;

  /// `flTKeyPresent`.
  const TRANSLATION_KEYS_PRESENT: u8 = 1 << 0;
  /// `flRKeyAbsent`.
  const ROTATION_KEYS_ABSENT: u8 = 1 << 1;
  /// `flTKey16IsBit`.
  const TRANSLATION_KEYS_16_BIT: u8 = 1 << 2;

  /// A rotation as its key stores it, each component truncated towards zero.
  pub(crate) fn quantize_rotation(rotation: &Quaternion) -> [i16; 4] {
    [rotation.x, rotation.y, rotation.z, rotation.w]
      .map(|component| Self::truncate(component * Self::ROTATION_QUANT).clamp(-32767, 32767) as i16)
  }

  /// Appends a bone's run over its frames, one rotation and translation each.
  ///
  /// # Errors
  ///
  /// Returns an error for a run of no frames, or of more rotations than translations or fewer, and when the bytes do
  /// not write.
  pub(crate) fn write<T: ByteOrder>(
    output: &mut Vec<u8>,
    rotations: &[[i16; 4]],
    translations: &[Vector3d],
  ) -> XrfResult {
    let frames: usize = translations.len();

    if frames == 0 || rotations.len() != frames {
      return Err(XrfError::new_invalid_error(format!(
        "A bone's motion keys take a rotation and a translation each frame, not {} and {frames}",
        rotations.len()
      )));
    }

    let mut mean: Vector3d = Vector3d::ZERO;
    let (mut low, mut high): (Vector3d, Vector3d) = (translations[0], translations[0]);

    for translation in translations {
      mean = &mean + translation;
      low = low.min(translation);
      high = high.max(translation);
    }

    let mean: Vector3d = Vector3d::new(mean.x / frames as f32, mean.y / frames as f32, mean.z / frames as f32);
    let center: Vector3d = &(&high + &low) * 0.5;
    let reach: Vector3d = &(&high - &low) * 0.5;
    let is_wide: bool = reach.length() > Self::WIDE_TRANSLATION;
    let is_moving: bool = translations.iter().any(|translation| {
      (mean.x - translation.x).abs() >= Self::STILL_TRANSLATION
        || (mean.y - translation.y).abs() >= Self::STILL_TRANSLATION
        || (mean.z - translation.z).abs() >= Self::STILL_TRANSLATION
    });
    let is_turning: bool = rotations.iter().any(|rotation| *rotation != rotations[0]);

    output.write_u8(
      if is_moving { Self::TRANSLATION_KEYS_PRESENT } else { 0 }
        | if is_turning { 0 } else { Self::ROTATION_KEYS_ABSENT }
        | if is_wide { Self::TRANSLATION_KEYS_16_BIT } else { 0 },
    )?;

    if is_turning {
      let mut keys: Vec<u8> = Vec::with_capacity(rotations.len() * 8);

      for component in rotations.iter().flatten() {
        keys.write_i16::<T>(*component)?;
      }

      Self::write_checked::<T>(output, &keys)?;
    } else {
      for component in rotations[0] {
        output.write_i16::<T>(component)?;
      }
    }

    if !is_moving {
      return Self::write_vector::<T>(output, &mean);
    }

    let (steps, lowest, highest): (f32, i32, i32) = if is_wide {
      (32767.0, -32767, 32767)
    } else {
      (127.0, -128, 127)
    };
    let mut keys: Vec<u8> = Vec::with_capacity(frames * if is_wide { 6 } else { 3 });

    for translation in translations {
      for (value, (center, reach)) in translation
        .to_array()
        .into_iter()
        .zip(center.to_array().into_iter().zip(reach.to_array()))
      {
        // An axis that never moves divides zero by zero, which the export stored as key 0.
        let key: i32 = Self::truncate(steps * (value - center) / reach).clamp(lowest, highest);

        if is_wide {
          keys.write_i16::<T>(key as i16)?;
        } else {
          keys.write_i8(key as i8)?;
        }
      }
    }

    Self::write_checked::<T>(output, &keys)?;
    Self::write_vector::<T>(
      output,
      &Vector3d::new(reach.x / steps, reach.y / steps, reach.z / steps),
    )?;
    Self::write_vector::<T>(output, &center)
  }

  /// A stream led by its checksum, which the engine shares identical streams by.
  fn write_checked<T: ByteOrder>(output: &mut Vec<u8>, keys: &[u8]) -> XrfResult {
    output.write_u32::<T>(crc32fast::hash(keys))?;
    output.extend_from_slice(keys);

    Ok(())
  }

  fn write_vector<T: ByteOrder>(output: &mut Vec<u8>, vector: &Vector3d) -> XrfResult {
    for component in vector.to_array() {
      output.write_f32::<T>(component)?;
    }

    Ok(())
  }

  /// `int(value)`: towards zero, saturating, and `NaN` as 0, which is what the vanilla exports store for an axis
  /// that never moves.
  fn truncate(value: f32) -> i32 {
    value as i32
  }
}

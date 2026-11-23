use xrf_error::{XrfError, XrfResult};

use crate::scene::texture::sequence::texture_sequence_timing::TextureSequenceTiming;

/// A texture's `.seq` frame list as `CTexture::Load` reads it: an optional `cycled` line, the frame rate, then one frame
/// texture a line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TextureSequence {
  pub timing: TextureSequenceTiming,
  /// Each frame's texture reference, in the order shown, as written: a frame may repeat, or carry an image extension.
  pub frames: Vec<String>,
}

impl TextureSequence {
  /// The first line that makes a sequence play back and forth, read without regard to case.
  const CYCLED: &'static str = "cycled";

  /// Reads a frame list: lines trimmed and the blank ones skipped as the engine skips them.
  ///
  /// # Errors
  ///
  /// Returns an error for a frame rate the engine would divide by zero with: none, nought, negative or past a thousand.
  pub(crate) fn parse(text: &str) -> XrfResult<Self> {
    let mut lines = text.lines();
    let mut rate: &str = lines.next().unwrap_or_default();
    let is_cycled: bool = rate.eq_ignore_ascii_case(Self::CYCLED);

    if is_cycled {
      rate = lines.next().unwrap_or_default();
    }

    let fps: u32 = Self::parse_rate(rate)
      .filter(|fps| (1..=1000).contains(fps))
      .ok_or_else(|| {
        XrfError::new_texture_processing_error(format!(
          "A texture sequence's frame rate '{}' is not one from 1 to 1000",
          rate.trim()
        ))
      })?;

    Ok(Self {
      timing: TextureSequenceTiming {
        frame_ms: 1000 / fps,
        is_cycled,
      },
      frames: lines
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect(),
    })
  }

  /// The number a line starts with, as `atoi` reads it: leading blanks skipped, the digits up to the first other
  /// character; none for a line starting with none, or a negative one.
  fn parse_rate(line: &str) -> Option<u32> {
    let digits: &str = line.trim_start().strip_prefix('+').unwrap_or(line.trim_start());
    let end: usize = digits.find(|it: char| !it.is_ascii_digit()).unwrap_or(digits.len());

    digits[..end].parse().ok()
  }
}

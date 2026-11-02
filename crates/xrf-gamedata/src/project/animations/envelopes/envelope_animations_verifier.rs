use std::marker::PhantomData;

use xrf_animation_envelope::AnimationEnvelope;
use xrf_error::XrfResult;
use xrf_output::OutputOptions;

use crate::project::animations::envelopes::envelope_animation::EnvelopeAnimation;
use crate::project::animations::envelopes::envelope_animations_verification_result::GamedataEnvelopeAnimationsVerificationResult;
use crate::project::gamedata_asset_sweep::GamedataAssetSweep;
use crate::project::gamedata_asset_sweep_outcome::GamedataAssetSweepOutcome;
use crate::{Finding, GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions};

/// Reads every file of one envelope animation format the game holds, writes each back, and evaluates each channel.
pub(crate) struct EnvelopeAnimationsVerifier<'a, F> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
  // Names the format without holding one, so sharing the verifier across the workers asks nothing of the format.
  format: PhantomData<fn() -> F>,
}

impl<'a, F: EnvelopeAnimation> EnvelopeAnimationsVerifier<'a, F> {
  pub(crate) fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self {
      options,
      project,
      format: PhantomData,
    }
  }

  pub(crate) fn verify(&self) -> XrfResult<GamedataEnvelopeAnimationsVerificationResult<F>> {
    self.options.job.check_cancelled()?;

    // Enumerated through the VFS, so an installation's archived files are verified too.
    let paths: Vec<String> = self
      .project
      .entries_of_type(F::ASSET_TYPE)
      .into_iter()
      .map(|location| location.get_logical_path().to_string())
      .collect();

    let outcome: GamedataAssetSweepOutcome =
      GamedataAssetSweep::new(self.options).run(&paths, |options, path| self.verify_file(&options.output, path))?;

    self.options.job.check_cancelled()?;

    xrf_output::info!(
      self.options.output,
      "Verified gamedata {}, {}/{} valid",
      F::PLURAL_NAME,
      outcome.checked_count - outcome.invalid_count,
      outcome.checked_count,
    );

    Ok(GamedataEnvelopeAnimationsVerificationResult {
      checked_files_count: outcome.checked_count,
      findings: outcome.findings,
      invalid_files_count: outcome.invalid_count,
      format: PhantomData,
    })
  }

  /// What is wrong with one file: that it does not read, or else that it does not write back to the bytes it was read
  /// from, which of its channels evaluate to something other than a finite number, and what the engine refuses while
  /// it plays the file.
  fn verify_file(&self, output: &OutputOptions, path: &str) -> Vec<Finding> {
    xrf_output::verbose!(output, "Verify {}: {}", F::NAME, path);

    let read: XrfResult<(Vec<u8>, F)> = self
      .project
      .read_bytes(path)
      .and_then(|bytes| F::read_file(bytes.clone()).map(|file| (bytes, file)));

    let (bytes, file) = match read {
      Ok(read) => read,
      Err(error) => {
        xrf_output::info!(output, "Failed to read {} '{}': {}", F::NAME, path, error);

        return vec![GamedataFindingFactory::for_asset(
          F::READ_RULE,
          path,
          format!("Failed to read {}: {error}", F::NAME),
        )];
      }
    };

    let mut findings: Vec<Finding> = Vec::new();

    match file.write_file() {
      Ok(Some(written)) if written != bytes => findings.push(GamedataFindingFactory::for_asset(
        F::ROUND_TRIP_RULE,
        path,
        Self::describe_round_trip(&bytes, &written),
      )),
      Ok(_) => {}
      Err(error) => findings.push(GamedataFindingFactory::for_asset(
        F::ROUND_TRIP_RULE,
        path,
        format!("Failed to write the {} back: {error}", F::NAME),
      )),
    }

    for (channel, envelope) in file.get_channels() {
      if let Some((time, value)) = Self::find_non_finite_value(envelope) {
        findings.push(GamedataFindingFactory::for_asset(
          F::VALUE_RULE,
          path,
          format!("Channel '{channel}' evaluates to {value} at time {time:.3}"),
        ));
      }
    }

    for (rule, message) in file.find_playback_refusals() {
      findings.push(GamedataFindingFactory::for_asset(rule, path, message));
    }

    if !findings.is_empty() {
      xrf_output::info!(output, "Invalid {}: {}", F::NAME, path);
    }

    findings
  }

  /// How the bytes a file writes back as differ from the bytes it was read from.
  fn describe_round_trip(read: &[u8], written: &[u8]) -> String {
    let first_difference: usize = read
      .iter()
      .zip(written)
      .position(|(read, written)| read != written)
      .unwrap_or(read.len().min(written.len()));

    format!(
      "Does not write back to the bytes it was read from: {} bytes read, {} written, first difference at byte {}",
      read.len(),
      written.len(),
      first_difference
    )
  }

  /// The first time, and the value there, at which a channel evaluates to something other than a finite number. It is
  /// sampled at each key, the middle of each span, and one span's reach either side of the keys, where the envelope's
  /// behaviours take over; a span shorter than a second reaches a second.
  fn find_non_finite_value(envelope: &AnimationEnvelope) -> Option<(f32, f32)> {
    let (first, last) = (envelope.keys.first()?, envelope.keys.last()?);
    let reach: f32 = (last.time - first.time).max(1.0);
    let mut times: Vec<f32> = envelope
      .keys
      .windows(2)
      .map(|span| (span[0].time + span[1].time) * 0.5)
      .chain(envelope.keys.iter().map(|key| key.time))
      .chain([first.time - reach, last.time + reach])
      .collect();

    times.sort_by(f32::total_cmp);

    times
      .into_iter()
      .map(|time| (time, envelope.evaluate(time)))
      .find(|(_, value)| !value.is_finite())
  }
}

#[cfg(test)]
mod tests {
  use xrf_animation_envelope::{AnimationBehavior, AnimationEnvelope, AnimationKey, AnimationShape};
  use xrf_anm::AnmFile;

  use super::EnvelopeAnimationsVerifier;

  fn key(time: f32, value: f32) -> AnimationKey {
    AnimationKey {
      value,
      time,
      shape: AnimationShape::Linear,
      ..AnimationKey::default()
    }
  }

  fn envelope(keys: Vec<AnimationKey>) -> AnimationEnvelope {
    AnimationEnvelope {
      keys,
      ..AnimationEnvelope::default()
    }
  }

  #[test]
  fn a_finite_channel_has_no_value_to_report() {
    assert_eq!(
      EnvelopeAnimationsVerifier::<AnmFile>::find_non_finite_value(&envelope(vec![key(0.0, 1.0), key(2.0, 3.0)])),
      None
    );
    assert_eq!(
      EnvelopeAnimationsVerifier::<AnmFile>::find_non_finite_value(&AnimationEnvelope::default()),
      None
    );
  }

  #[test]
  fn a_non_finite_key_is_found_where_it_sits() {
    let found: Option<(f32, f32)> =
      EnvelopeAnimationsVerifier::<AnmFile>::find_non_finite_value(&envelope(vec![key(0.0, 1.0), key(2.0, f32::NAN)]));

    assert!(matches!(found, Some((1.0, value)) if value.is_nan()), "{found:?}");
  }

  // Two keys at one time leave the linear behaviour a slope over no time, which only the time past the keys reaches.
  #[test]
  fn a_behaviour_dividing_by_a_span_of_no_time_is_found_past_the_keys() {
    let stacked: AnimationEnvelope = AnimationEnvelope {
      behavior_after: AnimationBehavior::Linear,
      ..envelope(vec![key(1.0, 0.0), key(1.0, 2.0)])
    };

    assert_eq!(
      EnvelopeAnimationsVerifier::<AnmFile>::find_non_finite_value(&stacked),
      Some((2.0, f32::INFINITY))
    );
  }

  #[test]
  fn a_round_trip_names_the_sizes_and_the_first_difference() {
    assert_eq!(
      EnvelopeAnimationsVerifier::<AnmFile>::describe_round_trip(&[1, 2, 3, 4], &[1, 2, 3]),
      "Does not write back to the bytes it was read from: 4 bytes read, 3 written, first difference at byte 3"
    );
    assert_eq!(
      EnvelopeAnimationsVerifier::<AnmFile>::describe_round_trip(&[1, 2, 3], &[1, 9, 3]),
      "Does not write back to the bytes it was read from: 3 bytes read, 3 written, first difference at byte 1"
    );
  }
}

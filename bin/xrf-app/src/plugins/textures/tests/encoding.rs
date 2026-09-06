//! Pins what the held comparison answers for: the candidates one run really encoded, and nothing else.

use xrf_dds::{DdsEncodeAttempt, DdsEncodeCandidate, DdsMipChain, DdsMipmaps, Quality};
use xrf_job::JobOutcome;

use crate::plugins::textures::encoding::{
  TextureEncodingComparison, TextureEncodingCurrent, TextureEncodingFormat, TextureEncodingSession,
};
use crate::plugins::textures::source::TextureSource;
use crate::plugins::textures::state::TextureState;
use crate::plugins::textures::tests::fixtures::{BASE, source_image};

#[test]
fn a_held_session_answers_for_the_candidates_it_weighed_and_no_others() {
  let chain: DdsMipChain = DdsMipChain::build(&source_image(16), DdsMipmaps::Disabled).expect("chain");
  let session: TextureEncodingSession = TextureEncodingSession {
    source: TextureSource::Asset {
      reference: String::from(BASE),
    },
    label: String::from(BASE),
    attempts: vec![
      DdsEncodeAttempt::measure(&chain, DdsEncodeCandidate::Bc3, Quality::Fast).expect("bc3"),
      DdsEncodeAttempt::measure(&chain, DdsEncodeCandidate::Rgba8, Quality::Fast).expect("rgba8"),
    ],
  };

  assert!(session.get(TextureEncodingFormat::Bc3).is_some());
  assert!(
    session.get(TextureEncodingFormat::Bc7).is_none(),
    "expect a save to be unable to reach a candidate this comparison never encoded"
  );

  let comparison: TextureEncodingComparison = session.to_comparison(
    TextureEncodingCurrent {
      label: String::from("DXT5"),
      file_bytes: 0,
      gpu_bytes: 0,
      width: 16,
      height: 16,
      mipmap_levels: 1,
    },
    JobOutcome::Cancelled,
  );

  assert_eq!(comparison.reference, BASE);
  assert_eq!(
    comparison.outcome,
    JobOutcome::Cancelled,
    "expect a stopped comparison to still report the candidates it reached"
  );
  assert_eq!(comparison.candidates.len(), 2);

  // Uncompressed is the one candidate that loses nothing, and it is the largest.
  let uncompressed = comparison
    .candidates
    .iter()
    .find(|candidate| candidate.format == TextureEncodingFormat::Rgba8)
    .expect("rgba8 is reported");
  let compressed = comparison
    .candidates
    .iter()
    .find(|candidate| candidate.format == TextureEncodingFormat::Bc3)
    .expect("bc3 is reported");

  assert_eq!(uncompressed.psnr, None);
  assert!(compressed.psnr.is_some_and(|psnr| psnr > 0.0));
  assert!(uncompressed.gpu_bytes > compressed.gpu_bytes);
}

#[test]
fn a_candidate_can_be_looked_at_only_while_its_own_comparison_is_the_held_one() {
  // What the A/B preview reads. The bytes it draws are the encode the comparison measured, so a picture of a format
  // can only exist while that measurement does - and the failure, when the session has moved on, has to say so rather
  // than serve a picture of something else.
  let chain: DdsMipChain = DdsMipChain::build(&source_image(16), DdsMipmaps::Disabled).expect("chain");
  let state: TextureState = TextureState::new();

  let missing = state.with_held_encoding(TextureEncodingFormat::Bc3, |_| Ok(()));

  assert!(
    missing.is_err_and(|error| error.contains("No encoded texture is held")),
    "expect nothing to read before anything has been weighed"
  );

  *state.encodings.lock().expect("encodings") = Some(TextureEncodingSession {
    source: TextureSource::Asset {
      reference: String::from(BASE),
    },
    label: String::from(BASE),
    attempts: vec![DdsEncodeAttempt::measure(&chain, DdsEncodeCandidate::Bc3, Quality::Fast).expect("bc3")],
  });

  let png: Vec<u8> = state
    .with_held_encoding(TextureEncodingFormat::Bc3, |file| {
      Ok(file.to_png().map_err(|error| error.to_string())?.bytes)
    })
    .expect("the weighed candidate decodes");

  assert!(!png.is_empty(), "expect the held encode to decode to a picture");

  let unweighed = state.with_held_encoding(TextureEncodingFormat::Bc7, |_| Ok(()));

  assert!(
    unweighed.is_err_and(|error| error.contains("does not carry that format")),
    "expect a format this comparison never encoded to be named as absent rather than answered with another"
  );
}

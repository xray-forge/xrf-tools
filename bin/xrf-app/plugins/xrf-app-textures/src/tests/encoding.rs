//! Pins what the held comparison answers for: the candidates one run really encoded, the texture they were made of,
//! and nothing else.

use std::path::{Path, PathBuf};

use xrf_app_core::session::{SessionId, SessionScopes};
use xrf_dds::{DdsEncodeAttempt, DdsEncodeCandidate, DdsMipChain, DdsMipmaps, Quality};
use xrf_job::JobOutcome;
use xrf_texture::TextureEncodingFormat;
use xrf_vfs::XrayRoots;

use crate::describe::{TextureFiles, TextureSource};
use crate::encoding::{TextureComparison, TextureComparisons, TextureEncodingComparison, TextureEncodingCurrent};
use crate::tests::fixtures::{BASE, source_image};

fn comparison(files: Option<TextureFiles>, candidates: &[DdsEncodeCandidate]) -> TextureComparison {
  let chain: DdsMipChain = DdsMipChain::build(&source_image(16), DdsMipmaps::Disabled).expect("chain");

  TextureComparison {
    roots: XrayRoots::default(),
    source: TextureSource::Asset {
      reference: String::from(BASE),
    },
    label: String::from(BASE),
    files,
    attempts: candidates
      .iter()
      .map(|candidate| DdsEncodeAttempt::measure(&chain, *candidate, Quality::Fast).expect("encode"))
      .collect(),
  }
}

#[test]
fn a_held_comparison_answers_for_the_candidates_it_weighed_and_no_others() {
  let held: TextureComparison = comparison(None, &[DdsEncodeCandidate::Bc3, DdsEncodeCandidate::Rgba8]);
  let session_id: SessionId = SessionId::generate();

  assert!(held.get(TextureEncodingFormat::Bc3).is_some());
  assert!(
    held.get(TextureEncodingFormat::Bc7).is_none(),
    "expect a save to be unable to reach a candidate this comparison never encoded"
  );

  let answer: TextureEncodingComparison = held.to_comparison(
    session_id,
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

  assert_eq!(answer.session_id, session_id);
  assert_eq!(answer.reference, BASE);
  assert_eq!(
    answer.outcome,
    JobOutcome::Cancelled,
    "expect a stopped comparison to still report the candidates it reached"
  );
  assert_eq!(answer.candidates.len(), 2);

  // Uncompressed is the one candidate that loses nothing, and it is the largest.
  let uncompressed = answer
    .candidates
    .iter()
    .find(|candidate| candidate.format == TextureEncodingFormat::Rgba8)
    .expect("rgba8 is reported");
  let compressed = answer
    .candidates
    .iter()
    .find(|candidate| candidate.format == TextureEncodingFormat::Bc3)
    .expect("bc3 is reported");

  assert_eq!(uncompressed.psnr, None);
  assert!(compressed.psnr.is_some_and(|psnr| psnr > 0.0));
  assert!(uncompressed.gpu_bytes > compressed.gpu_bytes);
}

#[test]
fn a_candidate_can_be_looked_at_only_while_its_own_comparison_is_held() {
  // What the A/B preview reads. The bytes it draws are the encode the comparison measured, so a picture of a format
  // can only exist while that measurement does - and the failure, when the comparison has moved on, has to say so
  // rather than serve a picture of something else.
  let comparisons: TextureComparisons = TextureComparisons::new("texture comparison", &SessionScopes::default());
  let session_id: SessionId = SessionId::generate();
  let opening = comparisons.begin_fixture_open(session_id).expect("comparison");

  assert!(
    comparisons.require(session_id).is_err(),
    "expect nothing to read before anything has been weighed"
  );

  opening
    .commit(comparison(None, &[DdsEncodeCandidate::Bc3]))
    .expect("hold comparison");

  let held = comparisons.require(session_id).expect("comparison");
  let png: Vec<u8> = held
    .require(TextureEncodingFormat::Bc3)
    .expect("candidate")
    .to_png()
    .expect("the weighed candidate decodes")
    .bytes;

  assert!(!png.is_empty(), "expect the held encode to decode to a picture");
  assert!(
    held
      .require(TextureEncodingFormat::Bc7)
      .is_err_and(|error| error.contains("does not carry that format")),
    "expect a format this comparison never encoded to be named as absent rather than answered with another"
  );

  comparisons.close(&[session_id]);

  assert!(
    comparisons.require(session_id).is_err(),
    "a closed comparison is held no more"
  );
}

#[test]
fn a_comparison_is_saved_only_beside_the_descriptor_of_the_texture_it_was_made_of() {
  // The encodes are of one texture's pixels; written beside another's descriptor they would replace that texture.
  let files: TextureFiles = TextureFiles::of(&PathBuf::from("D:/gamedata/textures/ston/ston_beton05.dds"));
  let held: TextureComparison = comparison(Some(files.clone()), &[DdsEncodeCandidate::Bc3]);

  assert_eq!(held.require_paired(&files.descriptor).expect("paired"), &files);
  assert!(
    held
      .require_paired(Path::new("D:/gamedata/textures/ston/ston_beton06.thm"))
      .is_err_and(|error| error.contains("whose descriptor is")),
    "expect another texture's descriptor to be refused rather than written beside these encodes"
  );
}

#[test]
fn a_comparison_of_an_archived_texture_has_nowhere_to_be_saved() {
  let held: TextureComparison = comparison(None, &[DdsEncodeCandidate::Bc3]);

  assert!(
    held
      .require_paired(Path::new("D:/gamedata/textures/ston/ston_beton05.thm"))
      .is_err_and(|error| error.contains("served out of an archive"))
  );
}

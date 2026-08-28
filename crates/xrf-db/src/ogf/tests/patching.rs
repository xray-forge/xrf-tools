//! Patching a visual the engine loads but the format does not describe.
//!
//! The property under test is the one the losslessness guard rests on: a rewrite that changes nothing must land on
//! `normalize(source)`, which is the source itself whenever the source is well formed.

use std::fs;
use std::fs::File;
use std::path::PathBuf;

use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_test_utils::utils::{build_relative_test_sample_file_path, write_generated_test_resource};

use crate::ogf::ogf_file::OgfFile;
use crate::ogf::ogf_motion_refs_processor::OgfMotionRefsProcessor;
use crate::ogf::ogf_refs_patch_report::OgfRefsPatchReport;
use crate::ogf::ogf_residue::normalize_ogf_bytes;
use crate::ogf::ogf_texture_refs_processor::OgfTextureRefsProcessor;
use crate::ogf::tests::fixtures;

fn write_fixture(name: &str, bytes: Vec<u8>) -> XrfResult<PathBuf> {
  Ok(write_generated_test_resource(
    &build_relative_test_sample_file_path(file!(), name),
    bytes,
  )?)
}

#[test]
fn rewriting_existing_refs_lands_on_the_normalized_source() -> XrfResult {
  let source: Vec<u8> = fixtures::split_motion_ref()?;
  let path: PathBuf = write_fixture("unchanged_rewrite.ogf", source.clone())?;
  let existing: Vec<String> = fixtures::DECLARED_REFS.map(String::from).to_vec();

  let rewritten: Vec<u8> =
    OgfMotionRefsProcessor::write_motion_refs_to_buffer::<XRayByteOrder>(File::open(&path)?, &existing)?;

  assert_ne!(
    rewritten, source,
    "Expect a rewrite to drop what the engine ignores rather than carry it"
  );
  assert_eq!(
    rewritten,
    normalize_ogf_bytes::<XRayByteOrder>(&source)?,
    "Expect the rewrite to agree with the guard's target byte for byte"
  );

  Ok(())
}

#[test]
fn patching_a_split_reference_visual_is_allowed_and_reports_what_it_dropped() -> XrfResult {
  let source: Vec<u8> = fixtures::split_motion_ref()?;
  let path: PathBuf = write_fixture("patch_split_reference.ogf", source.clone())?;
  let replacement: Vec<String> = vec![String::from("actors\\replacement_animation")];

  // The whole point of the issue: this call used to fail before it could reach the chunk it edits.
  let report: OgfRefsPatchReport =
    OgfMotionRefsProcessor::patch_motion_refs_to_path::<XRayByteOrder>(&path, &path, &replacement, false)?;

  assert_eq!(
    report.discarded_size, 34,
    "Expect both fragments of the uncounted fifth path to be counted as discarded"
  );
  assert_eq!(report.patched_count, 1);

  let patched: OgfFile = OgfFile::read_from_path::<XRayByteOrder, _>(&path)?;

  assert_eq!(patched.kinematics.map(|it| it.motion_refs), Some(replacement));
  assert!(
    patched.residue.is_none(),
    "Expect the patched file to be well formed, with nothing left for the engine to ignore"
  );

  Ok(())
}

#[test]
fn patching_a_well_formed_visual_still_reproduces_it_exactly() -> XrfResult {
  // The regression that matters most: normalization must be invisible to the 99% of visuals that need none.
  let source: Vec<u8> = fixtures::well_formed()?;
  let path: PathBuf = write_fixture("patch_well_formed.ogf", source.clone())?;
  let existing: Vec<String> = fixtures::DECLARED_REFS.map(String::from).to_vec();

  let rewritten: Vec<u8> =
    OgfMotionRefsProcessor::write_motion_refs_to_buffer::<XRayByteOrder>(File::open(&path)?, &existing)?;

  assert_eq!(rewritten, source, "Expect an unchanged rewrite to reproduce the source");

  let report: OgfRefsPatchReport =
    OgfMotionRefsProcessor::patch_motion_refs_to_path::<XRayByteOrder>(&path, &path, &existing, true)?;

  assert_eq!(report.discarded_size, 0);

  Ok(())
}

#[test]
fn refuses_to_patch_a_visual_it_cannot_account_for() -> XrfResult {
  let path: PathBuf = write_fixture("patch_unexplained.ogf", fixtures::unexplained_residue()?)?;
  let original: Vec<u8> = fs::read(&path)?;

  assert!(
    OgfMotionRefsProcessor::patch_motion_refs_to_path::<XRayByteOrder>(
      &path,
      &path,
      &[String::from("actors\\anything")],
      false
    )
    .is_err(),
    "Expect a file the reader refuses to stay unpatched rather than be rewritten to its last good chunk"
  );
  assert_eq!(fs::read(&path)?, original, "Expect the refused source to be untouched");

  Ok(())
}

#[test]
fn renaming_a_texture_also_normalizes_the_motion_refs_chunk() -> XrfResult {
  // The texture patcher never touches chunk 24, but it rebuilds the file, so it has to agree with the same target.
  let source: Vec<u8> = fixtures::split_motion_ref()?;
  let path: PathBuf = write_fixture("texture_rename.ogf", source.clone())?;

  let (rewritten, patched_count) =
    OgfTextureRefsProcessor::write_texture_refs_to_buffer::<XRayByteOrder>(File::open(&path)?, "any", "any")?;

  assert_eq!(patched_count, 0, "Expect no texture reference in this fixture");
  assert_eq!(
    rewritten,
    normalize_ogf_bytes::<XRayByteOrder>(&source)?,
    "Expect both raw patchers to produce the same normalized bytes for the same source"
  );

  Ok(())
}

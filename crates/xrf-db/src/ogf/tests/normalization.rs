use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;

use crate::ogf::residue::{OgfNormalization, OgfResidueCause};
use crate::ogf::tests::fixtures;

/// The residue rides along with the bytes: after a write it is the only record of the path that went.
#[test]
fn normalizing_names_what_it_discards() -> XrfResult {
  let split: OgfNormalization = OgfNormalization::normalize::<XRayByteOrder>(&fixtures::split_motion_ref()?)?;
  let cause: OgfResidueCause = split
    .residue
    .expect("Expect the discarded path to travel with the bytes")
    .cause;

  assert_eq!(split.bytes, fixtures::well_formed()?);
  assert_eq!(
    cause,
    OgfResidueCause::SplitMotionRef {
      path: String::from(fixtures::SPLIT_REF),
    }
  );
  assert_eq!(cause.as_str(), "split-motion-ref");
  assert_eq!(cause.get_discarded_path(), Some(fixtures::SPLIT_REF));

  Ok(())
}

#[test]
fn normalizing_a_fragment_names_no_path() -> XrfResult {
  let fragment: OgfNormalization = OgfNormalization::normalize::<XRayByteOrder>(&fixtures::trailing_fragment()?)?;
  let cause: OgfResidueCause = fragment
    .residue
    .expect("Expect the stray bytes to be accounted for")
    .cause;

  assert_eq!(fragment.bytes, fixtures::well_formed()?);
  assert_eq!(cause, OgfResidueCause::TrailingFragment);
  assert_eq!(cause.as_str(), "trailing-fragment");
  assert_eq!(cause.get_discarded_path(), None);

  Ok(())
}

#[test]
fn normalizing_a_well_formed_visual_leaves_nothing_behind() -> XrfResult {
  let clean: OgfNormalization = OgfNormalization::normalize::<XRayByteOrder>(&fixtures::well_formed()?)?;

  assert_eq!(clean.bytes, fixtures::well_formed()?);
  assert!(clean.residue.is_none());

  Ok(())
}

#[test]
fn normalizing_changes_nothing_the_engine_reads() -> XrfResult {
  let original: Vec<u8> = fixtures::split_motion_ref()?;
  let normalization: OgfNormalization = OgfNormalization::normalize::<XRayByteOrder>(&original)?;

  assert!(normalization.is_changed_from(&original));
  assert_eq!(normalization.get_discarded_size(&original), 34);

  normalization.assert_engine_reads_the_same::<XRayByteOrder>(&original)?;

  let clean: Vec<u8> = fixtures::well_formed()?;
  let unchanged: OgfNormalization = OgfNormalization::normalize::<XRayByteOrder>(&clean)?;

  assert!(!unchanged.is_changed_from(&clean));
  assert_eq!(unchanged.get_discarded_size(&clean), 0);

  Ok(())
}

/// The guard is what makes the byte surgery safe to trust: bytes that read back differently are refused, whatever
/// produced them.
#[test]
fn the_guard_refuses_bytes_that_read_back_differently() -> XrfResult {
  let original: Vec<u8> = fixtures::split_motion_ref()?;

  let still_split: OgfNormalization = OgfNormalization {
    bytes: original.clone(),
    residue: None,
  };

  assert!(
    still_split
      .assert_engine_reads_the_same::<XRayByteOrder>(&original)
      .is_err(),
    "Expect bytes still carrying residue to be refused"
  );

  let other_visual: OgfNormalization = OgfNormalization {
    bytes: fixtures::unexplained_residue()?[..fixtures::well_formed()?.len() - 1].to_vec(),
    residue: None,
  };

  assert!(
    other_visual
      .assert_engine_reads_the_same::<XRayByteOrder>(&original)
      .is_err(),
    "Expect bytes that no longer parse as the same visual to be refused"
  );

  Ok(())
}

/// The two doors answer with the same bytes, so a caller wanting only bytes loses nothing by taking the short one.
#[test]
fn the_bytes_door_agrees_with_the_full_one() -> XrfResult {
  for fixture in [
    fixtures::well_formed()?,
    fixtures::split_motion_ref()?,
    fixtures::trailing_fragment()?,
  ] {
    assert_eq!(
      OgfNormalization::normalize_bytes::<XRayByteOrder>(&fixture)?,
      OgfNormalization::normalize::<XRayByteOrder>(&fixture)?.bytes
    );
  }

  Ok(())
}

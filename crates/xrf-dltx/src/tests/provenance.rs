//! What the dialect answers about a field's origin, through the shared record a caller actually reads.

use xrf_error::XrfResult;
use xrf_ltx::{LtxDialect, LtxFieldOrigin, LtxResolution, LtxResolveRequest};

use crate::dltx_dialect::DltxDialect;
use crate::tests::dltx_map_source::DltxMapSource;

/// A base config one mod file patches, which is the smallest tree that has a contest in it.
const TREE: &[(&str, &str)] = &[
  (
    "system.ltx",
    "[wpn_ak74]\ncost = 4000\nammo = 30\nammo_class = ammo_a, ammo_b\n",
  ),
  ("mod_system_a.ltx", "![wpn_ak74]\ncost = 9000\n>ammo_class = ammo_c\n"),
];

fn resolve(request: LtxResolveRequest) -> XrfResult<LtxResolution> {
  DltxDialect.resolve("system.ltx", &DltxMapSource::new(TREE)?, request)
}

#[test]
fn a_patched_field_names_the_mod_file_that_won_it() -> XrfResult {
  let resolved: LtxResolution = resolve(LtxResolveRequest::with_provenance())?;

  let Some(LtxFieldOrigin::Loaded { file, depth, operation }) = resolved.get_origin("wpn_ak74", "cost") else {
    panic!("a patched field to carry a load-order origin");
  };

  assert_eq!(&**file, "mod_system_a.ltx");
  assert_eq!(*depth, -200, "a mod file to outrank the base tree");
  assert_eq!(&**operation, "", "an override writing a plain assignment");

  Ok(())
}

#[test]
fn an_uncontested_field_still_names_its_own_file() -> XrfResult {
  let resolved: LtxResolution = resolve(LtxResolveRequest::with_provenance())?;

  // The reason the variant is not called `Patched`: "the base file wrote it and nothing contested it" is an answer a
  // caller needs as much as naming a patch, and a record that only covered patched fields would leave most of a
  // section unexplained.
  assert_eq!(
    resolved
      .get_origin("wpn_ak74", "ammo")
      .and_then(LtxFieldOrigin::get_file),
    Some("system.ltx")
  );

  assert_eq!(
    resolved
      .get_origin("wpn_ak74", "ammo")
      .and_then(LtxFieldOrigin::get_depth),
    Some(0)
  );

  Ok(())
}

#[test]
fn a_resolution_that_was_not_asked_records_nothing() -> XrfResult {
  let resolved: LtxResolution = resolve(LtxResolveRequest::plain())?;

  // The guard `ltx verify` and `gamedata verify` rest on: they resolve every root of an install and read none of this.
  assert!(resolved.provenance.is_empty(), "a plain resolution to record nothing");

  assert_eq!(
    resolved.ltx.get_from("wpn_ak74", "cost"),
    Some("9000"),
    "while resolving the same values"
  );

  Ok(())
}

#[test]
fn a_list_append_reports_the_statement_that_appended_rather_than_a_plain_assignment() -> XrfResult {
  let resolved: LtxResolution = resolve(LtxResolveRequest::with_provenance())?;

  assert_eq!(
    resolved.ltx.get_from("wpn_ak74", "ammo_class"),
    Some("ammo_a,ammo_b,ammo_c"),
    "the append to reach the resolved value"
  );

  // Resolving rewrites the winning item to a plain assignment, because by then it holds the merged list. Reading the
  // operation off that item would tell a person to look for `ammo_class =` in a file that writes `>ammo_class =`.
  assert_eq!(
    resolved
      .get_origin("wpn_ak74", "ammo_class")
      .and_then(LtxFieldOrigin::get_operation),
    Some(">"),
    "the authored operation to survive into the record"
  );

  assert_eq!(
    resolved
      .get_origin("wpn_ak74", "ammo_class")
      .and_then(LtxFieldOrigin::get_file),
    Some("mod_system_a.ltx")
  );

  Ok(())
}

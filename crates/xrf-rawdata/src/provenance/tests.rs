use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::object::decompile::object_inference::ObjectInference;
use crate::provenance::object_provenance::ObjectProvenance;

/// An empty directory of generated resources for one test.
fn directory(name: &str) -> PathBuf {
  let directory: PathBuf = GeneratedResources::path(&format!("rawdata/provenance/{name}"));

  let _ = fs::remove_dir_all(&directory);

  fs::create_dir_all(&directory).expect("test directory is creatable");

  directory
}

#[test]
fn names_the_sidecar_after_its_object() {
  assert_eq!(
    ObjectProvenance::sidecar_path(&PathBuf::from("objects/crate.object")),
    PathBuf::from("objects/crate.object.xrf.json")
  );
}

#[test]
fn reads_back_the_sidecar_it_writes() -> XrfResult {
  let object: PathBuf = directory("round-trip").join("crate.object");
  let provenance: ObjectProvenance = ObjectProvenance::new(
    "meshes/crate.ogf",
    b"ogf bytes",
    "xrf-cli object decompile",
    "1.0.0",
    vec![ObjectInference::SurfaceNames, ObjectInference::Smoothing],
  );

  fs::write(ObjectProvenance::sidecar_path(&object), provenance.to_json()?).expect("sidecar is writable");

  assert_eq!(ObjectProvenance::read_beside(&object)?, Some(provenance));

  Ok(())
}

#[test]
fn reads_no_provenance_beside_an_authored_object() -> XrfResult {
  assert_eq!(
    ObjectProvenance::read_beside(&directory("authored").join("crate.object"))?,
    None
  );

  Ok(())
}

#[test]
fn refuses_a_sidecar_that_is_not_one() {
  let object: PathBuf = directory("broken").join("crate.object");

  fs::write(ObjectProvenance::sidecar_path(&object), "not json").expect("sidecar is writable");

  assert!(ObjectProvenance::read_beside(&object).is_err());
}

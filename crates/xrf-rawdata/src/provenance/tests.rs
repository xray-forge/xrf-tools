use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::object::compile::object_compile_options::{ObjectCompileOptions, TangentMethod};
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
    TangentMethod::MikkTSpace,
  );

  fs::write(ObjectProvenance::sidecar_path(&object), provenance.to_json()?).expect("sidecar is writable");

  assert_eq!(ObjectProvenance::read_beside(&object)?, Some(provenance));

  Ok(())
}

#[test]
fn compiles_with_the_tangent_method_a_sidecar_records_and_the_editors_without_one() -> XrfResult {
  // A sidecar written before the tangent method was recorded.
  let object: PathBuf = directory("older").join("crate.object");

  fs::write(
    ObjectProvenance::sidecar_path(&object),
    r#"{"source":"meshes/crate.ogf","sourceSha256":"00","tool":"xrf-cli object decompile","toolVersion":"1.0.0",
       "decompiledAt":0,"inferred":[]}"#,
  )
  .expect("sidecar is writable");

  let older: Option<ObjectProvenance> = ObjectProvenance::read_beside(&object)?;
  let mut blender: ObjectProvenance = older.clone().expect("expect the older sidecar read");

  blender.tangents = TangentMethod::MikkTSpace;

  assert_eq!(
    ObjectCompileOptions::following(older.as_ref()).tangents,
    TangentMethod::ActorEditor
  );
  assert_eq!(
    ObjectCompileOptions::following(Some(&blender)).tangents,
    TangentMethod::MikkTSpace
  );
  assert_eq!(
    ObjectCompileOptions::following(None).tangents,
    TangentMethod::ActorEditor
  );

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

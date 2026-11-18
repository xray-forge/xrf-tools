//! Which tangent method a compile asked for by path takes.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::object::compile::object_compile_request::ObjectCompileRequest;
use crate::object::compile::tangent_method::TangentMethod;
use crate::provenance::object_provenance::ObjectProvenance;

fn get_tangents(source: &Path, tangents: Option<TangentMethod>) -> XrfResult<TangentMethod> {
  let request: ObjectCompileRequest = ObjectCompileRequest {
    source: source.to_path_buf(),
    destination: source.with_extension("ogf"),
    tangents,
    is_forced: false,
  };

  Ok(request.get_options()?.tangents)
}

#[test]
fn takes_the_tangent_method_a_sidecar_records_unless_told_otherwise() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("rawdata-object-compile/tangents")?;
  let (decompiled, authored) = (root.join("decompiled.object"), root.join("authored.object"));
  let provenance: ObjectProvenance = ObjectProvenance::new(
    "meshes/decompiled.ogf",
    b"ogf",
    "xrf-cli object decompile",
    "1.0.0",
    Vec::new(),
    TangentMethod::MikkTSpace,
  );

  fs::write(ObjectProvenance::sidecar_path(&decompiled), provenance.to_json()?)?;

  assert_eq!(get_tangents(&decompiled, None)?, TangentMethod::MikkTSpace);
  assert_eq!(
    get_tangents(&decompiled, Some(TangentMethod::ActorEditor))?,
    TangentMethod::ActorEditor
  );
  assert_eq!(get_tangents(&authored, None)?, TangentMethod::ActorEditor);
  assert_eq!(
    get_tangents(&authored, Some(TangentMethod::MikkTSpace))?,
    TangentMethod::MikkTSpace
  );

  Ok(())
}

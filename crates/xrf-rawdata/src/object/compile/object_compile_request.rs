use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_extension::XrayExtension;

use crate::object::compile::object_compile_options::ObjectCompileOptions;
use crate::object::compile::tangent_method::TangentMethod;
use crate::provenance::object_provenance::ObjectProvenance;

/// An object's compile into a file, as a front-end asks for it: the `.object` read, the file written, a detail model
/// where it ends in `.dm` and the OGF otherwise, the tangent method a skeleton takes, and whether an existing file is
/// replaced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectCompileRequest {
  pub source: PathBuf,
  pub destination: PathBuf,
  /// The tangent method a skeleton's vertices are given; none follows the sidecar beside an object decompiled from
  /// one, and is the Actor Editor's otherwise. A static visual and a detail model store no tangents.
  pub tangents: Option<TangentMethod>,
  /// Whether an existing destination is replaced rather than refused before the object is read.
  pub is_forced: bool,
}

impl ObjectCompileRequest {
  /// Whether the destination is a detail model, whose flags and scale range a decompiled object's sidecar keeps.
  pub fn is_detail(&self) -> bool {
    XrayExtension::Dm.matches_path(&self.destination)
  }

  /// The options the compile takes: the tangent method asked for, else the one the source's sidecar records.
  ///
  /// # Errors
  ///
  /// Returns an error when a sidecar stands beside the source but does not read.
  pub fn get_options(&self) -> XrfResult<ObjectCompileOptions> {
    Ok(match self.tangents {
      Some(tangents) => ObjectCompileOptions { tangents },
      None => ObjectCompileOptions::following(ObjectProvenance::read_beside(&self.source)?.as_ref()),
    })
  }
}

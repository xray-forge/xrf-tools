use xrf_editable_object::EditableObjectFile;

use crate::object::compile::object_compile_options::TangentMethod;
use crate::object::decompile::object_inference::ObjectInference;

/// An OGF decompiled into an editable object, and what in it was inferred rather than read.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectDecompile {
  pub object: EditableObjectFile,
  /// Each kind of inference the object carries, once, in a fixed order.
  pub inferred: Vec<ObjectInference>,
  /// The tangent method the OGF's basis was made with, by the exporter its description names, which a compile follows
  /// to give the basis back.
  pub tangents: TangentMethod,
}

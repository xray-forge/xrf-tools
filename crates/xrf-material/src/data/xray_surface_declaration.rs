use serde::Serialize;

/// What the shader library says about a surface, as the renderer would read it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum XraySurfaceDeclaration {
  /// The surface names no shader at all, so there is nothing to look up.
  Undeclared,
  /// No `shaders.xr` in any searched root, so nothing can be said about any surface of this model.
  NoLibrary,
  /// A library was located and could not be read as one.
  Unreadable { reason: String },
  /// The library holds no blender of that name.
  Undefined,
  /// A blender whose class this crate does not derive a draw mode for, such as a particle or screen space class a
  /// mesh has no business naming, or one a mod's renderer added.
  Unmodelled { class: String },
  /// A blender whose class decides the surface from the knobs below.
  Described {
    /// The class tag, as `Blender_CLSID.h` spells it: `MODEL`, `MODELEbB`, `LM_AREF`.
    class: String,
    /// The class's own alpha switch, or `None` for a class that writes none and is therefore always opaque.
    is_alpha_used: Option<bool>,
    /// The authored `Alpha ref`, or `None` for a class that writes none.
    alpha_reference: Option<u8>,
    /// `Strict sorting`, which every class writes and which pushes a model surface out of the deferred path.
    is_strict_sorting: bool,
  },
}

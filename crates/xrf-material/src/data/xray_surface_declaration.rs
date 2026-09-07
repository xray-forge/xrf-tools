use serde::Serialize;

/// What the shader library says about a surface, as the renderer would read it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum XraySurfaceDeclaration {
  /// No `shaders.xr` in any searched root, so nothing can be said about any surface of this model.
  NoLibrary,
  /// A library was located and could not be read as one.
  Unreadable { reason: String },
  /// The library holds no blender of that name.
  ///
  /// What the engine reports as `! Shader '%s' not found in library` before falling back to the default shader
  /// (`Layers/xrRender/ResourceManager.cpp:40`), so the surface still draws - opaque, and not as authored.
  Undefined,
  /// A blender whose class this crate does not derive a draw mode for, such as a particle or screen space class a
  /// mesh has no business naming, or one a mod's renderer added.
  Unmodelled { class: String },
  /// A blender whose class decides the surface from the knobs below.
  Described {
    /// The class tag, as `Blender_CLSID.h` spells it: `MODEL`, `MODELEbB`, `LM_AREF`.
    class: String,
    /// The class's own alpha switch, or `None` for a class that writes none and is therefore always opaque.
    ///
    /// The engine spells it differently per class - `Use alpha-channel` for `B_MODEL`, `Alpha-blend` for
    /// `B_DEFAULT_AREF`, `Alpha-Blend` for `B_MODEL_EbB` - and the reader keeps those apart.
    is_alpha_used: Option<bool>,
    /// The authored `Alpha ref`, or `None` for a class that writes none.
    ///
    /// Not necessarily what the surface tests against: see [`crate::XraySurfaceDraw::AlphaTested`].
    alpha_reference: Option<u8>,
    /// `Strict sorting`, which every class writes and which pushes a model surface out of the deferred path.
    is_strict_sorting: bool,
  },
}

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
  /// A renderer shader script, which the engine looks up before the library and uses instead of it when it is
  /// there (`CResourceManager::Create` asks `_lua_HasShader` first).
  Scripted {
    /// Logical path of the script the pass was read from, so a reader can open the same file.
    script: String,
    /// The pass the engine compiles as the base element, named by its function.
    function: String,
    /// Whether the pass is composited rather than written.
    is_blended: bool,
    /// Whether the pass discards texels against its reference.
    is_alpha_tested: bool,
    /// The reference it discards against, which the engine compares with `D3DCMP_GREATER`: a texel is kept where its
    /// alpha is greater than this, so the usual `aref(true, 0)` of a wall mark discards every fully transparent one
    /// rather than discarding nothing.
    alpha_reference: u8,
    /// Whether the pass writes depth, which a mark laid on a wall does not.
    is_depth_written: bool,
    /// Whether the pass is a wall mark, which the engine draws with a depth bias of its own.
    is_wallmark: bool,
  },
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

use serde::Serialize;

use crate::data::xray_bump_mode::XrayBumpMode;

/// What a texture's descriptor says about its bump, as the engine reads it.
///
/// Every variant but [`Self::Declared`] renders flat, and they are six different things an author did. Collapsing them
/// into "no bump" is what leaves a modder opening a hex editor: a descriptor that looks complete and is skipped for
/// its type, and a descriptor that is simply absent, are the same surface in the viewport and opposite fixes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum XrayMaterialDeclaration {
  /// No `.thm` sits beside the texture in any searched root.
  NoDescriptor,
  /// A `.thm` was located and could not be read as one.
  Unreadable { reason: String },
  /// The descriptor's texture type is one `LoadTHM` skips whole (`TextureDescrManager.cpp`), so whatever its bump
  /// chunk declares is never read. `declared_bump` is that chunk's used name, when it has one, so the panel can say
  /// the declaration is fine and the type is not.
  TypeDisqualified {
    texture_type: u32,
    label: String,
    declared_bump: Option<String>,
  },
  /// The descriptor carries no bump chunk at all.
  NoBumpChunk,
  /// The bump chunk's mode is `none`, or the reserved value the engine clamps to it (`ETextureParams.cpp:77`).
  Disabled { mode: u32 },
  /// The mode asks for a bump and the name is empty, so `bump_exist()` is false and the flat shader is selected.
  EmptyName { mode: XrayBumpMode },
  /// A bump the renderer will try to bind.
  Declared { mode: XrayBumpMode, name: String },
}

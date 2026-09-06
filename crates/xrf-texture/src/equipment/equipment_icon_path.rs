//! Where a `$inventory_icon_path` points, when it does not point at a plain file beside the section.
//!
//! Two of the three roots are named by a leading marker rather than by a path, so an icon can be authored against the
//! game data or against an extension without either of them knowing where the other was installed.

/// Leading token of a custom icon path that resolves against the gamedata root.
pub(crate) const LTX_PATH_GAMEDATA_MARKER: char = '~';

/// The gamedata marker together with the separator that follows it, for stripping the prefix.
pub(crate) const LTX_PATH_GAMEDATA_MARKER_PREFIX: &str = "~\\";

/// Leading token of a custom icon path that resolves against the extensions directory.
pub(crate) const LTX_PATH_EXTENSION_MARKER: char = '#';

/// The extension marker together with the separator that follows it, for stripping the prefix.
pub(crate) const LTX_PATH_EXTENSION_MARKER_PREFIX: &str = "#\\";

pub(crate) const RESOURCES_DIRECTORY: &str = "resources";

pub(crate) const TEXTURES_DIRECTORY: &str = "textures";

pub(crate) const EXTENSIONS_DIRECTORY: &str = "extensions";

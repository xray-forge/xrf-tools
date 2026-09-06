use xrf_dds::DdsMipmaps;

/// Mip chain written into a packed UI sprite sheet.
///
/// None, following vanilla.
/// The engine cannot use a chain here anyway, since the UI is authored on a 1024x768 canvas
/// that is scaled up to the real resolution, so a sheet is magnified rather than minified
/// and only level 0 is ever sampled.
pub const UI_MIPMAPS: DdsMipmaps = DdsMipmaps::Disabled;

/// Number of mip levels [`UI_MIPMAPS`] produces, for comparing against an existing sheet.
pub const UI_MIPMAP_LEVELS: u32 = 1;

/// Block edge a BC compressed sheet is sized to.
///
/// Encoders and the engine both cope with an unaligned sheet by padding it to whole blocks, so this is
/// convention rather than necessity.
pub const DDS_BLOCK_ALIGNMENT: u32 = 4;

/// Inventory icon coordinates are expressed in grid cells rather than pixels, so every `inv_grid_*`
/// value is multiplied by this to reach the sprite sheet. Note it is not a multiple of 4, so icon
/// boundaries do not align with the 4x4 blocks of BC compressed sheets.
pub const INVENTORY_ICON_GRID_SQUARE_BASE: u32 = 50;
// pub const CHARACTER_ICON_GRID_SQUARE_BASE: u32 = 64;
// pub const CHARACTER_GRID_SQUARE_BASE: u32 = 2;
// pub const CHARACTER_FULL_GRID_SQUARE_BASE: u32 = 2;

/// Marks a section as owning an inventory icon, gating both packing and unpacking.
pub const LTX_FIELD_INVENTORY_ICON: &str = "$inventory_icon";

/// Overrides where the icon of a section is read from, instead of `<section>.dds` in the source dir.
pub const LTX_FIELD_INVENTORY_ICON_PATH: &str = "$inventory_icon_path";

pub const LTX_FIELD_INV_GRID_X: &str = "inv_grid_x";

pub const LTX_FIELD_INV_GRID_Y: &str = "inv_grid_y";

pub const LTX_FIELD_INV_GRID_WIDTH: &str = "inv_grid_width";

pub const LTX_FIELD_INV_GRID_HEIGHT: &str = "inv_grid_height";

/// Root node of a ui texture description file, wrapping all of its `file` nodes.
pub const XML_TAG_WINDOW: &str = "w";

pub const XML_TAG_FILE: &str = "file";

pub const XML_TAG_TEXTURE: &str = "texture";

pub const XML_ATTRIBUTE_ID: &str = "id";

pub const XML_ATTRIBUTE_NAME: &str = "name";

pub const XML_ATTRIBUTE_X: &str = "x";

pub const XML_ATTRIBUTE_Y: &str = "y";

pub const XML_ATTRIBUTE_WIDTH: &str = "width";

pub const XML_ATTRIBUTE_HEIGHT: &str = "height";

pub const DDS_EXTENSION: &str = "dds";

pub const PNG_EXTENSION: &str = "png";

/// Leading token of a custom icon path that resolves against the gamedata root.
pub const LTX_PATH_GAMEDATA_MARKER: char = '~';

/// The gamedata marker together with the separator that follows it, for stripping the prefix.
pub const LTX_PATH_GAMEDATA_MARKER_PREFIX: &str = "~\\";

/// Leading token of a custom icon path that resolves against the extensions directory.
pub const LTX_PATH_EXTENSION_MARKER: char = '#';

/// The extension marker together with the separator that follows it, for stripping the prefix.
pub const LTX_PATH_EXTENSION_MARKER_PREFIX: &str = "#\\";

pub const RESOURCES_DIRECTORY: &str = "resources";

pub const TEXTURES_DIRECTORY: &str = "textures";

pub const EXTENSIONS_DIRECTORY: &str = "extensions";

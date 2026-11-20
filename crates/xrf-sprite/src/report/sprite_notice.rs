use std::path::PathBuf;

use xrf_texture::ImageFileReshape;

/// Something a sprite run passed over or noticed without failing: what a strict run would have refused, or what
/// changed beyond the pixels it was asked to write.
///
/// A run hands each one to its caller as it happens, in the order it met them; [`crate::SpriteRunPrinter`] renders them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpriteNotice {
  /// An equipment icon that could not be read, left out of the sheet.
  IconUnreadable {
    path: PathBuf,
    section: String,
    reason: String,
  },
  /// Two sections claim one grid slot with different icons, so the one packed last is the one the sheet keeps.
  SlotConflict {
    x: u32,
    y: u32,
    previous: String,
    section: String,
  },
  /// An equipment icon whose slot lies outside the sheet it is cut from, left unwritten.
  IconOutsideSheet { section: String },
  /// A described sprite that could not be read, left out of its sheet.
  SpriteUnreadable {
    id: String,
    file: String,
    sheet: PathBuf,
    reason: String,
  },
  /// A described sprite reaching past the sheet it is cut from, left unwritten.
  SpriteOutsideSheet {
    id: String,
    max_x: u32,
    max_y: u32,
    width: u32,
    height: u32,
    sheet: PathBuf,
  },
  /// A described sheet that could not be read, passed over whole.
  SheetUnreadable { path: PathBuf, reason: String },
  /// A sheet written over one of another shape, or over one that could not be read to compare; the new shape is given.
  SheetReshaped {
    path: PathBuf,
    width: u32,
    height: u32,
    mipmap_levels: u32,
    replaced: ImageFileReshape,
  },
  /// A description file that is not valid XML, read as describing nothing.
  DescriptionUnparsable { path: PathBuf, reason: String },
  /// A description whose root is not the `<w>` window element, read as describing nothing.
  DescriptionWithoutWindow { path: PathBuf },
  /// A `<file>` node without a name, passed over.
  FileWithoutName,
  /// A `<file>` node declaring no usable texture, passed over.
  FileWithoutTextures { name: String, path: PathBuf },
  /// A `<texture>` node missing what a sprite needs, passed over with the attributes it did have.
  TextureNodeIncomplete {
    id: Option<String>,
    attributes: Vec<(String, String)>,
  },
}

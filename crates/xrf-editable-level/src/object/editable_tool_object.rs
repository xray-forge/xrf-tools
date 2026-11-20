use xrf_error::XrfResult;
use xrf_ini::IniFile;

use crate::part::editable_part_header::EditablePartHeader;
use crate::part::editable_part_object::EditablePartObject;

/// A typed object of one of the Level Editor's object tools: read from its `[object_N]` section and the sections named
/// after it, written back as its `SaveLTX` does, with what the tool writes into its part's `[main]`.
pub trait EditableToolObject: Sized {
  /// The editor's `ObjClassID` of the tool's objects, written as each object's `clsid`.
  const CLASS_ID: i32;

  /// The tool's `ClassName`, which names its part: `<name>.part` in the scene's directory (`EScene::LevelPartName`).
  const TOOL_NAME: &'static str;

  /// The tool's `*_TOOLS_VERSION`, its part's `[main]` version; its `LoadLTX` refuses any other. Every object tool of
  /// the CoP SDK writes 0.
  const TOOL_VERSION: u32 = 0;

  /// The `[main]` key the tool writes its version under: `version` for every tool but the glow tool
  /// (`glow_tool_version`, `ESceneGlowToolsIO.cpp`).
  const TOOL_VERSION_KEY: &'static str = EditablePartHeader::VERSION_KEY;

  /// Whether the tool keeps `flags` in its part's `[main]`, which its `LoadLTX` then requires.
  const KEEPS_FLAGS: bool = false;

  /// # Errors
  ///
  /// Returns an error when a field the object needs is missing or malformed.
  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self>;

  /// Writes the object's fields under `section`, and any section named after it, `clsid` aside.
  fn write_to_ini(&self, ini: &mut IniFile, section: &str);

  /// Writes the data a new scene's tool saves beyond its header, which its `LoadLTX` requires; nothing for most tools.
  /// [`crate::EditableObjectPart::new`] starts from it, so the editor opens a new part.
  fn write_new_tool_data(_tool_data: &mut IniFile) {}
}

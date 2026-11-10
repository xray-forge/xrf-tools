use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniFormat};
use xrf_utils::format_path;

use crate::level::editable_build_params::EditableBuildParams;
use crate::level::editable_level_camera::EditableLevelCamera;
use crate::level::editable_level_guid::EditableLevelGuid;
use crate::level::editable_level_options::EditableLevelOptions;
use crate::level::editable_level_value::EditableLevelValue;

/// A scene's main file, the `.level` under `$maps$` the Level Editor opens (`EScene::SaveLTX`): the scene's GUID,
/// version, level options, build parameters, camera and owner tag. Each tool's objects are in the scene's parts.
///
/// The file is kept whole as the [`IniFile`] it was read as, so it writes back byte for byte; the typed getters and
/// setters read and write its sections in the editor's own layout.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditableLevelFile {
  pub ini: IniFile,
}

impl EditableLevelFile {
  /// `CURRENT_FILE_VERSION`, the only scene version `EScene::LoadLTX` opens.
  pub const VERSION: u32 = 5;

  /// The extension of a scene's main file.
  pub const EXTENSION: &str = "level";

  /// A new scene's main file with every section `EScene::SaveLTX` writes and `EScene::LoadLTX` reads: the version, the
  /// GUID its parts carry, the level options and build parameters (their reserved fields zero), the camera, and the
  /// owner tag.
  pub fn new(
    guid: &EditableLevelGuid,
    options: &EditableLevelOptions,
    build_params: &EditableBuildParams,
    camera: &EditableLevelCamera,
    owner: &str,
    create_time: u32,
  ) -> Self {
    let mut file: Self = Self::default();

    file.ini.section_mut("version").set("value", Self::VERSION.to_string());
    file.set_guid(guid);
    file.set_level_options(options);
    file.set_build_params(build_params);

    // `b_params::LoadLTX` reads the reserved fields too, and the SDK's editor refuses a scene lacking one ("Can't find
    // variable reserved_1 in [build_params]"): every GSC scene of SDK 0.7 holds the quality's and all six floats.
    let build = file.ini.section_mut(EditableBuildParams::SECTION);

    build.set("light_quality_reserved", "0");

    for index in 0..EditableBuildParams::RESERVED_FLOATS {
      build.set(format!("reserved_{index}"), EditableLevelValue::format_f32(0.0));
    }

    camera.write_to_section(file.ini.section_mut(EditableLevelCamera::SECTION));

    let section = file.ini.section_mut("level_tag");

    section.set("owner", owner);
    section.set("create_time", create_time.to_string());

    file
  }

  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a scene [`Self::read_from_bytes`] reads.
  pub fn read_from_path<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    let bytes: Vec<u8> = std::fs::read(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level scene was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?;

    Self::read_from_bytes(&bytes)
  }

  /// Reads a scene's main file and checks what the editor checks before opening it: the version, and a GUID its parts
  /// can be matched against.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a file [`IniFormat::XRAY_SAVE_AS`] reads, the version is not
  /// [`Self::VERSION`], or the GUID is missing or malformed.
  pub fn read_from_bytes(bytes: &[u8]) -> XrfResult<Self> {
    let file: Self = Self {
      ini: IniFile::read_from_bytes_as(bytes, &IniFormat::XRAY_SAVE_AS)?,
    };
    let version: u32 = file.get_version()?;

    if version != Self::VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "Level scene version {version} is not {}, the only one the Level Editor opens",
        Self::VERSION
      )));
    }

    file.get_guid()?;

    Ok(file)
  }

  /// Writes the file through a staged file, creating the directories leading to it.
  ///
  /// # Errors
  ///
  /// Returns an error as [`Self::write_to_bytes`] does, or when the file cannot be written.
  pub fn write_to_path<P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    self.ini.write_to_path_as(path, &IniFormat::XRAY_SAVE_AS)
  }

  /// # Errors
  ///
  /// Returns an error when the file would not read back as itself (`IniFile::write_to_string_as`) or does not encode
  /// as Windows-1251.
  pub fn write_to_bytes(&self) -> XrfResult<Vec<u8>> {
    self.ini.write_to_bytes_as(&IniFormat::XRAY_SAVE_AS)
  }

  /// # Errors
  ///
  /// Returns an error when `[version] value` is missing or not an unsigned integer.
  pub fn get_version(&self) -> XrfResult<u32> {
    self.ini.section_required("version")?.get_parsed::<u32>("value")
  }

  /// # Errors
  ///
  /// Returns an error when the `[guid]` section is missing or malformed.
  pub fn get_guid(&self) -> XrfResult<EditableLevelGuid> {
    EditableLevelGuid::read_from_section(self.ini.section_required(EditableLevelGuid::SECTION)?)
  }

  pub fn set_guid(&mut self, guid: &EditableLevelGuid) {
    guid.write_to_section(self.ini.section_mut(EditableLevelGuid::SECTION));
  }

  /// # Errors
  ///
  /// Returns an error when `[level_options]` is missing or a field of it is missing or malformed.
  pub fn get_level_options(&self) -> XrfResult<EditableLevelOptions> {
    EditableLevelOptions::read_from_section(self.ini.section_required(EditableLevelOptions::SECTION)?)
  }

  pub fn set_level_options(&mut self, options: &EditableLevelOptions) {
    options.write_to_section(self.ini.section_mut(EditableLevelOptions::SECTION));
  }

  /// # Errors
  ///
  /// Returns an error when `[build_params]` is missing or a field of it is missing or malformed.
  pub fn get_build_params(&self) -> XrfResult<EditableBuildParams> {
    EditableBuildParams::read_from_section(self.ini.section_required(EditableBuildParams::SECTION)?)
  }

  pub fn set_build_params(&mut self, params: &EditableBuildParams) {
    params.write_to_section(self.ini.section_mut(EditableBuildParams::SECTION));
  }

  /// Where the editor's camera stood when the scene was saved.
  ///
  /// # Errors
  ///
  /// Returns an error when `[camera]` is missing, or its `pos` or `hpb` is not three floats.
  pub fn get_camera(&self) -> XrfResult<EditableLevelCamera> {
    EditableLevelCamera::read_from_section(self.ini.section_required(EditableLevelCamera::SECTION)?)
  }
}

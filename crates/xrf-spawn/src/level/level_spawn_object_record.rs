use byteorder::ByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxImportExport, LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};
use xrf_utils::{decode_bytes_from_base64, encode_bytes_to_base64};

use crate::constants::NET_ACTION_SPAWN;
use crate::data::alife::alife_object::AlifeObject;
use crate::level::level_spawn_export::LevelSpawnExport;
use crate::level::level_spawn_object::LevelSpawnObject;

/// An object's record in a level list's unpacked file, a section named by its place in the list.
///
/// An object the typed read accepts is written as a root spawn writes an ALife object ([`AlifeObject::export`]), so the
/// two unpacked forms share their keys. One it refuses is written verbatim: its head under the keys the typed form uses
/// for the same values, the rest of its packet in base64, and the reason, under a `form` key naming it, which a typed
/// record never carries. Either holds the chunk the object was found under only where that differs from its place.
impl LevelSpawnObject {
  const FORM: &'static str = "form";
  const VERBATIM_FORM: &'static str = "verbatim";
  const REASON: &'static str = "reason";
  const PAYLOAD: &'static str = "payload";
  const CHUNK_ID: &'static str = "chunk_id";

  /// Writes the object's record as section `index`, counting into `export` the form it was written in.
  ///
  /// # Errors
  ///
  /// Returns an error when a typed object's fields cannot be written.
  pub(crate) fn export_record<T: ByteOrder>(
    &self,
    index: usize,
    ltx: &mut impl LtxWrite,
    export: &mut LevelSpawnExport,
  ) -> XrfResult {
    let section_name: String = index.to_string();

    match self.read_typed_record::<T>() {
      Ok(object) => {
        object.export(&section_name, ltx)?;
        export.count_typed();
      }
      Err(reason) => {
        ltx
          .with_section(&section_name)
          .set(Self::FORM, Self::VERBATIM_FORM)
          .set(Self::REASON, &reason)
          .set("section", &self.section)
          .set("name", &self.name)
          .set("script_game_id", self.script_game_id)
          .set("script_rp", self.script_rp)
          .set("position", self.position)
          .set("direction", self.direction)
          .set(Self::PAYLOAD, encode_bytes_to_base64(&self.payload));
        export.count_verbatim(&reason);
      }
    }

    if usize::try_from(self.id).ok() != Some(index) {
      ltx.with_section(&section_name).set(Self::CHUNK_ID, self.id);
    }

    Ok(())
  }

  /// Reads the object a record holds, listed under its `chunk_id` or else the index its section is named by. A verbatim
  /// record packs from its payload as written, whether or not that would now read typed, and its reason is not read.
  ///
  /// # Errors
  ///
  /// Returns an error when the section is not named by an index, names a form other than none or `verbatim`, or its
  /// fields do not read.
  pub(crate) fn import_record<T: ByteOrder, L: LtxRead>(section_name: &str, ltx: &L) -> XrfResult<Self> {
    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Level spawn record '{section_name}' should be defined in ltx file"
      ))
    })?;
    let id: u32 = match section.read_optional_field(Self::CHUNK_ID)? {
      Some(id) => id,
      None => section_name.parse().map_err(|_| {
        XrfError::new_parsing_error(format!(
          "Level spawn record '{section_name}' is not named by its place in the list, and holds no {}",
          Self::CHUNK_ID
        ))
      })?,
    };

    match section.get(Self::FORM) {
      None => Self::from_alife_object::<T>(id, &AlifeObject::import(section_name, ltx)?),
      Some(Self::VERBATIM_FORM) => Ok(Self {
        id,
        net_action: NET_ACTION_SPAWN,
        section: section.read_field("section")?,
        name: section.read_field("name")?,
        script_game_id: section.read_field("script_game_id")?,
        script_rp: section.read_field("script_rp")?,
        position: section.read_field("position")?,
        direction: section.read_field("direction")?,
        payload: decode_bytes_from_base64(&section.read_field::<String>(Self::PAYLOAD)?)?,
      }),
      Some(form) => Err(XrfError::new_parsing_error(format!(
        "Level spawn record '{section_name}' is of form '{form}', expected none for a typed object or '{}'",
        Self::VERBATIM_FORM
      ))),
    }
  }

  /// The object typed, when the typed read accepts it and its typed form writes back the bytes it holds; otherwise the
  /// reason it is written verbatim ([`Self::format_reason`]).
  fn read_typed_record<T: ByteOrder>(&self) -> Result<AlifeObject, String> {
    let object: AlifeObject = self
      .read_alife_object::<T>()
      .map_err(|error| self.format_reason(&error.to_string()))?;
    let is_written_back: bool = Self::from_alife_object::<T>(self.id, &object)
      .and_then(|written| Ok(written.encode_packet::<T>()? == self.encode_packet::<T>()?))
      .map_err(|error| self.format_reason(&format!("Its typed form does not write back: {error}")))?;

    if !is_written_back {
      return Err(String::from(
        "Its typed form writes back other bytes than it was read from",
      ));
    }

    Ok(object)
  }

  /// The reason a refusal's `message` gives, as the object's record states it. It leaves out the object's name, which
  /// the record holds beside it, so objects refused alike share one. It is kept to one line and holds no double quote,
  /// as a record's value must: a message quoting a packet's text can carry an odd number, and a value leaving a quote
  /// open is refused on import, since the engine would read the lines after it into it.
  fn format_reason(&self, message: &str) -> String {
    message
      .replace(&format!(" '{}'", self.name), "")
      .replace(['\r', '\n'], " ")
      .replace('"', "'")
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use xrf_chunk::XRayByteOrder;
  use xrf_error::XrfResult;
  use xrf_ltx::{LtxRecordWriter, LtxSectionWrite, LtxWrite};
  use xrf_test_utils::GeneratedResources;

  use crate::fixtures::LevelSpawnObjects;
  use crate::level::level_spawn_file::LevelSpawnFile;
  use crate::level::level_spawn_object::LevelSpawnObject;

  /// A refusal quoting text the packet holds, as a script class's absent state does, can carry an odd number of double
  /// quotes, and still writes a record that reads back.
  #[test]
  fn writes_a_reason_quoting_a_value_as_a_record_that_reads_back() -> XrfResult {
    let object: LevelSpawnObject = LevelSpawnObjects::shadow_of_chernobyl_graph_point(0, "l04u_graph_point_0")?;
    let reason: String = object.format_reason(
      r#"Invalid error: se_test holds game "a\"b" where a spawn holds "nil": simulation state that is not read"#,
    );
    let path: PathBuf = GeneratedResources::create_dir("level_spawn_object_record/quoted_reason")?;
    let mut ltx: LtxRecordWriter = LtxRecordWriter::new();

    object.export_record::<XRayByteOrder>(0, &mut ltx, &mut Default::default())?;
    ltx.with_section("0").set("reason", &reason);
    ltx.write_to_path(path.join(LevelSpawnFile::UNPACKED_FILE_NAME))?;

    assert_eq!(
      reason,
      "Invalid error: se_test holds game 'a\\'b' where a spawn holds 'nil': simulation state that is not read"
    );
    assert_eq!(
      LevelSpawnFile::import_from_path::<XRayByteOrder, _>(&path)?,
      LevelSpawnFile { objects: vec![object] }
    );

    Ok(())
  }

  /// The object's name is left out wherever a message quotes it, and the message is kept to one line.
  #[test]
  fn formats_a_reason_without_the_objects_name_on_one_line() -> XrfResult {
    let object: LevelSpawnObject = LevelSpawnObjects::shadow_of_chernobyl_graph_point(0, "esc_wolf")?;

    assert_eq!(
      object.format_reason("Spawn packet 'esc_wolf' state size 1\r\nis less than 'esc_wolf_2'"),
      "Spawn packet state size 1  is less than 'esc_wolf_2'"
    );

    Ok(())
  }
}

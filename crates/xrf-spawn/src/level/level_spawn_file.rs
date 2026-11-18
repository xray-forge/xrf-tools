use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxRecordReader, LtxRecordWriter};
use xrf_utils::{StagedFile, format_path};

use crate::level::level_spawn_export::LevelSpawnExport;
use crate::level::level_spawn_object::LevelSpawnObject;

/// The objects a level spawns, `level.spawn`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnFile {
  pub objects: Vec<LevelSpawnObject>,
}

impl LevelSpawnFile {
  /// The file's name in a level's directory.
  pub const FILE_NAME: &'static str = "level.spawn";

  /// The one file of the list's unpacked folder, which holds every object in the list's order.
  pub const UNPACKED_FILE_NAME: &'static str = "level_spawns.ltx";

  /// Reads a level spawn list from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a spawn list this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level spawn list was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a level spawn list from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a spawn list this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived list arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a spawn list this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk does not open with a spawn packet, which is what tells this format from the set
  /// that shares its extension.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Self::read_from_chunks::<T, D>(reader.read_children()?)
  }

  /// Reads from a file's top-level chunks already walked, which is how a caller that first asked
  /// [`SpawnFileKind`](crate::SpawnFileKind) avoids walking them again.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk does not open with a spawn packet.
  pub fn read_from_chunks<T: ByteOrder, D: ChunkDataSource>(chunks: Vec<ChunkReader<D>>) -> XrfResult<Self> {
    let mut objects: Vec<LevelSpawnObject> = Vec::with_capacity(chunks.len());

    for mut chunk in chunks {
      let id: u32 = chunk.id;

      objects.push(LevelSpawnObject::read::<T, D>(id, &mut chunk)?);
    }

    Ok(Self { objects })
  }

  /// Writes the list back as the run of packets the engine walks.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for object in &self.objects {
      let mut chunk: ChunkWriter = ChunkWriter::new();

      object.write::<T>(&mut chunk)?;
      chunk.flush_chunk_into::<T>(&mut writer.buffer, object.id)?;
    }

    Ok(())
  }

  /// Writes the list to `path`, making the directories above it: encoded whole first and staged beside it, so a
  /// failure leaves whatever was there before.
  ///
  /// # Errors
  ///
  /// Returns an error when the list cannot be encoded or the file cannot be written.
  pub fn write_to_path<T: ByteOrder, P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    self.write::<T>(&mut writer)?;

    Ok(StagedFile::write_creating_directories(path.as_ref(), &writer.buffer)?)
  }

  /// Imports the list's unpacked folder at `path`: its objects in the order its file holds them
  /// (`LevelSpawnObject::import_record`).
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or a record does not import.
  pub fn import_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    let mut objects: Vec<LevelSpawnObject> = Vec::new();

    LtxRecordReader::read_path(path.as_ref().join(Self::UNPACKED_FILE_NAME), |record| {
      objects.push(LevelSpawnObject::import_record::<T, _>(record.name(), record)?);

      Ok(())
    })?;

    Ok(Self { objects })
  }

  /// Exports the list into the folder `path` as one record per object, named by its place in the list
  /// (`LevelSpawnObject::export_record`), answering how many were written typed and how many verbatim.
  ///
  /// # Errors
  ///
  /// Returns an error when an object's record cannot be written, or the file cannot be.
  pub fn export_to_path<T: ByteOrder, P: AsRef<Path>>(&self, path: &P) -> XrfResult<LevelSpawnExport> {
    let mut ltx: LtxRecordWriter = LtxRecordWriter::new();
    let mut export: LevelSpawnExport = LevelSpawnExport::default();

    for (index, object) in self.objects.iter().enumerate() {
      object.export_record::<T>(index, &mut ltx, &mut export)?;
    }

    ltx.write_to_path(path.as_ref().join(Self::UNPACKED_FILE_NAME))?;

    Ok(export)
  }
}

impl LevelSpawnFile {
  /// What the level spawns, counted by the section each object is built from.
  pub fn get_sections(&self) -> BTreeMap<&str, usize> {
    let mut counted: BTreeMap<&str, usize> = BTreeMap::new();

    for object in &self.objects {
      *counted.entry(object.section.as_str()).or_default() += 1;
    }

    counted
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_chunk::{ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::GeneratedResources;
  use xrf_utils::encode_bytes_to_base64;

  use crate::fixtures::LevelSpawnObjects;
  use crate::level::level_spawn_export::LevelSpawnExport;
  use crate::level::level_spawn_file::LevelSpawnFile;
  use crate::level::level_spawn_object::LevelSpawnObject;
  use crate::level::level_spawn_verbatim_group::LevelSpawnVerbatimGroup;

  /// The bytes the list packs into.
  fn packed(list: &LevelSpawnFile) -> XrfResult<Vec<u8>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    list.write::<XRayByteOrder>(&mut writer)?;

    writer.flush_raw_into_buffer()
  }

  /// The list unpacked into a folder of its own and imported back, with what the unpack wrote and the text it wrote.
  fn unpack_and_import(list: &LevelSpawnFile, folder: &str) -> XrfResult<(LevelSpawnFile, LevelSpawnExport, String)> {
    let path: PathBuf = GeneratedResources::create_dir(folder)?;
    let export: LevelSpawnExport = list.export_to_path::<XRayByteOrder, _>(&path)?;
    let text: String = fs::read_to_string(path.join(LevelSpawnFile::UNPACKED_FILE_NAME))?;
    let imported: LevelSpawnFile = LevelSpawnFile::import_from_path::<XRayByteOrder, _>(&path)?;

    Ok((imported, export, text))
  }

  /// Typed objects of both games unpack with the keys a root spawn's ALife objects use, no more, and pack back into the
  /// bytes they were read from.
  #[test]
  fn unpacks_typed_objects_and_packs_them_back_byte_for_byte() -> XrfResult {
    let list: LevelSpawnFile = LevelSpawnFile {
      objects: vec![
        LevelSpawnObjects::bolt(0)?,
        LevelSpawnObjects::clear_sky_smart_cover(1)?,
        LevelSpawnObjects::clear_sky_stalker(2)?,
      ],
    };
    let (imported, export, text) = unpack_and_import(&list, "level_spawn_file/typed")?;

    assert_eq!(
      export,
      LevelSpawnExport {
        typed_objects: 3,
        verbatim_objects: Vec::new(),
      }
    );
    assert!(text.starts_with("[0]\r\n"), "{text}");
    assert!(text.contains("\r\nsection = smart_cover\r\n"), "{text}");
    assert!(!text.contains("form ="), "{text}");
    assert!(!text.contains("chunk_id ="), "{text}");
    assert_eq!(imported, list);
    assert_eq!(packed(&imported)?, packed(&list)?);

    Ok(())
  }

  /// An object the typed read refuses is written as its head, its payload and the reason, which names no object, so
  /// objects refused alike count as one reason; it packs back from the payload as written.
  #[test]
  fn unpacks_an_object_the_typed_read_refuses_verbatim_with_its_reason() -> XrfResult {
    let list: LevelSpawnFile = LevelSpawnFile {
      objects: vec![
        LevelSpawnObjects::shadow_of_chernobyl_graph_point(0, "l04u_graph_point_0")?,
        LevelSpawnObjects::bolt(1)?,
        LevelSpawnObjects::shadow_of_chernobyl_graph_point(2, "l04u_graph_point_1")?,
      ],
    };
    let reason: &str =
      "Assertion error: Spawn packet is of spawn version 118, Shadow of Chernobyl's, whose layout is not supported";
    let (imported, export, text) = unpack_and_import(&list, "level_spawn_file/verbatim")?;

    assert_eq!(
      export,
      LevelSpawnExport {
        typed_objects: 1,
        verbatim_objects: vec![LevelSpawnVerbatimGroup {
          reason: String::from(reason),
          objects: 2,
        }],
      }
    );
    assert_eq!(export.sum_verbatim_objects(), 2);
    assert!(
      text.starts_with(&format!(
        "[0]\r\nform = verbatim\r\nreason = {reason}\r\nsection = graph_point\r\nname = l04u_graph_point_0\r\n\
         script_game_id = 0\r\nscript_rp = 255\r\nposition = -4,0.5,7.75\r\ndirection = 0,0,0\r\npayload = {}\r\n",
        encode_bytes_to_base64(&list.objects[0].payload)
      )),
      "{text}"
    );
    assert_eq!(imported, list);
    assert_eq!(packed(&imported)?, packed(&list)?);

    Ok(())
  }

  /// A list whose chunks are not numbered by their place keeps each object's chunk, writing it only where it differs.
  #[test]
  fn keeps_chunk_ids_other_than_the_index() -> XrfResult {
    let list: LevelSpawnFile = LevelSpawnFile {
      objects: vec![
        LevelSpawnObjects::bolt(5)?,
        LevelSpawnObjects::shadow_of_chernobyl_graph_point(1, "l04u_graph_point_0")?,
        LevelSpawnObjects::clear_sky_stalker(0)?,
      ],
    };
    let (imported, _, text) = unpack_and_import(&list, "level_spawn_file/chunk_ids")?;

    assert_eq!(text.matches("chunk_id = ").count(), 2, "{text}");
    assert!(text.contains("\r\nchunk_id = 5\r\n"), "{text}");
    assert!(text.contains("\r\nchunk_id = 0\r\n"), "{text}");
    assert_eq!(imported, list);
    assert_eq!(packed(&imported)?, packed(&list)?);

    Ok(())
  }

  /// A verbatim record is the user's: its payload packs as written though it would now read typed, and its reason is
  /// not checked against anything.
  #[test]
  fn packs_a_verbatim_record_from_its_payload_though_it_reads_typed() -> XrfResult {
    let bolt: LevelSpawnObject = LevelSpawnObjects::bolt(0)?;
    let path: PathBuf = GeneratedResources::create_dir("level_spawn_file/verbatim_typed")?;

    fs::write(
      path.join(LevelSpawnFile::UNPACKED_FILE_NAME),
      format!(
        "[0]\r\nform = verbatim\r\nreason = edited by hand\r\nsection = bolt\r\nname = zat_bolt\r\nscript_game_id = 0\r\n\
         script_rp = 254\r\nposition = 12.5,-3,101.25\r\ndirection = 0,1.5,0\r\npayload = {}\r\n",
        encode_bytes_to_base64(&bolt.payload)
      ),
    )?;

    assert_eq!(
      LevelSpawnFile::import_from_path::<XRayByteOrder, _>(&path)?,
      LevelSpawnFile { objects: vec![bolt] }
    );

    Ok(())
  }

  /// A section is typed or verbatim by its `form` key alone, so a form neither names is refused rather than guessed.
  #[test]
  fn refuses_a_record_of_another_form() -> XrfResult {
    let path: PathBuf = GeneratedResources::create_dir("level_spawn_file/other_form")?;

    fs::write(
      path.join(LevelSpawnFile::UNPACKED_FILE_NAME),
      "[0]\r\nform = sketch\r\n",
    )?;

    let error: String = LevelSpawnFile::import_from_path::<XRayByteOrder, _>(&path)
      .expect_err("an unknown form")
      .to_string();

    assert!(error.contains("'0' is of form 'sketch'"), "{error}");

    Ok(())
  }
}

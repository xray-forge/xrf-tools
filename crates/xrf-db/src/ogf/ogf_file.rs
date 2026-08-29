use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{
  ChunkDataSource, ChunkReader, find_one_of_optional_chunk_by_id, find_one_of_required_chunks_by_id,
  find_optional_chunk_by_id, find_required_chunk_by_id,
};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::data::ogf::ogf_geometry::OgfGeometry;
use crate::data::ogf::ogf_motion::OgfMotion;
use crate::data::ogf::ogf_motion_definition::OgfMotionDefinition;
use crate::ogf::chunks::ogf_bones_chunk::OgfBonesChunk;
use crate::ogf::chunks::ogf_children_chunk::OgfChildrenChunk;
use crate::ogf::chunks::ogf_description_chunk::OgfDescriptionChunk;
use crate::ogf::chunks::ogf_header_chunk::OgfHeaderChunk;
use crate::ogf::chunks::ogf_ik_data_chunk::OgfIkDataChunk;
use crate::ogf::chunks::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::ogf::chunks::ogf_lods_chunk::OgfLodsChunk;
use crate::ogf::chunks::ogf_swi_data_chunk::OgfSwiDataChunk;
use crate::ogf::chunks::ogf_texture_chunk::OgfTextureChunk;
use crate::ogf::chunks::ogf_user_data_chunk::OgfUserDataChunk;
use crate::ogf::ogf_residue::OgfResidue;
use crate::omf::chunks::omf_motions_chunk::OmfMotionsChunk;
use crate::omf::chunks::omf_parameters_chunk::OmfParametersChunk;

/// FMesh in c++ codebase.
///
/// Reads only. OGF payloads cannot be fully re-serialized, so the parsed form is a view for
/// inspection and verification rather than a document: geometry keeps only its bone indices and
/// unknown chunks are skipped entirely. Editing an ogf file therefore never goes through this type,
/// see [`crate::OgfMotionRefsProcessor`] and [`crate::OgfTextureRefsProcessor`], which patch raw
/// chunks and copy everything they do not change byte for byte.
#[derive(Debug, Serialize, Deserialize)]
pub struct OgfFile {
  pub header: OgfHeaderChunk,
  pub texture: Option<OgfTextureChunk>,
  pub geometry: Option<OgfGeometry>,
  pub bones: Option<OgfBonesChunk>,
  /// Progressive mesh level of detail table, present only on progressive visuals.
  pub swi_data: Option<OgfSwiDataChunk>,
  pub children: Option<OgfChildrenChunk>,
  pub description: Option<OgfDescriptionChunk>,
  pub kinematics: Option<OgfKinematicsChunk>,
  pub ik_data: Option<OgfIkDataChunk>,
  pub user_data: Option<OgfUserDataChunk>,
  pub lods: Option<OgfLodsChunk>,
  /// Motions stored inside the visual itself rather than referenced from an omf file.
  ///
  /// Self-animated models embed the same two chunks an omf carries, under the same ids, so the omf
  /// types are reused verbatim.
  pub motions: Option<OmfMotionsChunk>,
  pub motion_parameters: Option<OmfParametersChunk>,
  /// Bytes past the last root chunk that the engine's loader never reads.
  ///
  /// Present only for a visual that is not a well-formed chunk stream and is loaded anyway. Nothing forces a consumer
  /// to look, so a reader answering "did this parse?" gets the same answer it always did; a consumer answering "is this
  /// well-formed?" asks whether this is `None`, and learns what is wrong rather than only that something is.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub residue: Option<OgfResidue>,
}

impl OgfFile {
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "OGF file was not read: {}, error: {}",
        format_path(path.as_ref()),
        error
      ))
    })?)
  }

  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads a visual already held in memory, such as one served out of an archive volume.
  ///
  /// An archived entry has no file to slice, and a caller holding its bytes should not have to learn about chunk
  /// readers to parse them.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let (chunks, residue) = OgfResidue::read_root_chunks::<T, _>(reader)?;

    Ok(Self {
      residue,
      ..Self::read_from_chunks::<T, _>(&chunks)?
    })
  }

  pub fn read_from_chunks<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> XrfResult<Self> {
    // Bones are read up front because the ik data chunk stores no count of its own and has one record
    // per bone, so it can only be read once the bone list is known.
    let bones: Option<OgfBonesChunk> = match find_optional_chunk_by_id(chunks, OgfBonesChunk::CHUNK_ID) {
      Some(mut it) => Some(it.read_xr::<T, _>()?),
      None => None,
    };

    let ik_data: Option<OgfIkDataChunk> = match (&bones, find_optional_chunk_by_id(chunks, OgfIkDataChunk::CHUNK_ID)) {
      (Some(bones), Some(mut it)) => Some(OgfIkDataChunk::read::<T, _>(&mut it, bones.bones.len())?),
      _ => None,
    };

    // Read ahead of the record so the two embedded motion chunks can be checked against each other.
    let motions: Option<OmfMotionsChunk> = match find_optional_chunk_by_id(chunks, OmfMotionsChunk::CHUNK_ID) {
      Some(mut it) => Some(it.read_xr::<T, _>()?),
      None => None,
    };

    let motion_parameters: Option<OmfParametersChunk> =
      match find_optional_chunk_by_id(chunks, OmfParametersChunk::CHUNK_ID) {
        Some(mut it) => Some(it.read_xr::<T, _>()?),
        None => None,
      };

    Self::assert_motions_are_paired(&motions, &motion_parameters)?;

    Ok(Self {
      bones,
      ik_data,
      header: find_required_chunk_by_id(chunks, OgfHeaderChunk::CHUNK_ID)?.read_xr::<T, _>()?,
      texture: match find_optional_chunk_by_id(chunks, OgfTextureChunk::CHUNK_ID) {
        Some(mut it) => Some(it.read_xr::<T, _>()?),
        None => None,
      },
      geometry: OgfGeometry::read_from_chunks::<T, _>(chunks)?,
      swi_data: match find_optional_chunk_by_id(chunks, OgfSwiDataChunk::CHUNK_ID) {
        Some(mut it) => Some(it.read_xr::<T, _>()?),
        None => None,
      },
      children: match find_optional_chunk_by_id(chunks, OgfChildrenChunk::CHUNK_ID) {
        Some(mut it) => Some(it.read_xr::<T, _>()?),
        None => None,
      },
      description: match find_optional_chunk_by_id(chunks, OgfDescriptionChunk::CHUNK_ID) {
        Some(mut it) => Some(it.read_xr::<T, _>()?),
        None => None,
      },
      kinematics: match find_one_of_optional_chunk_by_id(
        chunks,
        &[OgfKinematicsChunk::CHUNK_ID, OgfKinematicsChunk::CHUNK_ID_OLD],
      ) {
        Some((id, mut it)) => Some(OgfKinematicsChunk::read::<T, _>(&mut it, id)?),
        None => None,
      },
      user_data: match find_optional_chunk_by_id(chunks, OgfUserDataChunk::CHUNK_ID) {
        Some(mut it) => Some(it.read_xr::<T, _>()?),
        None => None,
      },
      lods: match find_optional_chunk_by_id(chunks, OgfLodsChunk::CHUNK_ID) {
        Some(mut it) => Some(it.read_xr::<T, _>()?),
        None => None,
      },
      motions,
      motion_parameters,
      // Root residue is a property of the byte stream, not of a chunk list, so a caller handed chunks directly cannot
      // be told about it. `read_from_chunk` fills this in.
      residue: None,
    })
  }

  /// Read only list of motion refs specifically and skip other data parts.
  pub fn read_motion_refs_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Vec<String>> {
    Self::read_motion_refs_from_file::<T>(File::open(path)?)
  }

  /// Read only list of motion refs specifically and skip other data parts.
  pub fn read_motion_refs_from_file<T: ByteOrder>(file: File) -> XrfResult<Vec<String>> {
    Self::read_motion_refs_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Lists motion references from a chunk reader over any data source.
  ///
  /// The route an archived visual takes: a volume entry has no file, so only its decompressed bytes are available.
  pub fn read_motion_refs_from_chunk<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<Vec<String>> {
    let (chunks, _) = OgfResidue::read_root_chunks::<T, _>(reader)?;

    log::info!(
      "Reading ogf file motion refs, {} chunks, {} bytes",
      chunks.len(),
      reader.read_bytes_len(),
    );

    let (chunk_id, mut chunk) = find_one_of_required_chunks_by_id(
      &chunks,
      &[OgfKinematicsChunk::CHUNK_ID, OgfKinematicsChunk::CHUNK_ID_OLD],
    )?;

    Ok(OgfKinematicsChunk::read::<T, _>(&mut chunk, chunk_id)?.motion_refs)
  }

  /// Collect the texture reference of every nested child visual.
  ///
  /// Texture chunks of a skeleton live inside the children container rather than at the top level,
  /// so the top level `texture` field is empty for the models that have any.
  pub fn read_texture_refs_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Vec<String>> {
    Ok(
      Self::read_from_path::<T, _>(path)?
        .children
        .map(|children| {
          children
            .nested
            .iter()
            .filter_map(|it| it.texture.as_ref().map(|it| it.texture_name.clone()))
            .collect()
        })
        .unwrap_or_default(),
    )
  }

  /// Rejects embedded motion chunks that cannot be paired by ordinal.
  ///
  /// The engine loads a self-animated visual through the same `motions_value::load` an omf takes, which fatals when
  /// the parameters chunk is missing and indexes payloads by the definition ordinal
  /// (`xray-16/src/xrCore/Animation/SkeletonMotions.cpp`). Either chunk alone, or two lists of different lengths,
  /// therefore describes a visual the engine cannot animate, and would silently drop motions from every ordinal pair
  /// this file hands out. `OmfFile` guards its own copies of the same two chunks the same way.
  fn assert_motions_are_paired(
    motions: &Option<OmfMotionsChunk>,
    motion_parameters: &Option<OmfParametersChunk>,
  ) -> XrfResult {
    match (motions, motion_parameters) {
      (Some(motions), Some(parameters)) if motions.motions.len() != parameters.motions.len() => {
        Err(XrfError::new_parsing_error(format!(
          "Unexpected data stored in OGF file, count of embedded motions and motions definitions mismatch: {} got, {} expected",
          parameters.motions.len(),
          motions.motions.len()
        )))
      }
      (Some(_), None) => Err(XrfError::new_parsing_error(
        "Unexpected data stored in OGF file, embedded motions are stored without motion definitions",
      )),
      (None, Some(_)) => Err(XrfError::new_parsing_error(
        "Unexpected data stored in OGF file, motion definitions are stored without embedded motions",
      )),
      _ => Ok(()),
    }
  }
}

impl OgfFile {
  /// Motions the visual embeds, each definition with the key payload at its own ordinal.
  ///
  /// Empty for a visual that animates only from referenced omf files. A self-animated one carries both chunks, which
  /// [`OgfFile::read_from_chunks`] requires and checks for equal length, so every definition here has a payload.
  pub fn get_motions(&self) -> impl Iterator<Item = (&OgfMotionDefinition, &OgfMotion)> {
    self
      .motion_parameters
      .as_ref()
      .zip(self.motions.as_ref())
      .into_iter()
      .flat_map(|(parameters, motions)| parameters.motions.iter().zip(motions.motions.iter()))
  }

  /// Names of the motions the visual embeds, as used for engine lookups.
  pub fn get_motion_names(&self) -> Vec<&str> {
    self
      .get_motions()
      .map(|(definition, _)| definition.name.as_str())
      .collect()
  }

  /// The embedded motion a name resolves to, with the payload at its ordinal.
  pub fn get_motion_by_name(&self, name: &str) -> Option<(&OgfMotionDefinition, &OgfMotion)> {
    self.get_motions().find(|(definition, _)| definition.name == name)
  }

  /// Count of embedded payloads whose preserved label no longer matches the name of the motion it carries.
  pub fn get_diverging_labels_count(&self) -> usize {
    self
      .get_motions()
      .filter(|(definition, motion)| !motion.has_label_matching(&definition.name))
      .count()
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::data::generic::vector_3d::Vector3d;
  use crate::data::ogf::ogf_box::OgfBox;
  use crate::data::ogf::ogf_motion::OgfMotion;
  use crate::data::ogf::ogf_motion_definition::OgfMotionDefinition;
  use crate::data::ogf::ogf_part::OgfPart;
  use crate::data::ogf::ogf_sphere::OgfSphere;
  use crate::ogf::chunks::ogf_header_chunk::OgfHeaderChunk;
  use crate::ogf::ogf_file::OgfFile;
  use crate::omf::chunks::omf_motions_chunk::OmfMotionsChunk;
  use crate::omf::chunks::omf_parameters_chunk::OmfParametersChunk;

  fn new_chunk_bytes<C: ChunkReadWrite>(id: u32, value: &C) -> XrfResult<Vec<u8>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    writer.write_xr::<XRayByteOrder, _>(value)?;

    writer.flush_chunk_into_buffer::<XRayByteOrder>(id)
  }

  fn new_header_bytes() -> XrfResult<Vec<u8>> {
    new_chunk_bytes(
      OgfHeaderChunk::CHUNK_ID,
      &OgfHeaderChunk {
        version: 4,
        model_type: 3,
        shader_id: 0,
        bounding_box: OgfBox {
          min: Vector3d::new(0.0, 0.0, 0.0),
          max: Vector3d::new(1.0, 1.0, 1.0),
        },
        bounding_sphere: OgfSphere {
          position: Vector3d::new(0.0, 0.0, 0.0),
          radius: 1.0,
        },
      },
    )
  }

  fn new_motions_bytes(count: usize) -> XrfResult<Vec<u8>> {
    new_chunk_bytes(
      OmfMotionsChunk::CHUNK_ID,
      &OmfMotionsChunk {
        motions: (0..count)
          .map(|ordinal| OgfMotion {
            label: format!("embedded_{ordinal}"),
            count: 2,
            flags: 0,
            remaining: vec![1, 2, 3],
          })
          .collect(),
      },
    )
  }

  fn new_parameters_bytes(names: &[&str]) -> XrfResult<Vec<u8>> {
    new_chunk_bytes(
      OmfParametersChunk::CHUNK_ID,
      &OmfParametersChunk {
        version: 4,
        parts: vec![OgfPart {
          name: String::from("default"),
          bones: vec![(String::from("bip01"), 0)],
        }],
        motions: names
          .iter()
          .enumerate()
          .map(|(ordinal, name)| {
            let mut definition: OgfMotionDefinition = OgfMotionDefinition::new_mock(Vec::new());

            definition.name = String::from(*name);
            definition.motion = ordinal as u16;
            definition
          })
          .collect(),
      },
    )
  }

  #[test]
  fn test_read_rejects_embedded_motions_without_definitions() -> XrfResult {
    let mut bytes: Vec<u8> = new_header_bytes()?;

    bytes.extend(new_motions_bytes(1)?);

    assert!(
      OgfFile::read_from_bytes::<XRayByteOrder>(bytes).is_err(),
      "Expect a visual whose motions have no definitions to be rejected, as the engine cannot name them"
    );

    Ok(())
  }

  #[test]
  fn test_read_rejects_definitions_without_embedded_motions() -> XrfResult {
    let mut bytes: Vec<u8> = new_header_bytes()?;

    bytes.extend(new_parameters_bytes(&["embedded_idle"])?);

    assert!(
      OgfFile::read_from_bytes::<XRayByteOrder>(bytes).is_err(),
      "Expect definitions without payloads to be rejected rather than read as a nameless motion set"
    );

    Ok(())
  }

  #[test]
  fn test_read_rejects_mismatched_embedded_motion_counts() -> XrfResult {
    let mut bytes: Vec<u8> = new_header_bytes()?;

    bytes.extend(new_motions_bytes(2)?);
    bytes.extend(new_parameters_bytes(&["embedded_idle"])?);

    assert!(
      OgfFile::read_from_bytes::<XRayByteOrder>(bytes).is_err(),
      "Expect unequal counts to be rejected rather than silently truncating the ordinal pairs"
    );

    Ok(())
  }

  #[test]
  fn test_read_pairs_embedded_motions_by_ordinal() -> XrfResult {
    let mut bytes: Vec<u8> = new_header_bytes()?;

    bytes.extend(new_motions_bytes(2)?);
    bytes.extend(new_parameters_bytes(&["embedded_idle", "embedded_draw"])?);

    let file: OgfFile = OgfFile::read_from_bytes::<XRayByteOrder>(bytes)?;

    assert_eq!(file.get_motion_names(), vec!["embedded_idle", "embedded_draw"]);
    assert_eq!(
      file
        .get_motion_by_name("embedded_draw")
        .map(|(_, motion)| motion.label.as_str()),
      Some("embedded_1"),
      "Expect the payload at the definition's own ordinal, whatever its label says"
    );
    assert_eq!(
      file.get_diverging_labels_count(),
      2,
      "Expect both labels to be reported as diverging from the names they pair with"
    );

    Ok(())
  }

  #[test]
  fn test_read_accepts_a_visual_without_embedded_motions() -> XrfResult {
    let file: OgfFile = OgfFile::read_from_bytes::<XRayByteOrder>(new_header_bytes()?)?;

    assert!(file.get_motion_names().is_empty());
    assert_eq!(file.get_diverging_labels_count(), 0);

    Ok(())
  }
}

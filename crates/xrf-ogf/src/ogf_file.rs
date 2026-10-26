use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader};
use xrf_error::{XrfError, XrfResult};
use xrf_skeleton::{SkeletonMotion, SkeletonMotionDefinition, SkeletonMotionParametersChunk, SkeletonMotionsChunk};
use xrf_utils::format_path;

use crate::chunks::level::ogf_children_link_chunk::OgfChildrenLinkChunk;
use crate::chunks::level::ogf_fastpath_chunk::OgfFastpathChunk;
use crate::chunks::level::ogf_geometry_container_chunk::OgfGeometryContainerChunk;
use crate::chunks::level::ogf_lod_definition_chunk::OgfLodDefinitionChunk;
use crate::chunks::level::ogf_swi_container_chunk::OgfSwiContainerChunk;
use crate::chunks::level::ogf_tree_definition_chunk::OgfTreeDefinitionChunk;
use crate::chunks::mesh::ogf_children_chunk::OgfChildrenChunk;
use crate::chunks::mesh::ogf_collision_indices_chunk::OgfCollisionIndicesChunk;
use crate::chunks::mesh::ogf_collision_vertices_chunk::OgfCollisionVerticesChunk;
use crate::chunks::mesh::ogf_swi_data_chunk::OgfSwiDataChunk;
use crate::chunks::skeleton::ogf_bones_chunk::OgfBonesChunk;
use crate::chunks::skeleton::ogf_ik_data_chunk::OgfIkDataChunk;
use crate::chunks::skeleton::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::chunks::skeleton::ogf_lods_chunk::OgfLodsChunk;
use crate::chunks::skeleton::ogf_user_data_chunk::OgfUserDataChunk;
use crate::chunks::visual::ogf_description_chunk::OgfDescriptionChunk;
use crate::chunks::visual::ogf_header_chunk::OgfHeaderChunk;
use crate::chunks::visual::ogf_texture_chunk::OgfTextureChunk;
use crate::data::geometry::ogf_geometry::OgfGeometry;
use crate::data::ogf_model_type::OgfModelType;
use crate::residue::OgfResidue;

/// One visual, `FMesh` in the engine: every chunk the engine's visual kinds read, each an `Option`, and which of them a
/// visual carries follows from `header.model_type`.
///
/// The kinds read their chunks in layers (`xray-16/src/Layers/xrRender/`): every visual its header, texture and
/// description (`dxRender_Visual`); a static one its embedded geometry or the level buffers it draws from, and their
/// fast path (`Fvisual`); a progressive one its sliding windows (`FProgressive`); a hierarchy its children, nested or
/// linked (`FHierrarhyVisual`); a tree its placement and the level's window table it draws from (`FTreeVisual`); a level
/// of detail its impostor (`FLOD`); a skeleton its bones, physics, user data and LOD (`CKinematics`) and its motions,
/// referenced or embedded (`CKinematicsAnimated`). The SDK's exporter adds a collision mesh no loader reads.
#[derive(Debug)]
pub struct OgfFile {
  pub header: OgfHeaderChunk,
  pub texture: Option<OgfTextureChunk>,
  /// Who built the visual, or why the description did not read: nothing in the engine depends on it, so a malformed
  /// one costs the visual that part rather than the whole read.
  pub description: Option<XrfResult<OgfDescriptionChunk>>,
  /// Vertices and indices the visual carries itself.
  pub geometry: Option<OgfGeometry>,
  /// Where a level visual's geometry sits in `level.geom`, in place of its own.
  pub geometry_container: Option<OgfGeometryContainerChunk>,
  /// The same in `level.geomX`, for a level visual the compiler gave a fast path.
  pub fastpath: Option<OgfFastpathChunk>,
  /// Progressive mesh level of detail table, present only on progressive visuals.
  pub swi_data: Option<OgfSwiDataChunk>,
  /// The level's window table a progressive tree draws from, in place of its own.
  pub swi_container: Option<OgfSwiContainerChunk>,
  /// Child visuals carried whole, as a skeleton carries its meshes.
  pub children: Option<OgfChildrenChunk>,
  /// Child visuals named by index in the level's visuals run, as a level hierarchy names its parts.
  pub children_links: Option<OgfChildrenLinkChunk>,
  /// Where a tree stands and how it is lit.
  pub tree_definition: Option<OgfTreeDefinitionChunk>,
  /// The impostor a `MT_LOD` visual draws in place of the trees it stands for.
  pub lod_definition: Option<OgfLodDefinitionChunk>,
  pub bones: Option<OgfBonesChunk>,
  pub kinematics: Option<OgfKinematicsChunk>,
  pub ik_data: Option<OgfIkDataChunk>,
  pub user_data: Option<OgfUserDataChunk>,
  pub lods: Option<OgfLodsChunk>,
  /// Motions stored inside the visual itself rather than referenced from an omf file.
  pub motions: Option<SkeletonMotionsChunk>,
  pub motion_parameters: Option<SkeletonMotionParametersChunk>,
  pub collision_vertices: Option<OgfCollisionVerticesChunk>,
  pub collision_indices: Option<OgfCollisionIndicesChunk>,
  /// Where the root chunks depart from a well-formed tree in a way the engine's loader does not notice.
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
    let bones: Option<OgfBonesChunk> = match ChunkReader::find_optional_by_id(chunks, OgfBonesChunk::CHUNK_ID) {
      Some(mut it) => Some(it.read_xr::<T, _>()?),
      None => None,
    };

    let ik_data: Option<OgfIkDataChunk> = match (
      &bones,
      ChunkReader::find_optional_by_id(chunks, OgfIkDataChunk::CHUNK_ID),
    ) {
      (Some(bones), Some(mut it)) => Some(OgfIkDataChunk::read::<T, _>(&mut it, bones.bones.len())?),
      _ => None,
    };

    // Read ahead of the record so the two embedded motion chunks can be checked against each other.
    let motions: Option<SkeletonMotionsChunk> =
      match ChunkReader::find_optional_by_id(chunks, SkeletonMotionsChunk::CHUNK_ID) {
        Some(mut it) => Some(it.read_xr::<T, _>()?),
        None => None,
      };

    let motion_parameters: Option<SkeletonMotionParametersChunk> =
      match ChunkReader::find_optional_by_id(chunks, SkeletonMotionParametersChunk::CHUNK_ID) {
        Some(mut it) => Some(it.read_xr::<T, _>()?),
        None => None,
      };

    Self::assert_motions_are_paired(&motions, &motion_parameters)?;

    let header: OgfHeaderChunk =
      ChunkReader::find_required_by_id(chunks, OgfHeaderChunk::CHUNK_ID)?.read_xr::<T, _>()?;

    Ok(Self {
      bones,
      ik_data,
      texture: Self::read_optional::<T, _, _>(chunks, OgfTextureChunk::CHUNK_ID)?,
      description: ChunkReader::find_optional_by_id(chunks, OgfDescriptionChunk::CHUNK_ID)
        .map(|mut it| it.read_xr::<T, _>()),
      geometry: OgfGeometry::read_from_chunks::<T, _>(chunks, OgfModelType::from_raw(header.model_type))?,
      geometry_container: Self::read_optional::<T, _, _>(chunks, OgfGeometryContainerChunk::CHUNK_ID)?,
      fastpath: match ChunkReader::find_optional_by_id(chunks, OgfFastpathChunk::CHUNK_ID) {
        Some(mut it) => Some(OgfFastpathChunk::read_from_chunk::<T, _>(&mut it)?),
        None => None,
      },
      swi_data: Self::read_optional::<T, _, _>(chunks, OgfSwiDataChunk::CHUNK_ID)?,
      swi_container: Self::read_optional::<T, _, _>(chunks, OgfSwiContainerChunk::CHUNK_ID)?,
      children: match ChunkReader::find_optional_by_id(chunks, OgfChildrenChunk::CHUNK_ID) {
        Some(mut it) => Some(OgfChildrenChunk::read::<T, _>(&mut it)?),
        None => None,
      },
      children_links: Self::read_optional::<T, _, _>(chunks, OgfChildrenLinkChunk::CHUNK_ID)?,
      tree_definition: Self::read_optional::<T, _, _>(chunks, OgfTreeDefinitionChunk::CHUNK_ID)?,
      lod_definition: Self::read_optional::<T, _, _>(chunks, OgfLodDefinitionChunk::CHUNK_ID)?,
      collision_vertices: Self::read_optional::<T, _, _>(chunks, OgfCollisionVerticesChunk::CHUNK_ID)?,
      collision_indices: Self::read_optional::<T, _, _>(chunks, OgfCollisionIndicesChunk::CHUNK_ID)?,
      header,
      kinematics: match ChunkReader::find_one_of_optional_by_id(
        chunks,
        &[OgfKinematicsChunk::CHUNK_ID, OgfKinematicsChunk::CHUNK_ID_OLD],
      ) {
        Some((id, mut it)) => Some(OgfKinematicsChunk::read::<T, _>(&mut it, id)?),
        None => None,
      },
      user_data: Self::read_optional::<T, _, _>(chunks, OgfUserDataChunk::CHUNK_ID)?,
      lods: Self::read_optional::<T, _, _>(chunks, OgfLodsChunk::CHUNK_ID)?,
      motions,
      motion_parameters,
      // Root residue is a property of the byte stream, not of a chunk list, so a caller handed chunks directly cannot
      // be told about it. `read_from_chunk` fills this in.
      residue: None,
    })
  }

  /// The motion refs of the visual `bytes` hold, skipping its other data.
  pub fn read_motion_refs_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Vec<String>> {
    Self::read_motion_refs_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Lists motion references from a chunk reader over any data source.
  pub fn read_motion_refs_from_chunk<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<Vec<String>> {
    let (chunks, _) = OgfResidue::read_root_chunks::<T, _>(reader)?;

    log::info!(
      "Reading ogf file motion refs, {} chunks, {} bytes",
      chunks.len(),
      reader.read_bytes_len(),
    );

    let (chunk_id, mut chunk) = ChunkReader::find_one_of_required_by_id(
      &chunks,
      &[OgfKinematicsChunk::CHUNK_ID, OgfKinematicsChunk::CHUNK_ID_OLD],
    )?;

    Ok(OgfKinematicsChunk::read::<T, _>(&mut chunk, chunk_id)?.motion_refs)
  }

  /// The texture reference of every nested child visual the visual `bytes` hold.
  pub fn read_texture_refs_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Vec<String>> {
    Ok(Self::read_from_bytes::<T>(bytes)?.list_texture_refs())
  }

  /// Lists texture references from a chunk reader over any data source.
  pub fn read_texture_refs_from_chunk<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<Vec<String>> {
    Ok(Self::read_from_chunk::<T, _>(reader)?.list_texture_refs())
  }

  /// The chunk of `id` read as `C`, when the visual carries one.
  fn read_optional<T: ByteOrder, D: ChunkDataSource, C: ChunkReadWrite>(
    chunks: &[ChunkReader<D>],
    id: u32,
  ) -> XrfResult<Option<C>> {
    match ChunkReader::find_optional_by_id(chunks, id) {
      Some(mut chunk) => Ok(Some(chunk.read_xr::<T, C>()?)),
      None => Ok(None),
    }
  }

  fn list_texture_refs(self) -> Vec<String> {
    self
      .children
      .map(|children| {
        children
          .nested
          .iter()
          .filter_map(|it| it.texture.as_ref().map(|it| it.texture_name.clone()))
          .collect()
      })
      .unwrap_or_default()
  }

  /// Rejects embedded motion chunks that cannot be paired by ordinal.
  fn assert_motions_are_paired(
    motions: &Option<SkeletonMotionsChunk>,
    motion_parameters: &Option<SkeletonMotionParametersChunk>,
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
  /// The description, when the visual carries one that reads.
  pub fn get_description(&self) -> Option<&OgfDescriptionChunk> {
    self
      .description
      .as_ref()
      .and_then(|description| description.as_ref().ok())
  }

  /// Motions the visual embeds, each definition with the key payload at its own ordinal.
  pub fn get_motions(&self) -> impl Iterator<Item = (&SkeletonMotionDefinition, &SkeletonMotion)> {
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
  pub fn get_motion_by_name(&self, name: &str) -> Option<(&SkeletonMotionDefinition, &SkeletonMotion)> {
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
  use xrf_math::Vector3d;
  use xrf_skeleton::{
    SkeletonMotion, SkeletonMotionDefinition, SkeletonMotionParametersChunk, SkeletonMotionsChunk, SkeletonPart,
  };

  use crate::chunks::visual::ogf_header_chunk::OgfHeaderChunk;
  use crate::data::shape::ogf_box::OgfBox;
  use crate::data::shape::ogf_sphere::OgfSphere;
  use crate::ogf_file::OgfFile;

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
      SkeletonMotionsChunk::CHUNK_ID,
      &SkeletonMotionsChunk {
        motions: (0..count)
          .map(|ordinal| SkeletonMotion {
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
      SkeletonMotionParametersChunk::CHUNK_ID,
      &SkeletonMotionParametersChunk {
        version: 4,
        parts: vec![SkeletonPart {
          name: String::from("default"),
          bones: vec![(String::from("bip01"), 0)],
        }],
        motions: names
          .iter()
          .enumerate()
          .map(|(ordinal, name)| {
            let mut definition: SkeletonMotionDefinition = SkeletonMotionDefinition::new_mock(Vec::new());

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

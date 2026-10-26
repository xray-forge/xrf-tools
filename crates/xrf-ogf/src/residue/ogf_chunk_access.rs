use xrf_skeleton::{SkeletonMotionParametersChunk, SkeletonMotionsChunk};

use crate::chunks::ogf_bones_chunk::OgfBonesChunk;
use crate::chunks::ogf_children_chunk::OgfChildrenChunk;
use crate::chunks::ogf_children_link_chunk::OgfChildrenLinkChunk;
use crate::chunks::ogf_description_chunk::OgfDescriptionChunk;
use crate::chunks::ogf_fastpath_chunk::OgfFastpathChunk;
use crate::chunks::ogf_geometry_container_chunk::OgfGeometryContainerChunk;
use crate::chunks::ogf_header_chunk::OgfHeaderChunk;
use crate::chunks::ogf_ik_data_chunk::OgfIkDataChunk;
use crate::chunks::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::chunks::ogf_lod_definition_chunk::OgfLodDefinitionChunk;
use crate::chunks::ogf_lods_chunk::OgfLodsChunk;
use crate::chunks::ogf_swi_container_chunk::OgfSwiContainerChunk;
use crate::chunks::ogf_swi_data_chunk::OgfSwiDataChunk;
use crate::chunks::ogf_texture_chunk::OgfTextureChunk;
use crate::chunks::ogf_tree_definition_chunk::OgfTreeDefinitionChunk;
use crate::chunks::ogf_user_data_chunk::OgfUserDataChunk;
use crate::data::ogf_geometry::OgfGeometry;

/// How the engine's visual loaders reach a root chunk, which is what its declared size and its compression mark mean.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OgfChunkAccess {
  /// `r_chunk_safe`: found, then asserted to be exactly the record's size. The header.
  Sized,
  /// `find_chunk`: the payload is read off the visual's reader, unbounded by the declared size, and the mark is ignored.
  Found,
  /// `open_chunk`: bounded to the declared size, and decompressed when marked.
  Opened,
}

impl OgfChunkAccess {
  /// The engine's access to a root chunk of `chunk_id`, mark masked off, or `None` for an id no loader reads.
  pub const fn of(chunk_id: u32) -> Option<Self> {
    match chunk_id {
      OgfHeaderChunk::CHUNK_ID => Some(Self::Sized),
      OgfTextureChunk::CHUNK_ID
      | OgfGeometry::VERTICES_CHUNK_ID
      | OgfGeometry::INDICES_CHUNK_ID
      | OgfChildrenLinkChunk::CHUNK_ID
      | OgfLodDefinitionChunk::CHUNK_ID
      | OgfTreeDefinitionChunk::CHUNK_ID
      | OgfBonesChunk::CHUNK_ID
      | OgfDescriptionChunk::CHUNK_ID
      | OgfKinematicsChunk::CHUNK_ID_OLD
      | OgfSwiContainerChunk::CHUNK_ID
      | OgfGeometryContainerChunk::CHUNK_ID
      | OgfKinematicsChunk::CHUNK_ID => Some(Self::Found),
      OgfSwiDataChunk::CHUNK_ID
      | OgfChildrenChunk::CHUNK_ID
      | OgfIkDataChunk::CHUNK_ID
      | OgfUserDataChunk::CHUNK_ID
      | OgfFastpathChunk::CHUNK_ID
      | OgfLodsChunk::CHUNK_ID
      | SkeletonMotionsChunk::CHUNK_ID
      | SkeletonMotionParametersChunk::CHUNK_ID => Some(Self::Opened),
      _ => None,
    }
  }
}

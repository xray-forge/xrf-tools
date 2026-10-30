use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

use byteorder::ByteOrder;
use xrf_chunk::ChunkDataSource;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelGeomSource, LevelShaderEntry, LevelShadersChunk, LevelVertexPayload, LevelVisualsChunk};
use xrf_math::Matrix4x4;
use xrf_utils::ContentHash;

use crate::pack::sector::gathering::sector_instance_key::SectorInstanceKey;
use crate::pack::tree::tree_part::TreePart;
use crate::pack::tree::tree_type::TreeType;

/// A level's trees by type: the compiler writes each part of a tree (a mesh and its shader) as a tree visual of its
/// own at the tree's transform, so the parts standing at one transform are one tree, and every tree of the same parts
/// is one type, wherever a sector stands it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TreeTypes {
  /// The types, the most stood first.
  pub types: Vec<TreeType>,
  /// Tree visuals the level holds, a tree's part each.
  pub parts: usize,
  /// Distinct transforms they stand at: the level's trees.
  pub placements: usize,
  /// Each tree visual's type, by the visual's index in the run.
  visual_types: BTreeMap<u32, u32>,
}

impl TreeTypes {
  /// What a sector names for the type of an impostor whose tree has none.
  pub const NO_TYPE: u32 = u32::MAX;

  /// Groups every tree visual of a level by where it stands, then the trees by their parts.
  pub fn of(visuals: &LevelVisualsChunk) -> Self {
    let mut trees: BTreeMap<[u32; Matrix4x4::ELEMENTS], (BTreeSet<SectorInstanceKey>, Vec<u32>)> = BTreeMap::new();
    let mut parts: BTreeMap<SectorInstanceKey, TreePart> = BTreeMap::new();
    let mut count: usize = 0;

    for (index, visual) in visuals.visuals.iter().enumerate() {
      let (Some(tree), Some(container)) = (&visual.tree, &visual.geometry) else {
        continue;
      };
      let key: SectorInstanceKey = SectorInstanceKey::of(container, visual.header.shader_id);
      let tree: &mut (BTreeSet<SectorInstanceKey>, Vec<u32>) =
        trees.entry(tree.transform.values.map(f32::to_bits)).or_default();

      count += 1;
      parts.entry(key).or_insert(TreePart {
        visual: index as u32,
        shader_id: visual.header.shader_id,
        triangles: container.get_primitive_count(),
      });
      tree.0.insert(key);
      tree.1.push(index as u32);
    }

    let mut kinds: BTreeMap<&BTreeSet<SectorInstanceKey>, u32> = BTreeMap::new();

    for (keys, _) in trees.values() {
      *kinds.entry(keys).or_default() += 1;
    }

    let mut types: Vec<(&BTreeSet<SectorInstanceKey>, TreeType)> = kinds
      .into_iter()
      .map(|(keys, instances)| {
        (
          keys,
          TreeType {
            parts: keys.iter().map(|key| parts[key]).collect(),
            instances,
          },
        )
      })
      .collect();

    types.sort_by_key(|(_, kind)| Reverse(kind.instances));

    let indices: BTreeMap<&BTreeSet<SectorInstanceKey>, u32> = types
      .iter()
      .enumerate()
      .map(|(index, (keys, _))| (*keys, index as u32))
      .collect();
    let visual_types: BTreeMap<u32, u32> = trees
      .values()
      .flat_map(|(keys, visuals)| visuals.iter().map(|visual| (*visual, indices[keys])))
      .collect();

    Self {
      placements: trees.len(),
      types: types.into_iter().map(|(_, kind)| kind).collect(),
      parts: count,
      visual_types,
    }
  }

  /// The type of the tree a tree visual is a part of, by its index in `types`; none for a visual that is no tree.
  pub fn get_type(&self, visual: u32) -> Option<u32> {
    self.visual_types.get(&visual).copied()
  }

  /// Each type's content key, in `types`' order: a hash of what draws its parts, their vertices and indices as the
  /// level stores them and the shader and textures dressing each, in an order of their own. The same tree model placed
  /// on two levels keys the same, whatever buffers either level stores it in.
  ///
  /// # Errors
  ///
  /// Returns an error when a part's vertices or indices cannot be read.
  pub fn read_keys<T: ByteOrder, D: ChunkDataSource>(
    &self,
    visuals: &LevelVisualsChunk,
    shaders: Option<&LevelShadersChunk>,
    source: &LevelGeomSource<D>,
  ) -> XrfResult<Vec<u64>> {
    let mut parts: BTreeMap<u32, u64> = BTreeMap::new();

    self
      .types
      .iter()
      .map(|kind| {
        let mut keys: Vec<u64> = kind
          .parts
          .iter()
          .map(|part| match parts.get(&part.visual) {
            Some(key) => Ok(*key),
            None => {
              let key: u64 = Self::read_part_key::<T, D>(visuals, shaders, source, part)?;

              parts.insert(part.visual, key);

              Ok(key)
            }
          })
          .collect::<XrfResult<_>>()?;

        keys.sort_unstable();

        Ok(
          keys
            .iter()
            .fold(&mut ContentHash::new(), |hash, key| hash.write(&key.to_le_bytes()))
            .finish(),
        )
      })
      .collect()
  }

  /// A part's content key: its vertices with their stride, its indices, and its shader table entry.
  fn read_part_key<T: ByteOrder, D: ChunkDataSource>(
    visuals: &LevelVisualsChunk,
    shaders: Option<&LevelShadersChunk>,
    source: &LevelGeomSource<D>,
    part: &TreePart,
  ) -> XrfResult<u64> {
    let container = visuals
      .visuals
      .get(part.visual as usize)
      .and_then(|visual| visual.geometry.as_ref())
      .ok_or_else(|| XrfError::new_not_found_error(format!("Tree visual {} has no geometry", part.visual)))?;
    let vertices: LevelVertexPayload = source.read_vertex_payload(
      container.vertex_buffer_id,
      container.vertex_base,
      container.vertex_count,
    )?;
    let indices: Vec<u16> =
      source.read_indices::<T>(container.index_buffer_id, container.index_base, container.index_count)?;
    let shader: String = shaders
      .and_then(|shaders| shaders.entries.get(usize::from(part.shader_id)))
      .map(LevelShaderEntry::to_raw)
      .unwrap_or_default();
    let mut hash: ContentHash = ContentHash::new();

    hash
      .write(&vertices.get_layout().get_stride().to_le_bytes())
      .write(vertices.get_bytes())
      .write(&(indices.len() as u64).to_le_bytes());

    for index in &indices {
      hash.write(&index.to_le_bytes());
    }

    Ok(hash.write_str(&shader).finish())
  }
}

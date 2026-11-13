use sha2::{Digest, Sha256};
use uuid::Uuid;
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelAiFile, LevelAiHeader, LevelAiNode, LevelAiNodes, LevelAiVersion};
use xrf_math::{Aabb, AiNodePosition, EPS_L};

use crate::ai::ai_grid::AiGrid;
use crate::ai::ai_map::AiMap;
use crate::ai::ai_map_compilation::AiMapCompilation;
use crate::ai::ai_map_compiler::AiMapCompiler;
use crate::ai::ai_map_guid::AiMapGuid;
use crate::ai::ai_vertex::AiVertex;

impl AiMapCompiler {
  /// The layout xrAI writes, `XRAI_CURRENT_VERSION` of Clear Sky and Call of Pripyat.
  const VERSION: LevelAiVersion = LevelAiVersion::CsCop;

  /// The light every node is saved with (`compiler_save.cpp:18`); the 2009 binary leaves whatever its stack held.
  const LIGHT: u8 = 15;

  /// A link of [`Self::VERSION`] at full width; xrAI's `InvalidNode` is stored cut to it.
  const LINK_MASK: u32 = (1 << Self::VERSION.get_link_bits()) - 1;

  /// Saves the nodes as xrAI does (`xrSaveNodes`, `xrAI/compiler_save.cpp:140`): the header's box around every node
  /// with its height grown by `EPS_L` (`CalculateHeight`), each node compressed over it, then the nodes stably sorted by
  /// packed cell with their links renumbered (`CNodeRenumberer`).
  pub(crate) fn save(cell_size: f32, vertices: &[AiVertex], replaced: Option<&AiMap>) -> XrfResult<AiMapCompilation> {
    let bounds: Aabb = Aabb::from_points(vertices.iter().map(|vertex| vertex.position))
      .ok_or_else(|| XrfError::new_invalid_error("Level AI-map holds no node to save"))?;
    let size_y: f32 = bounds.max.y - bounds.min.y + EPS_L;
    let grid: AiGrid = AiGrid::new(&bounds, cell_size);
    let count: u32 = u32::try_from(vertices.len())
      .map_err(|_| XrfError::new_invalid_error("Level AI-map holds more nodes than a header counts"))?;
    let compressed: Vec<LevelAiNode> = vertices
      .iter()
      .map(|vertex| LevelAiNode {
        links: vertex.links,
        high_cover: vertex.high_cover.map(Self::compress_cover),
        low_cover: vertex.low_cover.map(Self::compress_cover),
        plane: vertex.normal.to_compressed_normal(),
        packed_xz: grid.pack(vertex.position.x, vertex.position.z),
        packed_y: AiNodePosition::pack_height(vertex.position.y, bounds.min.y, size_y),
        light: Self::LIGHT,
      })
      .collect();
    let mut sorted: Vec<usize> = (0..compressed.len()).collect();

    sorted.sort_by_key(|index| compressed[*index].packed_xz);

    let mut renumbering: Vec<u32> = vec![0; sorted.len()];

    for (position, index) in sorted.iter().enumerate() {
      renumbering[*index] = position as u32;
    }

    // Each link is cut to the layout's width as it is compressed, then renumbered when it names a node.
    let nodes: Vec<LevelAiNode> = sorted
      .iter()
      .map(|index| {
        let node: LevelAiNode = compressed[*index];

        LevelAiNode {
          links: node.links.map(|link| {
            let link: u32 = link & Self::LINK_MASK;

            if link < count { renumbering[link as usize] } else { link }
          }),
          ..node
        }
      })
      .collect();
    let mut header: LevelAiHeader = LevelAiHeader {
      version: Self::VERSION as u32,
      count,
      size: cell_size,
      size_y,
      bounds,
      guid: Uuid::nil(),
    };
    let packed: LevelAiNodes = LevelAiNodes::from_nodes::<XRayByteOrder>(Self::VERSION, &nodes)?;
    let guid: AiMapGuid = match replaced {
      Some(replaced) if Self::is_same_graph(&header, &nodes, replaced) => {
        header.guid = replaced.file.header.guid;

        AiMapGuid::Kept
      }
      _ => {
        header.guid = Self::derive_guid(&header, &packed)?;

        AiMapGuid::Derived
      }
    };

    Ok(AiMapCompilation {
      map: AiMap {
        file: LevelAiFile { header },
        nodes: packed,
      },
      guid,
      gpu: None,
    })
  }

  /// `compress(c, 15)`: `iFloor(c * 15 + 0.5)` clamped to 0..15.
  fn compress_cover(cover: f32) -> u8 {
    (cover * 15.0 + 0.5).floor().clamp(0.0, 15.0) as u8
  }

  /// Whether `nodes` under `header` are the replaced map's graph: as many nodes, of the same cell size, each at the
  /// same cell and linked to the same nodes. What a spawn built against the replaced map indexes.
  fn is_same_graph(header: &LevelAiHeader, nodes: &[LevelAiNode], replaced: &AiMap) -> bool {
    let theirs: &LevelAiHeader = &replaced.file.header;

    if theirs.count != header.count || theirs.size != header.size {
      return false;
    }

    let cell = |header: &LevelAiHeader, node: &LevelAiNode| {
      let position = header.get_node_position(node);

      (
        (position.x / header.size).round() as i32,
        (position.z / header.size).round() as i32,
      )
    };
    let links =
      |header: &LevelAiHeader, node: &LevelAiNode| node.links.map(|link| (link < header.count).then_some(link));

    nodes
      .iter()
      .zip(replaced.nodes.iter_nodes::<XRayByteOrder>())
      .all(|(ours, their)| cell(header, ours) == cell(theirs, &their) && links(header, ours) == links(theirs, &their))
  }

  /// An identity hashed from the map's bytes, `header`'s own identity nil, so the same nodes always take the same one:
  /// the first 16 bytes of their SHA-256.
  fn derive_guid(header: &LevelAiHeader, nodes: &LevelAiNodes) -> XrfResult<Uuid> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    LevelAiFile {
      header: LevelAiHeader {
        guid: Uuid::nil(),
        ..header.clone()
      },
    }
    .write::<XRayByteOrder>(nodes, &mut writer)?;

    let digest = Sha256::digest(&writer.buffer);
    let mut bytes: [u8; 16] = [0; 16];

    bytes.copy_from_slice(&digest[..16]);

    Ok(Uuid::from_bytes(bytes))
  }
}

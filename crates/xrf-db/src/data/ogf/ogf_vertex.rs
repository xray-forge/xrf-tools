use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};

use crate::data::generic::vector_3d::Vector3d;

/// One bone a vertex is skinned to, and how strongly.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfVertexLink {
  pub bone: u16,
  pub weight: f32,
}

/// One vertex of an OGF visual, with its skinning links resolved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfVertex {
  pub position: Vector3d,
  pub normal: Vector3d,
  pub tangent: Vector3d,
  pub binormal: Vector3d,
  pub texture_u: f32,
  pub texture_v: f32,
  pub links: Vec<OgfVertexLink>,
}

impl OgfVertex {
  /// Bytes of geometry every format shares: four vectors and a uv pair.
  pub const GEOMETRY_SIZE: usize = 3 * 4 * 4 + 2 * 4;

  /// Bytes of the four vectors alone, which every format stores contiguously.
  const VECTORS_SIZE: usize = 4 * 3 * 4;
  const UV_SIZE: usize = 2 * 4;

  /// Read one vertex out of an already sized slice.
  ///
  /// Layouts follow `vertBoned1W` through `vertBoned4W` in `xrCore/Animation/Bone.hpp`, which are
  /// `#pragma pack(2)` and therefore have no padding to account for:
  ///
  /// - One link: `P N T B`, the uv pair, then the bone as a `u32`.
  /// - Two to four links: the bones as `u16`, `P N T B`, one fewer weight than there are bones, then
  ///   the uv pair. The weights sit between the binormal and the uv pair rather than before the
  ///   vectors, which is the whole reason the multi-link offsets are not simply the one-link ones
  ///   shifted along.
  ///
  /// The final weight is not stored, because the set sums to one, so it is reconstructed here and every
  /// returned vertex carries a weight for each of its links.
  ///
  /// A two link vertex stores the weight of its _second_ bone, not its first.
  pub fn read_from_slice<T: ByteOrder>(vertex: &[u8], links_count: usize) -> Self {
    debug_assert!(links_count >= 1, "a vertex is linked to at least one bone");

    if links_count == 1 {
      return Self {
        links: vec![OgfVertexLink {
          bone: T::read_u32(&vertex[Self::GEOMETRY_SIZE..Self::GEOMETRY_SIZE + 4]) as u16,
          weight: 1.0,
        }],
        ..Self::read_geometry::<T>(&vertex[..Self::VECTORS_SIZE], &vertex[Self::VECTORS_SIZE..], Vec::new())
      };
    }

    let bones_size: usize = links_count * 2;
    let weights_offset: usize = bones_size + Self::VECTORS_SIZE;
    let uv_offset: usize = weights_offset + (links_count - 1) * 4;

    let stored: Vec<f32> = (0..links_count - 1)
      .map(|index| {
        let offset: usize = weights_offset + index * 4;

        T::read_f32(&vertex[offset..offset + 4])
      })
      .collect();

    // Two links are the engine's lerp, whose one stored weight belongs to the second bone, leaving the remainder to the
    // first. Three and four store a weight per bone but the last, which takes whatever the stored ones did not claim.
    let weights: Vec<f32> = match links_count {
      2 => vec![1.0 - stored[0], stored[0]],
      _ => stored
        .iter()
        .copied()
        .chain([1.0 - stored.iter().sum::<f32>()])
        .collect(),
    };

    let links: Vec<OgfVertexLink> = weights
      .into_iter()
      .enumerate()
      .map(|(index, weight)| OgfVertexLink {
        bone: T::read_u16(&vertex[index * 2..index * 2 + 2]),
        weight,
      })
      .collect();

    Self::read_geometry::<T>(
      &vertex[bones_size..bones_size + Self::VECTORS_SIZE],
      &vertex[uv_offset..uv_offset + Self::UV_SIZE],
      links,
    )
  }

  fn read_geometry<T: ByteOrder>(vectors: &[u8], uv: &[u8], links: Vec<OgfVertexLink>) -> Self {
    let vector = |offset: usize| Vector3d {
      x: T::read_f32(&vectors[offset..offset + 4]),
      y: T::read_f32(&vectors[offset + 4..offset + 8]),
      z: T::read_f32(&vectors[offset + 8..offset + 12]),
    };

    Self {
      position: vector(0),
      normal: vector(12),
      tangent: vector(24),
      binormal: vector(36),
      texture_u: T::read_f32(&uv[0..4]),
      texture_v: T::read_f32(&uv[4..8]),
      links,
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::XRayByteOrder;

  use super::OgfVertex;

  /// The four vectors, which every format stores contiguously: position, normal, tangent, binormal.
  fn vector_bytes() -> Vec<u8> {
    let floats: [f32; 12] = [
      1.0, 2.0, 3.0, // position
      0.0, 1.0, 0.0, // normal
      1.0, 0.0, 0.0, // tangent
      0.0, 0.0, 1.0, // binormal
    ];

    floats.iter().flat_map(|it| it.to_le_bytes()).collect()
  }

  fn uv_bytes() -> Vec<u8> {
    [0.25f32, 0.75].iter().flat_map(|it| it.to_le_bytes()).collect()
  }

  /// One link stores the uv pair directly after the vectors; see `vertBoned1W`.
  fn geometry_bytes() -> Vec<u8> {
    let mut bytes: Vec<u8> = vector_bytes();

    bytes.extend(uv_bytes());

    bytes
  }

  /// A multi-link vertex, laid out as `vertBoned2W` through `vertBoned4W` do: bones, the four vectors,
  /// one fewer weight than bones, then the uv pair.
  fn linked_vertex_bytes(bones: &[u16], weights: &[f32]) -> Vec<u8> {
    assert_eq!(weights.len() + 1, bones.len(), "one weight is implied by the others");

    let mut bytes: Vec<u8> = Vec::new();

    for bone in bones {
      bytes.extend_from_slice(&bone.to_le_bytes());
    }

    bytes.extend(vector_bytes());

    for weight in weights {
      bytes.extend_from_slice(&weight.to_le_bytes());
    }

    bytes.extend(uv_bytes());

    bytes
  }

  fn assert_geometry(vertex: &OgfVertex) {
    assert_eq!(
      (vertex.position.x, vertex.position.y, vertex.position.z),
      (1.0, 2.0, 3.0)
    );
    assert_eq!((vertex.normal.x, vertex.normal.y, vertex.normal.z), (0.0, 1.0, 0.0));
    assert_eq!((vertex.tangent.x, vertex.tangent.y, vertex.tangent.z), (1.0, 0.0, 0.0));
    assert_eq!(
      (vertex.binormal.x, vertex.binormal.y, vertex.binormal.z),
      (0.0, 0.0, 1.0)
    );
    assert_eq!((vertex.texture_u, vertex.texture_v), (0.25, 0.75));
  }

  #[test]
  fn reads_a_single_link_vertex() {
    // One link stores its bone as a u32 after the geometry, and carries no stored weight.
    let mut bytes: Vec<u8> = geometry_bytes();
    bytes.extend_from_slice(&7u32.to_le_bytes());

    let vertex: OgfVertex = OgfVertex::read_from_slice::<XRayByteOrder>(&bytes, 1);

    assert_geometry(&vertex);
    assert_eq!(vertex.links.len(), 1);
    assert_eq!(vertex.links[0].bone, 7);
    assert_eq!(
      vertex.links[0].weight, 1.0,
      "Expect a lone link to own the whole weight"
    );
  }

  #[test]
  fn gives_a_two_link_vertexs_stored_weight_to_its_second_bone() {
    // `vertBoned2W` is blended as `lerp(P0, P1, w)`, so the stored weight is the second bone's and the first takes the
    // remainder - the opposite of the three and four link layouts. Reading it the uniform way still sums to one, which
    // is why this asserts which bone got which weight rather than the total.
    let bytes: Vec<u8> = linked_vertex_bytes(&[3, 9], &[0.25]);

    let vertex: OgfVertex = OgfVertex::read_from_slice::<XRayByteOrder>(&bytes, 2);

    assert_geometry(&vertex);
    assert_eq!(vertex.links.len(), 2);
    assert_eq!((vertex.links[0].bone, vertex.links[0].weight), (3, 0.75));
    assert_eq!((vertex.links[1].bone, vertex.links[1].weight), (9, 0.25));
  }

  #[test]
  fn reads_a_four_link_vertex_with_weights_summing_to_one() {
    let bytes: Vec<u8> = linked_vertex_bytes(&[1, 2, 3, 4], &[0.1, 0.2, 0.3]);

    let vertex: OgfVertex = OgfVertex::read_from_slice::<XRayByteOrder>(&bytes, 4);

    assert_geometry(&vertex);
    assert_eq!(
      vertex.links.iter().map(|it| it.bone).collect::<Vec<u16>>(),
      vec![1, 2, 3, 4]
    );
    assert!(
      (vertex.links.iter().map(|it| it.weight).sum::<f32>() - 1.0).abs() < 1e-6,
      "Expect weights to sum to one, got {:?}",
      vertex.links.iter().map(|it| it.weight).collect::<Vec<f32>>()
    );
    assert!((vertex.links[3].weight - 0.4).abs() < 1e-6);
  }

  #[test]
  fn reads_a_three_link_vertex() {
    let bytes: Vec<u8> = linked_vertex_bytes(&[5, 6, 7], &[0.5, 0.25]);

    let vertex: OgfVertex = OgfVertex::read_from_slice::<XRayByteOrder>(&bytes, 3);

    assert_geometry(&vertex);
    assert_eq!(
      vertex.links.iter().map(|it| it.bone).collect::<Vec<u16>>(),
      vec![5, 6, 7]
    );
    assert!((vertex.links[2].weight - 0.25).abs() < 1e-6);
  }

  #[test]
  fn matches_the_engine_vertex_sizes() {
    // Sizes of `vertBoned1W` through `vertBoned4W` in `xrCore/Animation/Bone.hpp`, which are
    // `#pragma pack(2)` and so have no padding. A layout whose fields are ordered differently but whose
    // total is right reads plausible garbage, so the sizes alone are not enough - see the vector
    // assertions above - but a wrong total desynchronises every following vertex.
    assert_eq!(geometry_bytes().len() + 4, 60, "vertBoned1W");
    assert_eq!(linked_vertex_bytes(&[0, 0], &[0.0]).len(), 64, "vertBoned2W");
    assert_eq!(linked_vertex_bytes(&[0; 3], &[0.0; 2]).len(), 70, "vertBoned3W");
    assert_eq!(linked_vertex_bytes(&[0; 4], &[0.0; 3]).len(), 76, "vertBoned4W");
  }

  #[test]
  fn reads_the_position_rather_than_a_later_vector_for_every_link_count() {
    // The regression this guards: multi-link layouts place their weights after the binormal, not before
    // the vectors. Reading them the other way put `position` where the engine keeps `normal`, so every
    // multi-link model measured as a unit cube and rendered as noise.
    for (links, bytes) in [
      (1usize, {
        let mut bytes: Vec<u8> = geometry_bytes();
        bytes.extend_from_slice(&7u32.to_le_bytes());
        bytes
      }),
      (2, linked_vertex_bytes(&[0, 0], &[0.5])),
      (3, linked_vertex_bytes(&[0; 3], &[0.5, 0.25])),
      (4, linked_vertex_bytes(&[0; 4], &[0.4, 0.3, 0.2])),
    ] {
      let vertex: OgfVertex = OgfVertex::read_from_slice::<XRayByteOrder>(&bytes, links);

      assert_eq!(
        (vertex.position.x, vertex.position.y, vertex.position.z),
        (1.0, 2.0, 3.0),
        "Expect {links} link position to be read from the first vector"
      );
      assert_eq!(
        (vertex.texture_u, vertex.texture_v),
        (0.25, 0.75),
        "Expect {links} link uv to be read from the trailing pair"
      );
    }
  }
}

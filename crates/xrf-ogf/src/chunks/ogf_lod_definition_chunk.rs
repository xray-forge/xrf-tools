use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::data::ogf_box::OgfBox;
use crate::data::ogf_lod_facet::OgfLodFacet;

/// The impostor of a `MT_LOD` visual, `OGF_LODDEF2`: eight facets looking at the trees it stands for from eight
/// sides, in the level's own space.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfLodDefinitionChunk {
  pub facets: [OgfLodFacet; 8],
}

impl OgfLodDefinitionChunk {
  /// `OGF_LODDEF2` (`xray-16/src/xrCore/FMesh.hpp`).
  pub const CHUNK_ID: u32 = 11;

  /// Facets an impostor has.
  pub const FACETS: usize = 8;

  /// A facet's normal as `FLOD::Load` derives it: the four corners' triangle normals averaged, normalized, and
  /// turned to face back along the direction a camera looks at it from.
  pub fn get_facet_normal(&self, facet: usize) -> Vector3d {
    let corners: Vec<&Vector3d> = self.facets[facet]
      .vertices
      .iter()
      .map(|vertex| &vertex.position)
      .collect();
    let mut sum: Vector3d = Vector3d::new(0.0, 0.0, 0.0);

    for index in 0..4 {
      let normal: Vector3d = Self::make_normal(corners[index], corners[(index + 1) % 4], corners[(index + 2) % 4]);

      sum = Vector3d::new(sum.x + normal.x, sum.y + normal.y, sum.z + normal.z);
    }

    let average: Vector3d = Self::normalize(&Vector3d::new(sum.x / 4.0, sum.y / 4.0, sum.z / 4.0));

    Vector3d::new(-average.x, -average.y, -average.z)
  }

  /// `FLOD::Load`'s correction to the visual's screen area: how much of its bounding sphere's disc a band of its
  /// middle extent covers, which is how much of a tree's sphere the impostor's facets fill.
  pub fn get_lod_factor(bounding_box: &OgfBox, radius: f32) -> f32 {
    let mut extents: [f32; 3] = [
      (bounding_box.max.x - bounding_box.min.x) / 2.0,
      (bounding_box.max.y - bounding_box.min.y) / 2.0,
      (bounding_box.max.z - bounding_box.min.z) / 2.0,
    ];

    extents.sort_by(f32::total_cmp);

    let middle: f32 = extents[1].min(radius);
    let band: f32 =
      4.0 * (0.5 * (radius * radius * (middle / radius).asin() + middle * (radius * radius - middle * middle).sqrt()));

    band / (std::f32::consts::PI * radius * radius)
  }

  /// `Fvector::mknormal`: the normal of the triangle the three points make, in their winding.
  fn make_normal(first: &Vector3d, second: &Vector3d, third: &Vector3d) -> Vector3d {
    let along: Vector3d = Vector3d::new(second.x - first.x, second.y - first.y, second.z - first.z);
    let across: Vector3d = Vector3d::new(third.x - second.x, third.y - second.y, third.z - second.z);

    Self::normalize(&Vector3d::new(
      along.y * across.z - along.z * across.y,
      along.z * across.x - along.x * across.z,
      along.x * across.y - along.y * across.x,
    ))
  }

  fn normalize(vector: &Vector3d) -> Vector3d {
    let length: f32 = (vector.x * vector.x + vector.y * vector.y + vector.z * vector.z).sqrt();

    if length > f32::EPSILON {
      Vector3d::new(vector.x / length, vector.y / length, vector.z / length)
    } else {
      Vector3d::new(0.0, 0.0, 0.0)
    }
  }
}

impl ChunkReadWrite for OgfLodDefinitionChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let definition: Self = Self {
      facets: [
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
      ],
    };

    reader.assert_read("Expect all data to be read from ogf lod definition chunk")?;

    Ok(definition)
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for facet in &self.facets {
      writer.write_xr::<T, _>(facet)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_math::Vector3d;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::chunks::ogf_lod_definition_chunk::OgfLodDefinitionChunk;
  use crate::data::ogf_box::OgfBox;
  use crate::data::ogf_lod_facet::OgfLodFacet;
  use crate::data::ogf_lod_vertex::OgfLodVertex;

  /// A unit square in the plane `z = depth`, wound so its triangles face `+z`.
  fn facet(depth: f32) -> OgfLodFacet {
    let corner = |x: f32, y: f32| OgfLodVertex {
      position: Vector3d::new(x, y, depth),
      rgb_hemi: 0x4d00_0000,
      sun: 16,
      texture_coordinate: (x, y),
    };

    OgfLodFacet {
      vertices: [corner(0.0, 0.0), corner(1.0, 0.0), corner(1.0, 1.0), corner(0.0, 1.0)],
    }
  }

  fn definition() -> OgfLodDefinitionChunk {
    OgfLodDefinitionChunk {
      facets: std::array::from_fn(|index: usize| facet(index as f32)),
    }
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();
    let original: OgfLodDefinitionChunk = definition();

    original.write::<XRayByteOrder>(&mut writer)?;

    // Eight facets of four 28-byte corners: the struct's padding is on disk, 896 bytes on every level measured.
    assert_eq!(writer.bytes_written(), 8 * 4 * OgfLodVertex::SIZE);

    writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      OgfLodDefinitionChunk::CHUNK_ID,
    )?;

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;
    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(OgfLodDefinitionChunk::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_facet_normal_faces_back_along_its_triangles() {
    let normal: Vector3d = definition().get_facet_normal(0);

    assert_eq!(
      (normal.x, normal.y, normal.z),
      (0.0, 0.0, -1.0),
      "the triangles face +z, the normal is inverted"
    );
  }

  #[test]
  fn test_lod_factor_is_the_share_of_the_disc_the_middle_extent_covers() {
    let bounding_box: OgfBox = OgfBox {
      max: Vector3d::new(1.0, 2.0, 1.0),
      min: Vector3d::new(-1.0, -2.0, -1.0),
    };
    let radius: f32 = 2.0;
    // Half extents 1, 1 and 2: the middle is 1. The band two across a disc of radius two.
    let expected: f32 = 4.0 * (0.5 * (4.0 * (0.5f32).asin() + 3.0f32.sqrt())) / (std::f32::consts::PI * 4.0);

    assert!((OgfLodDefinitionChunk::get_lod_factor(&bounding_box, radius) - expected).abs() < 1e-6);
  }
}

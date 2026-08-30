use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::ogf::ogf_bone_ik_data::OgfBoneIkData;

/// Per bone physics and bind pose data, `OGF_S_IKDATA`.
///
/// The chunk stores one record per bone with no count of its own, so the number of records comes from
/// the bone names chunk. That is why this cannot implement [`ChunkReadWrite`] and takes the count
/// explicitly, the same way the kinematics chunk takes its source id.
#[derive(Debug)]
pub struct OgfIkDataChunk {
  pub bones: Vec<OgfBoneIkData>,
}

impl OgfIkDataChunk {
  pub const CHUNK_ID: u32 = 16;

  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>, bones_count: usize) -> XrfResult<Self> {
    let mut bones: Vec<OgfBoneIkData> = reader.new_bounded_vec(
      bones_count as u64,
      OgfBoneIkData::MIN_SERIALIZED_SIZE,
      "ogf ik data bones",
    )?;

    for _ in 0..bones_count {
      bones.push(reader.read_xr::<T, _>()?);
    }

    reader.assert_read("Expect all data to be read from ogf ik data")?;

    Ok(Self { bones })
  }

  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for bone in &self.bones {
      bone.write::<T>(writer)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::io::Write;
  use xrf_chunk::InMemoryChunkDataSource;

  use xrf_chunk::{ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use super::OgfIkDataChunk;
  use crate::data::generic::vector_3d::Vector3d;
  use crate::data::ogf::ogf_bone_ik_data::OgfBoneIkData;
  use crate::data::ogf::ogf_bone_shape::OgfBoneShape;
  use crate::data::ogf::ogf_cylinder::OgfCylinder;
  use crate::data::ogf::ogf_joint_ik_data::OgfJointIkData;
  use crate::data::ogf::ogf_joint_limit::OgfJointLimit;
  use crate::data::ogf::ogf_obb::OgfObb;
  use crate::data::ogf::ogf_sphere::OgfSphere;

  fn vector(base: f32) -> Vector3d {
    Vector3d::new(base, base + 1.0, base + 2.0)
  }

  fn limit(base: f32) -> OgfJointLimit {
    OgfJointLimit {
      limit_from: base,
      limit_to: base + 1.0,
      spring_factor: base + 2.0,
      damping_factor: base + 3.0,
    }
  }

  fn bone(version: u16) -> OgfBoneIkData {
    OgfBoneIkData {
      version,
      game_material: String::from(r"materials\bone"),
      shape: OgfBoneShape {
        shape_type: 1,
        flags: 2,
        box_shape: OgfObb {
          rotate: [vector(1.0), vector(4.0), vector(7.0)],
          translate: vector(10.0),
          half_size: vector(13.0),
        },
        sphere: OgfSphere {
          position: vector(16.0),
          radius: 19.0,
        },
        cylinder: OgfCylinder {
          center: vector(20.0),
          direction: vector(23.0),
          height: 26.0,
          radius: 27.0,
        },
      },
      joint: OgfJointIkData {
        joint_type: 3,
        limits: [limit(30.0), limit(40.0), limit(50.0)],
        spring_factor: 60.0,
        damping_factor: 61.0,
        ik_flags: 1,
        break_force: 62.0,
        break_torque: 63.0,
        friction: if version > 0 { Some(64.0) } else { None },
      },
      bind_rotation: vector(70.0),
      bind_position: vector(73.0),
      mass: 76.0,
      center_of_mass: vector(77.0),
    }
  }

  fn write_then_read(name: &str, chunk: &OgfIkDataChunk) -> XrfResult<OgfIkDataChunk> {
    let filename: String = build_relative_test_sample_file_path(file!(), name);
    let mut writer: ChunkWriter = ChunkWriter::new();

    chunk.write::<XRayByteOrder>(&mut writer)?;

    let contents: Vec<u8> = writer.flush_chunk_into_buffer::<XRayByteOrder>(OgfIkDataChunk::CHUNK_ID)?;
    let mut file = overwrite_generated_test_resource_as_file(&filename)?;

    file.write_all(&contents)?;
    file.flush()?;

    let slice: FileSlice = open_generated_test_resource_as_slice(&filename)?;
    let mut reader: ChunkReader = ChunkReader::from_slice(slice)?
      .read_children()?
      .into_iter()
      .next()
      .expect("expect the written chunk to be present");

    OgfIkDataChunk::read::<XRayByteOrder, _>(&mut reader, chunk.bones.len())
  }

  #[test]
  fn round_trips_bones_with_friction() -> XrfResult {
    let chunk: OgfIkDataChunk = OgfIkDataChunk {
      bones: vec![bone(1), bone(1)],
    };

    let read: OgfIkDataChunk = write_then_read("versioned.chunk", &chunk)?;

    assert_eq!(read.bones.len(), 2);
    assert_eq!(read.bones[0], chunk.bones[0]);
    assert_eq!(read.bones[1], chunk.bones[1]);

    Ok(())
  }

  #[test]
  fn round_trips_version_zero_bones_without_friction() -> XrfResult {
    // Version 0 records stop before friction, so reading must not consume four bytes that are not there.
    let chunk: OgfIkDataChunk = OgfIkDataChunk { bones: vec![bone(0)] };

    let read: OgfIkDataChunk = write_then_read("unversioned.chunk", &chunk)?;

    assert_eq!(read.bones[0].joint.friction, None);
    assert_eq!(read.bones[0], chunk.bones[0]);

    Ok(())
  }

  #[test]
  fn rejects_bone_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[0, 0, 0, 0])?;

    let error: String = OgfIkDataChunk::read::<XRayByteOrder, _>(&mut reader, 1_000)
      .expect_err("expect the supplied bone count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("ogf ik data bones declares 1000 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}

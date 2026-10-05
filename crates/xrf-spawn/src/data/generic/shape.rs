use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section, read_ltx_field};
use xrf_math::Vector3d;
use xrf_utils::{assert_length, to_format_size};

use crate::types::{Matrix3d, Sphere3d};

/// Shape enumeration stored in objects descriptors.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Shape {
  Sphere(Sphere3d),
  Box(Matrix3d),
}

impl ChunkReadWrite for Shape {
  /// Read shape from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let shape_type: u8 = reader.read_u8().expect("Shape type to be read");

    Ok(match shape_type {
      0 => Self::Sphere((reader.read_xr::<T, _>()?, reader.read_f32::<T>()?)),
      1 => Self::Box((
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
      )),
      _ => {
        return Err(XrfError::new_parsing_error("Unexpected shape type provided"));
      }
    })
  }

  /// Write shape data into the chunk reader.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    match self {
      Self::Sphere(data) => {
        writer.write_u8(0)?;

        data.0.write::<T>(writer)?;

        writer.write_f32::<T>(data.1)?;
      }
      Self::Box(data) => {
        writer.write_u8(1)?;

        data.0.write::<T>(writer)?;
        data.1.write::<T>(writer)?;
        data.2.write::<T>(writer)?;
        data.3.write::<T>(writer)?;
      }
    }

    Ok(())
  }
}

impl ChunkReadWriteList for Shape {
  /// Read list of shapes from the chunk reader.
  fn read_list<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<Self>> {
    let mut shapes: Vec<Self> = Vec::new();
    let count: u8 = reader.read_u8().expect("Count flag to be read");

    for _ in 0..count {
      shapes.push(Self::read::<T, _>(reader)?);
    }

    assert_length(
      &shapes,
      count as usize,
      "Declared and read shapes count should be equal",
    )?;

    Ok(shapes)
  }

  /// Write list of shapes data into the chunk reader.
  fn write_list<T: ByteOrder>(writer: &mut ChunkWriter, shapes: &[Self]) -> XrfResult {
    writer.write_u8(to_format_size(shapes.len(), "shapes")?)?;

    for shape in shapes {
      shape.write::<T>(writer)?;
    }

    Ok(())
  }
}

impl Shape {
  /// `CCF_Shape::ComputeBounds`: the sphere an object's shapes are measured by, in its own space; its one sphere as it
  /// is, else the sphere about the box holding every shape, none for no shapes.
  pub fn get_bounding_sphere(shapes: &[Self]) -> Option<Sphere3d> {
    if let [Self::Sphere(sphere)] = shapes {
      return Some(sphere.clone());
    }

    let mut low: [f32; 3] = [f32::MAX; 3];
    let mut high: [f32; 3] = [f32::MIN; 3];
    let mut hold = |point: [f32; 3]| {
      for axis in 0..3 {
        low[axis] = low[axis].min(point[axis]);
        high[axis] = high[axis].max(point[axis]);
      }
    };

    for shape in shapes {
      match shape {
        Self::Sphere((center, radius)) => {
          hold([center.x - radius, center.y - radius, center.z - radius]);
          hold([center.x + radius, center.y + radius, center.z + radius]);
        }
        // The unit cube's corners where the box's matrix stands them, `transform_tiny`.
        Self::Box((i, j, k, c)) => {
          for corner in 0..8 {
            let [x, y, z] = [corner & 1, corner >> 1 & 1, corner >> 2 & 1].map(|bit| bit as f32 - 0.5);

            hold([
              c.x + i.x * x + j.x * y + k.x * z,
              c.y + i.y * x + j.y * y + k.y * z,
              c.z + i.z * x + j.z * y + k.z * z,
            ]);
          }
        }
      }
    }

    if shapes.is_empty() {
      return None;
    }

    // `Fbox::getsphere`: its centre, and the distance to its far corner.
    let center: [f32; 3] = [0, 1, 2].map(|axis| (low[axis] + high[axis]) / 2.0);
    let radius: f32 = (0..3)
      .map(|axis| (high[axis] - center[axis]).powi(2))
      .sum::<f32>()
      .sqrt();

    Some((Vector3d::new(center[0], center[1], center[2]), radius))
  }

  /// Import shape objects from ltx config file.
  pub fn import_list(section: &Section) -> XrfResult<Vec<Self>> {
    let mut shapes: Vec<Self> = Vec::new();
    let count: usize = read_ltx_field("shapes_count", section)?;

    for index in 0..count {
      let prefix: String = format!("shape.{index}");
      let shape_type: String = read_ltx_field(&format!("{prefix}.type"), section)?;

      match shape_type.as_str() {
        "sphere" => {
          shapes.push(Self::Sphere((
            read_ltx_field(&format!("{prefix}.center"), section)?,
            read_ltx_field(&format!("{prefix}.radius"), section)?,
          )));
        }
        "box" => {
          shapes.push(Self::Box((
            read_ltx_field(&format!("{prefix}.a"), section)?,
            read_ltx_field(&format!("{prefix}.b"), section)?,
            read_ltx_field(&format!("{prefix}.c"), section)?,
            read_ltx_field(&format!("{prefix}.d"), section)?,
          )));
        }
        _ => {
          return Err(XrfError::new_parsing_error(format!(
            "Failed to parsed unknown type of shape - {shape_type} when importing from ltx"
          )));
        }
      }
    }

    Ok(shapes)
  }

  /// Export shapes object to target ltx file section.
  pub fn export_list(shapes: &[Self], section_name: &str, ltx: &mut Ltx) {
    ltx
      .with_section(section_name)
      .set("shapes_count", shapes.len().to_string());

    for (index, shape) in shapes.iter().enumerate() {
      let prefix: String = format!("shape.{index}");

      match shape {
        Self::Sphere(sphere) => {
          ltx
            .with_section(section_name)
            .set(format!("{prefix}.type"), "sphere")
            .set(format!("{prefix}.center"), sphere.0.to_string())
            .set(format!("{prefix}.radius"), sphere.1.to_string());
        }
        Self::Box(square) => {
          ltx
            .with_section(section_name)
            .set(format!("{prefix}.type"), "box")
            .set(format!("{prefix}.a"), square.0.to_string())
            .set(format!("{prefix}.b"), square.1.to_string())
            .set(format!("{prefix}.c"), square.2.to_string())
            .set(format!("{prefix}.d"), square.3.to_string());
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};
  use std::path::Path;

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_math::Vector3d;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_sample_file_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_file, overwrite_generated_test_resource_as_file,
  };

  use crate::data::generic::shape::Shape;

  #[test]
  fn measures_one_sphere_as_it_is_and_anything_else_by_the_box_about_it() {
    let one: Vec<Shape> = vec![Shape::Sphere((Vector3d::new(1.0, 2.0, 3.0), 4.0))];

    assert_eq!(
      Shape::get_bounding_sphere(&one),
      Some((Vector3d::new(1.0, 2.0, 3.0), 4.0))
    );
    assert_eq!(Shape::get_bounding_sphere(&[]), None);

    // A box two wide, four tall and six deep at the origin: the sphere through its corners.
    let block: Vec<Shape> = vec![Shape::Box((
      Vector3d::new(2.0, 0.0, 0.0),
      Vector3d::new(0.0, 4.0, 0.0),
      Vector3d::new(0.0, 0.0, 6.0),
      Vector3d::new(0.0, 0.0, 0.0),
    ))];
    let (center, radius) = Shape::get_bounding_sphere(&block).unwrap();

    assert_eq!(center, Vector3d::new(0.0, 0.0, 0.0));
    assert!((radius - 14.0_f32.sqrt()).abs() < 1e-5);

    // Two spheres: the box about both, not either sphere.
    let pair: Vec<Shape> = vec![
      Shape::Sphere((Vector3d::new(-2.0, 0.0, 0.0), 1.0)),
      Shape::Sphere((Vector3d::new(2.0, 0.0, 0.0), 1.0)),
    ];
    let (center, radius) = Shape::get_bounding_sphere(&pair).unwrap();

    assert_eq!(center, Vector3d::new(0.0, 0.0, 0.0));
    assert!((radius - 11.0_f32.sqrt()).abs() < 1e-5);
  }

  #[test]
  fn rejects_a_shape_list_past_its_count_field() {
    let shapes: Vec<Shape> = vec![Shape::Sphere((Vector3d::new(0.0, 0.0, 0.0), 1.0)); 256];
    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(
      Shape::write_list::<XRayByteOrder>(&mut writer, &shapes)
        .expect_err("expect the shape count to exceed its format field")
        .to_string(),
      "Invalid error: shapes exceeds the u8 format limit"
    );
  }

  #[test]
  fn test_read_write_list() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write_list.chunk");

    let original: Vec<Shape> = vec![
      Shape::Sphere((
        Vector3d {
          x: 125.465,
          y: 456.123,
          z: 675.345,
        },
        150.0,
      )),
      Shape::Box((
        Vector3d {
          x: 10.5,
          y: 10.7,
          z: 10.0,
        },
        Vector3d {
          x: 20.5,
          y: 20.7,
          z: 20.0,
        },
        Vector3d {
          x: 30.5,
          y: 30.7,
          z: 30.0,
        },
        Vector3d {
          x: 40.5,
          y: 40.7,
          z: 40.0,
        },
      )),
    ];

    Shape::write_list::<XRayByteOrder>(&mut writer, &original)?;

    assert_eq!(writer.bytes_written(), 67);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 67);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 67 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(Shape::read_list::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write_sphere() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write_sphere.chunk");

    let original: Shape = Shape::Sphere((
      Vector3d {
        x: 25.5,
        y: 3.4,
        z: 45.1,
      },
      150.0,
    ));

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 17);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 17);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 17 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(Shape::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write_box() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write_box.chunk");

    let original: Shape = Shape::Box((
      Vector3d { x: 1.5, y: 1.7, z: 1.0 },
      Vector3d { x: 2.5, y: 2.7, z: 2.0 },
      Vector3d { x: 3.5, y: 3.7, z: 3.0 },
      Vector3d { x: 4.5, y: 4.7, z: 4.0 },
    ));

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 49);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 49);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 49 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(Shape::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let config_path: &Path = &build_absolute_generated_test_sample_file_path(file!(), "test_import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: Vec<Shape> = vec![
      Shape::Sphere((
        Vector3d {
          x: 1634.465,
          y: 2652.123,
          z: 3624.345,
        },
        150.0,
      )),
      Shape::Box((
        Vector3d {
          x: 1000.5,
          y: 1000.7,
          z: 1000.0,
        },
        Vector3d {
          x: 2000.5,
          y: 2000.7,
          z: 2000.0,
        },
        Vector3d {
          x: 3000.5,
          y: 3000.7,
          z: 3000.0,
        },
        Vector3d {
          x: 4000.5,
          y: 4000.7,
          z: 4000.0,
        },
      )),
    ];

    Shape::export_list(&original, "data", &mut ltx);
    ltx.write_to(&mut overwrite_file(config_path)?)?;

    assert_eq!(
      Shape::import_list(Ltx::read_from_path(config_path)?.section("data").unwrap(),)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize_sphere() -> XrfResult {
    let original: Shape = Shape::Sphere((
      Vector3d {
        x: 243.5,
        y: 456.4,
        z: 475.1,
      },
      52.0,
    ));

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize_sphere.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<Shape>(&serialized)?);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize_box() -> XrfResult {
    let original: Shape = Shape::Box((
      Vector3d {
        x: 175.5,
        y: 135.7,
        z: 163.0,
      },
      Vector3d {
        x: 264.5,
        y: 274.7,
        z: 244.0,
      },
      Vector3d {
        x: 375.5,
        y: 385.7,
        z: 386.0,
      },
      Vector3d {
        x: 498.5,
        y: 460.7,
        z: 489.0,
      },
    ));

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize_box.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<Shape>(&serialized)?);

    Ok(())
  }
}

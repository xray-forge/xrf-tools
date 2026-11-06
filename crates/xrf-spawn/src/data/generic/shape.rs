use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_math::{Aabb, Affine3d, Sphere, Vector3d};
use xrf_utils::{assert_length, to_format_size};

/// One shape of an object's volume, `CShapeData::shape_def`, in the object's own space.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Shape {
  Sphere(Sphere),
  /// The unit cube centred on the origin, placed by the transform: its axes are the box's sides and its
  /// translation the box's centre.
  Box(Affine3d),
}

impl Shape {
  /// `cfSphere`, the type byte ahead of a sphere.
  pub const SPHERE_TYPE: u8 = 0;

  /// `cfBox`, the type byte ahead of a box.
  pub const BOX_TYPE: u8 = 1;
}

impl ChunkReadWrite for Shape {
  /// Read shape from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(match reader.read_u8()? {
      Self::SPHERE_TYPE => Self::Sphere(reader.read_xr::<T, _>()?),
      Self::BOX_TYPE => Self::Box(reader.read_xr::<T, _>()?),
      _ => {
        return Err(XrfError::new_parsing_error("Unexpected shape type provided"));
      }
    })
  }

  /// Write shape data into the chunk reader.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    match self {
      Self::Sphere(sphere) => {
        writer.write_u8(Self::SPHERE_TYPE)?;
        sphere.write::<T>(writer)?;
      }
      Self::Box(transform) => {
        writer.write_u8(Self::BOX_TYPE)?;
        transform.write::<T>(writer)?;
      }
    }

    Ok(())
  }
}

impl ChunkReadWriteList for Shape {
  /// Read list of shapes from the chunk reader.
  fn read_list<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<Self>> {
    let mut shapes: Vec<Self> = Vec::new();
    let count: u8 = reader.read_u8()?;

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
  pub fn get_bounding_sphere(shapes: &[Self]) -> Option<Sphere> {
    if let [Self::Sphere(sphere)] = shapes {
      return Some(sphere.clone());
    }

    Aabb::from_points(shapes.iter().flat_map(Self::get_extreme_points)).map(|bounds: Aabb| bounds.get_bounding_sphere())
  }

  /// The points whose box is the shape's own: a sphere's lowest and highest corner, a box's eight corners where its
  /// transform places the unit cube's, `transform_tiny`.
  fn get_extreme_points(&self) -> Vec<Vector3d> {
    match self {
      Self::Sphere(sphere) => {
        let reach: Vector3d = Vector3d::new(sphere.radius, sphere.radius, sphere.radius);

        vec![&sphere.center - &reach, &sphere.center + &reach]
      }
      Self::Box(transform) => (0..8)
        .map(|corner: u8| {
          let [x, y, z] = [corner & 1, corner >> 1 & 1, corner >> 2 & 1].map(|bit: u8| f32::from(bit) - 0.5);

          transform.transform_point(&Vector3d::new(x, y, z))
        })
        .collect(),
    }
  }

  /// Import shape objects from ltx config file.
  pub fn import_list(section: &Section) -> XrfResult<Vec<Self>> {
    let mut shapes: Vec<Self> = Vec::new();
    let count: usize = section.read_field("shapes_count")?;

    for index in 0..count {
      let prefix: String = format!("shape.{index}");
      let shape_type: String = section.read_field(&format!("{prefix}.type"))?;

      match shape_type.as_str() {
        "sphere" => {
          shapes.push(Self::Sphere(Sphere {
            center: section.read_field(&format!("{prefix}.center"))?,
            radius: section.read_field(&format!("{prefix}.radius"))?,
          }));
        }
        "box" => {
          shapes.push(Self::Box(Affine3d {
            x_axis: section.read_field(&format!("{prefix}.a"))?,
            y_axis: section.read_field(&format!("{prefix}.b"))?,
            z_axis: section.read_field(&format!("{prefix}.c"))?,
            translation: section.read_field(&format!("{prefix}.d"))?,
          }));
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
            .set(format!("{prefix}.center"), sphere.center.to_string())
            .set(format!("{prefix}.radius"), sphere.radius.to_string());
        }
        Self::Box(transform) => {
          ltx
            .with_section(section_name)
            .set(format!("{prefix}.type"), "box")
            .set(format!("{prefix}.a"), transform.x_axis.to_string())
            .set(format!("{prefix}.b"), transform.y_axis.to_string())
            .set(format!("{prefix}.c"), transform.z_axis.to_string())
            .set(format!("{prefix}.d"), transform.translation.to_string());
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write, read_to_string};
  use std::path::Path;

  use byteorder::ReadBytesExt;
  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_math::{Affine3d, Sphere, Vector3d};
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::data::generic::shape::Shape;

  fn sphere(x: f32, y: f32, z: f32, radius: f32) -> Shape {
    Shape::Sphere(Sphere {
      center: Vector3d::new(x, y, z),
      radius,
    })
  }

  /// A box whose rows are the four given vectors, each `(n + 0.5, n + 0.7, n)` times `scale` for row `n`.
  fn box_of_rows(scale: f32) -> Shape {
    let row = |n: f32| Vector3d::new((n + 0.5) * scale, (n + 0.7) * scale, n * scale);

    Shape::Box(Affine3d {
      x_axis: row(1.0),
      y_axis: row(2.0),
      z_axis: row(3.0),
      translation: row(4.0),
    })
  }

  /// A truncated shape list is an error, not a panic that a release build turns into an abort.
  #[test]
  fn refuses_a_shape_list_cut_short() -> XrfResult {
    // A count with no shape tag, and a sphere tag with no sphere.
    for payload in [&[1][..], &[1, 0]] {
      assert!(
        Shape::read_list::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(payload)?).is_err(),
        "a shape list of {payload:?} must be refused"
      );
    }

    // No count at all: the list starts where the object's data already ended.
    let mut exhausted = ChunkReader::from_bytes(&[7])?;

    exhausted.read_u8()?;

    assert!(Shape::read_list::<XRayByteOrder, _>(&mut exhausted).is_err());

    Ok(())
  }

  #[test]
  fn measures_one_sphere_as_it_is_and_anything_else_by_the_box_about_it() {
    assert_eq!(
      Shape::get_bounding_sphere(&[sphere(1.0, 2.0, 3.0, 4.0)]),
      Some(Sphere {
        center: Vector3d::new(1.0, 2.0, 3.0),
        radius: 4.0,
      })
    );
    assert_eq!(Shape::get_bounding_sphere(&[]), None);

    // A box two wide, four tall and six deep at the origin: the sphere through its corners.
    let block: Vec<Shape> = vec![Shape::Box(Affine3d {
      x_axis: Vector3d::new(2.0, 0.0, 0.0),
      y_axis: Vector3d::new(0.0, 4.0, 0.0),
      z_axis: Vector3d::new(0.0, 0.0, 6.0),
      translation: Vector3d::ZERO,
    })];
    let Sphere { center, radius } = Shape::get_bounding_sphere(&block).unwrap();

    assert_eq!(center, Vector3d::ZERO);
    assert!((radius - 14.0_f32.sqrt()).abs() < 1e-5);

    // Two spheres: the box about both, not either sphere.
    let pair: Vec<Shape> = vec![sphere(-2.0, 0.0, 0.0, 1.0), sphere(2.0, 0.0, 0.0, 1.0)];
    let Sphere { center, radius } = Shape::get_bounding_sphere(&pair).unwrap();

    assert_eq!(center, Vector3d::ZERO);
    assert!((radius - 11.0_f32.sqrt()).abs() < 1e-5);
  }

  #[test]
  fn rejects_a_shape_list_past_its_count_field() {
    let shapes: Vec<Shape> = vec![sphere(0.0, 0.0, 0.0, 1.0); 256];
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
    let filename: String = GeneratedResources::sample_name(file!(), "read_write_list.chunk");
    let original: Vec<Shape> = vec![sphere(125.465, 456.123, 675.345, 150.0), box_of_rows(10.0)];

    Shape::write_list::<XRayByteOrder>(&mut writer, &original)?;

    assert_eq!(writer.bytes_written(), 67);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut GeneratedResources::overwrite(&filename)?, 0)?;

    assert_eq!(bytes_written, 67);

    let file: FileSlice = GeneratedResources::open_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 67 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(Shape::read_list::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write_sphere() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = GeneratedResources::sample_name(file!(), "read_write_sphere.chunk");
    let original: Shape = sphere(25.5, 3.4, 45.1, 150.0);

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 17);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut GeneratedResources::overwrite(&filename)?, 0)?;

    assert_eq!(bytes_written, 17);

    let file: FileSlice = GeneratedResources::open_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 17 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(Shape::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write_box() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = GeneratedResources::sample_name(file!(), "read_write_box.chunk");
    let original: Shape = box_of_rows(1.0);

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 49);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut GeneratedResources::overwrite(&filename)?, 0)?;

    assert_eq!(bytes_written, 49);

    let file: FileSlice = GeneratedResources::open_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 49 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(Shape::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let config_path: &Path = &GeneratedResources::sample_path(file!(), "test_import_export.ltx");
    let mut ltx: Ltx = Ltx::new();
    let original: Vec<Shape> = vec![sphere(1634.465, 2652.123, 3624.345, 150.0), box_of_rows(1000.0)];

    Shape::export_list(&original, "data", &mut ltx);
    ltx.write_to(&mut GeneratedResources::overwrite_path(config_path)?)?;

    assert_eq!(
      Shape::import_list(Ltx::read_from_path(config_path)?.section("data").unwrap())?,
      original
    );

    Ok(())
  }

  /// The config keys are the format's, kept from before the shapes had named fields: a box's rows are `a` to `d`.
  #[test]
  fn exports_a_box_under_the_keys_existing_configs_use() {
    let mut ltx: Ltx = Ltx::new();

    Shape::export_list(&[box_of_rows(1.0)], "data", &mut ltx);

    let section = ltx.section("data").unwrap();

    assert_eq!(section.get("shape.0.a"), Some("1.5,1.7,1"));
    assert_eq!(section.get("shape.0.d"), Some("4.5,4.7,4"));
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    for (name, original) in [
      ("serialize_deserialize_sphere.json", sphere(243.5, 456.4, 475.1, 52.0)),
      ("serialize_deserialize_box.json", box_of_rows(100.0)),
    ] {
      let mut file: File = GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), name))?;

      file.write_all(to_string_pretty(&original)?.as_bytes())?;
      file.seek(SeekFrom::Start(0))?;

      assert_eq!(original, serde_json::from_str::<Shape>(&read_to_string(&mut file)?)?);
    }

    Ok(())
  }
}

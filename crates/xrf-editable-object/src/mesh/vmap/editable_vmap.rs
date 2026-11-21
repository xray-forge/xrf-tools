use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{assert_count_fits, narrow_format_size};

use crate::mesh::vmap::editable_vmap_kind::EditableVMapKind;

/// One vertex map of a mesh, `st_VMap` as `EMESH_CHUNK_VMAPS_2` stores it: a UV set or a bone's weights, each entry
/// tied to the point it belongs to and, in a polygon map, to the face it is for.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableVMap {
  pub name: String,
  /// Values per entry: 2 for a UV map, 1 for a weight map.
  pub dimension: u8,
  pub kind: EditableVMapKind,
  /// `dimension` values per entry, entry after entry.
  pub values: Vec<f32>,
  /// The point each entry belongs to.
  pub vertex_indices: Vec<u32>,
  /// The face each entry is for in a polygon map, which lets a seam give one point two UVs. `None` where entries hold
  /// for every face around their point.
  pub polygon_indices: Option<Vec<u32>>,
}

impl EditableVMap {
  /// An empty name, three bytes, and a count of no entries.
  pub const MIN_SERIALIZED_SIZE: u64 = 1 + 3 + 4;

  /// Entries in the map.
  pub const fn get_count(&self) -> usize {
    self.vertex_indices.len()
  }

  /// One entry's values, `dimension` of them, or `None` past the last entry.
  pub fn get_entry(&self, index: usize) -> Option<&[f32]> {
    let dimension: usize = usize::from(self.dimension);
    let start: usize = index.checked_mul(dimension)?;

    self.values.get(start..start.checked_add(dimension)?)
  }

  /// Refuses a map whose lists disagree on its entry count.
  fn assert_consistent(&self) -> XrfResult {
    let count: usize = self.get_count();
    let is_consistent: bool = self.values.len() == count * usize::from(self.dimension)
      && self
        .polygon_indices
        .as_ref()
        .is_none_or(|indices| indices.len() == count);

    if is_consistent {
      Ok(())
    } else {
      Err(XrfError::new_invalid_error(format!(
        "Vertex map '{}' holds {} values, {count} vertex indices and {:?} polygon indices, which no one count describes",
        self.name,
        self.values.len(),
        self.polygon_indices.as_ref().map(Vec::len)
      )))
    }
  }
}

impl ChunkReadWrite for EditableVMap {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let name: String = reader.read_w1251_string()?;
    let dimension: u8 = reader.read_u8()?;
    let polymap: u8 = reader.read_u8()?;
    let kind: EditableVMapKind = EditableVMapKind::from(reader.read_u8()?);
    let count: u32 = reader.read_u32()?;

    let is_polymap: bool = match polymap {
      0 => false,
      1 => true,
      other => {
        return Err(XrfError::new_parsing_error(format!(
          "Unexpected polygon map mark {other} on vertex map '{name}', the editor stores 0 or 1"
        )));
      }
    };

    let entry_size: u64 = (u64::from(dimension) + 1 + u64::from(is_polymap)) * 4;
    let value_count: u64 = u64::from(count) * u64::from(dimension);

    assert_count_fits(
      count.into(),
      reader.read_bytes_remain(),
      entry_size,
      "vertex map entries",
    )?;

    let values: Vec<f32> = reader.read_records(value_count, "vertex map values")?;
    let vertex_indices: Vec<u32> = reader.read_records(count.into(), "vertex map indices")?;
    let polygon_indices: Option<Vec<u32>> = if is_polymap {
      Some(reader.read_records(count.into(), "vertex map indices")?)
    } else {
      None
    };

    Ok(Self {
      name,
      dimension,
      kind,
      values,
      vertex_indices,
      polygon_indices,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.assert_consistent()?;

    writer.write_w1251_string(&self.name)?;
    writer.write_u8(self.dimension)?;
    writer.write_u8(u8::from(self.polygon_indices.is_some()))?;
    writer.write_u8(u8::from(self.kind))?;
    writer.write_u32(narrow_format_size(self.get_count(), "Vertex map entries")?)?;

    writer.write_records(&self.values)?;
    writer.write_records(&self.vertex_indices)?;

    if let Some(polygon_indices) = &self.polygon_indices {
      writer.write_records(polygon_indices)?;
    }

    Ok(())
  }
}

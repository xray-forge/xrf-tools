use byteorder::ByteOrder;
use xrf_chunk::{ChunkReadWrite, ChunkWriter};
use xrf_editable_object::{EditableObjectFile, EditableTimeWidth};
use xrf_error::XrfResult;
use xrf_math::{Aabb, Sphere, Vector3d};
use xrf_ogf::{OgfDescriptionChunk, OgfDescriptionTimeWidth, OgfHeaderChunk, OgfModelType};

use crate::object::compile::object_compile_source::ObjectCompileSource;

/// The chunks the skeleton and the static export write alike: a visual's header and the object's description.
pub(crate) struct OgfExportChunks;

impl OgfExportChunks {
  /// `xrOGF_FormatVersion`.
  const FORMAT_VERSION: u8 = 4;

  /// A visual's header: its kind and bounds, and the sphere `Fbox::getsphere` draws around them, from the centre out to
  /// the upper corner.
  pub(crate) fn header(model_type: OgfModelType, bounds: &Aabb) -> OgfHeaderChunk {
    let center: Vector3d = bounds.min.midpoint(&bounds.max);

    OgfHeaderChunk {
      version: Self::FORMAT_VERSION,
      model_type: model_type.to_raw(),
      shader_id: 0,
      bounding_box: *bounds,
      bounding_sphere: Sphere {
        center,
        radius: center.distance(&bounds.max),
      },
    }
  }

  /// The description `CEditableObject::PrepareOGFDesc` fills: the source, what built it when, and who made and last
  /// changed the object when.
  pub(crate) fn description(object: &EditableObjectFile, source: &ObjectCompileSource) -> OgfDescriptionChunk {
    let authored = object.description.as_ref();

    OgfDescriptionChunk {
      source_file: source.source_file.clone(),
      convertor: source.convertor.clone(),
      built_at: source.built_at,
      creator: authored.map(|it| it.creator.clone()).unwrap_or_default(),
      created_at: authored.map_or(0, |it| it.created_at),
      editor: authored.map(|it| it.modifier.clone()).unwrap_or_default(),
      edited_at: authored.map_or(0, |it| it.modified_at),
      time_width: match authored.map(|it| it.time_width) {
        Some(EditableTimeWidth::Seconds64) => OgfDescriptionTimeWidth::Wide,
        _ => OgfDescriptionTimeWidth::Narrow,
      },
    }
  }

  /// The box around `positions`, or the zero box around none.
  pub(crate) fn get_bounds<'a>(positions: impl IntoIterator<Item = &'a Vector3d>) -> Aabb {
    let mut positions = positions.into_iter();
    let Some(first) = positions.next() else {
      return Aabb {
        min: Vector3d::ZERO,
        max: Vector3d::ZERO,
      };
    };

    positions.fold(
      Aabb {
        min: *first,
        max: *first,
      },
      |bounds, position| Aabb {
        min: bounds.min.min(position),
        max: bounds.max.max(position),
      },
    )
  }

  /// Writes `chunk` into `writer` as chunk `id`.
  pub(crate) fn write_chunk<T: ByteOrder, C: ChunkReadWrite>(
    writer: &mut ChunkWriter,
    id: u32,
    chunk: &C,
  ) -> XrfResult {
    let mut contents: ChunkWriter = ChunkWriter::new();

    chunk.write::<T>(&mut contents)?;
    contents.flush_chunk_into::<T>(writer, id)?;

    Ok(())
  }
}

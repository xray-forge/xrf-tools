use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::EditableExtraChunk;
use crate::chunks::editable_chunk_sequence::EditableChunkSequence;

/// How one container of an editable object was laid out in its file, apart from what it holds: the order its chunks
/// were stored in and the chunks no field of it takes. The SDK finds every chunk by id, so neither means anything to
/// it; keeping both is what lets a file come back byte for byte whichever editor or exporter saved it.
///
/// The default is the layout of a container built in code: written in the order the editor's `Save` writes, with
/// nothing extra.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditableChunkLayout {
  /// The container's chunk ids in the order the file stored them.
  pub order: Vec<u32>,
  /// Chunks no field takes, see [`EditableExtraChunk`].
  pub extra: Vec<EditableExtraChunk>,
}

impl EditableChunkLayout {
  /// Reads every child of a container, handing each to `fold`, which takes it into a field and answers `true`, or
  /// answers `false` to leave it to be kept as stored.
  pub(crate) fn read<D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    mut fold: impl FnMut(&mut ChunkReader<D>) -> XrfResult<bool>,
  ) -> XrfResult<Self> {
    let children: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut layout: Self = Self {
      order: children.iter().map(|chunk| chunk.id).collect(),
      extra: Vec::new(),
    };

    for mut chunk in children {
      let is_folded: bool = fold(&mut chunk)?;

      if !is_folded {
        layout.extra.push(EditableExtraChunk {
          id: chunk.id,
          bytes: chunk.read_remaining()?,
        });
      }
    }

    Ok(layout)
  }

  /// Puts down a container's chunks, encoded into `sequence` in `Save`'s order, and the extra ones in this layout.
  pub(crate) fn write<T: ByteOrder>(
    &self,
    destination: &mut ChunkWriter,
    mut sequence: EditableChunkSequence,
  ) -> XrfResult {
    sequence.push_extra(&self.extra);
    sequence.write_into::<T>(destination, &self.order)
  }
}

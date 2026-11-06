use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::layout::chunk_payload::ChunkPayload;
use crate::layout::chunk_sequence::ChunkSequence;
use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_data_source::ChunkDataSource;
use crate::writer::chunk_writer::ChunkWriter;

/// How a container's children were laid out in a file, apart from what they hold: the order the file stored them in,
/// and the children no field of the container takes, kept as stored.
///
/// A format that finds its chunks by id gives that order no meaning, and its own writer puts chunks down in one order of
/// its own; keeping the stored order is what lets a file another writer laid out come back byte for byte. The default
/// is the layout of a container built in code: nothing kept, and the format's own order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChunkLayout {
  /// The children's ids in the order the file stored them.
  pub order: Vec<u32>,
  /// The children no field takes, a second copy of one that a field does included, in the order they were read.
  pub kept: Vec<ChunkPayload>,
}

impl ChunkLayout {
  /// Reads every child of `reader`, handing each to `fold`, which takes it into a field and answers `true`, or answers
  /// `false` to have it kept as stored.
  ///
  /// # Errors
  ///
  /// Returns an error when the children are not well-formed chunks, one is compressed (which
  /// [`ChunkReader::read_children`] refuses rather than keep bytes that would read as something else), or `fold`
  /// returns one.
  pub fn read<D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    mut fold: impl FnMut(&mut ChunkReader<D>) -> XrfResult<bool>,
  ) -> XrfResult<Self> {
    let children: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut layout: Self = Self {
      order: children.iter().map(|chunk| chunk.id).collect(),
      kept: Vec::new(),
    };

    for mut chunk in children {
      let is_folded: bool = fold(&mut chunk)?;

      if !is_folded {
        layout.kept.push(ChunkPayload {
          id: chunk.id,
          bytes: chunk.read_remaining()?,
        });
      }
    }

    Ok(layout)
  }

  /// Puts down the children `sequence` encoded and the kept ones: in [`Self::order`] first, each id taking the first
  /// child with that id not yet written, then the rest in the order they were pushed, the kept ones last.
  ///
  /// # Errors
  ///
  /// Returns an error when a payload is larger than a chunk can declare.
  pub fn write<T: ByteOrder>(&self, destination: &mut ChunkWriter, mut sequence: ChunkSequence) -> XrfResult {
    sequence.push_kept(&self.kept);
    sequence.write_in_order::<T>(destination, &self.order)
  }
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use byteorder::{ReadBytesExt, WriteBytesExt};
  use xrf_error::XrfResult;

  use crate::{
    ChunkLayout, ChunkPayload, ChunkReader, ChunkSequence, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder,
  };

  /// Children framed one after another under a parent's payload.
  fn container(children: &[ChunkPayload]) -> XrfResult<ChunkReader<InMemoryChunkDataSource>> {
    let mut payload: ChunkWriter = ChunkWriter::new();

    for child in children {
      child.write::<XRayByteOrder>(&mut payload)?;
    }

    ChunkReader::from_vec(payload.flush_chunk_into_buffer::<XRayByteOrder>(1)?)?.read_child_by_index(0)
  }

  fn payload(id: u32, bytes: &[u8]) -> ChunkPayload {
    ChunkPayload {
      id,
      bytes: bytes.to_vec(),
    }
  }

  /// A container whose writer knows ids 10 and 20 and writes them in that order.
  struct Pair {
    first: Option<u8>,
    second: Option<u8>,
    layout: ChunkLayout,
  }

  impl Pair {
    fn read(children: &[ChunkPayload]) -> XrfResult<Self> {
      let mut first: Option<u8> = None;
      let mut second: Option<u8> = None;
      let layout: ChunkLayout = ChunkLayout::read(&mut container(children)?, |chunk| {
        match chunk.id {
          10 if first.is_none() => first = Some(chunk.read_u8()?),
          20 if second.is_none() => second = Some(chunk.read_u8()?),
          _ => return Ok(false),
        }

        Ok(true)
      })?;

      Ok(Self { first, second, layout })
    }

    fn write(&self) -> XrfResult<Vec<u8>> {
      let mut sequence: ChunkSequence = ChunkSequence::new();

      if let Some(value) = self.first {
        sequence.push(10, |writer| Ok(writer.write_u8(value)?))?;
      }

      if let Some(value) = self.second {
        sequence.push(20, |writer| Ok(writer.write_u8(value)?))?;
      }

      let mut destination: ChunkWriter = ChunkWriter::new();

      self.layout.write::<XRayByteOrder>(&mut destination, sequence)?;

      Ok(destination.buffer)
    }
  }

  fn framed(children: &[ChunkPayload]) -> XrfResult<Vec<u8>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    for child in children {
      child.write::<XRayByteOrder>(&mut writer)?;
    }

    Ok(writer.buffer)
  }

  #[test]
  fn a_container_comes_back_in_the_order_its_file_stored_it() -> XrfResult {
    // Stored the other way round from the writer's own order, with a chunk nothing names in between.
    let stored: Vec<ChunkPayload> = vec![payload(20, &[2]), payload(99, &[9, 9]), payload(10, &[1])];
    let pair: Pair = Pair::read(&stored)?;

    assert_eq!(pair.first, Some(1));
    assert_eq!(pair.second, Some(2));
    assert_eq!(pair.layout.kept, vec![payload(99, &[9, 9])]);
    assert_eq!(pair.write()?, framed(&stored)?);

    Ok(())
  }

  #[test]
  fn a_duplicate_of_a_folded_chunk_is_kept_and_comes_back_second() -> XrfResult {
    let stored: Vec<ChunkPayload> = vec![payload(10, &[1]), payload(10, &[7]), payload(20, &[2])];
    let pair: Pair = Pair::read(&stored)?;

    assert_eq!(
      pair.first,
      Some(1),
      "the first copy is the field, as a reader finding by id takes it"
    );
    assert_eq!(pair.layout.kept, vec![payload(10, &[7])]);
    assert_eq!(pair.write()?, framed(&stored)?);

    Ok(())
  }

  #[test]
  fn a_container_built_in_code_is_written_in_its_writers_order() -> XrfResult {
    let pair: Pair = Pair {
      first: Some(1),
      second: Some(2),
      layout: ChunkLayout::default(),
    };

    assert_eq!(pair.write()?, framed(&[payload(10, &[1]), payload(20, &[2])])?);

    Ok(())
  }

  #[test]
  fn a_chunk_added_after_reading_follows_the_stored_ones() -> XrfResult {
    let mut pair: Pair = Pair::read(&[payload(20, &[2]), payload(99, &[9])])?;

    pair.first = Some(1);

    assert_eq!(
      pair.write()?,
      framed(&[payload(20, &[2]), payload(99, &[9]), payload(10, &[1])])?
    );

    Ok(())
  }

  #[test]
  fn a_chunk_removed_after_reading_leaves_no_gap() -> XrfResult {
    let mut pair: Pair = Pair::read(&[payload(20, &[2]), payload(10, &[1])])?;

    pair.second = None;

    assert_eq!(pair.write()?, framed(&[payload(10, &[1])])?);

    Ok(())
  }

  #[test]
  fn a_payload_writes_framed_under_its_id() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    payload(7, b"ab").write::<XRayByteOrder>(&mut writer)?;

    let mut expected: Vec<u8> = Vec::new();

    expected.write_u32::<XRayByteOrder>(7)?;
    expected.write_u32::<XRayByteOrder>(2)?;
    expected.write_all(b"ab")?;

    assert_eq!(writer.buffer, expected);

    Ok(())
  }
}

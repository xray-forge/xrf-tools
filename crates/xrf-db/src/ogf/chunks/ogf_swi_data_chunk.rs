use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::ogf::ogf_slide_window::OgfSlideWindow;

/// Progressive mesh level of detail table, `OGF_SWIDATA`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfSwiDataChunk {
  /// Read and written verbatim, unused by the engine.
  pub reserved: [u32; 4],
  pub windows: Vec<OgfSlideWindow>,
}

impl OgfSwiDataChunk {
  pub const CHUNK_ID: u32 = 6;
}

impl ChunkReadWrite for OgfSwiDataChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let reserved: [u32; 4] = [
      reader.read_u32::<T>()?,
      reader.read_u32::<T>()?,
      reader.read_u32::<T>()?,
      reader.read_u32::<T>()?,
    ];

    let count: u32 = reader.read_u32::<T>()?;
    let mut windows: Vec<OgfSlideWindow> =
      reader.new_bounded_vec(count.into(), OgfSlideWindow::MIN_SERIALIZED_SIZE, "ogf slide windows")?;

    for _ in 0..count {
      windows.push(reader.read_xr::<T, _>()?);
    }

    reader.assert_read("Expect all data to be read from ogf swi data")?;

    Ok(Self { reserved, windows })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for value in &self.reserved {
      writer.write_u32::<T>(*value)?;
    }

    writer.write_u32::<T>(self.windows.len() as u32)?;

    for window in &self.windows {
      window.write::<T>(writer)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::io::Write;
  use xrf_chunk::InMemoryChunkDataSource;

  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use super::OgfSwiDataChunk;
  use crate::data::ogf::ogf_slide_window::OgfSlideWindow;

  fn write_then_read(name: &str, chunk: &OgfSwiDataChunk) -> XrfResult<OgfSwiDataChunk> {
    let filename: String = build_relative_test_sample_file_path(file!(), name);
    let mut writer: ChunkWriter = ChunkWriter::new();

    chunk.write::<XRayByteOrder>(&mut writer)?;

    let contents: Vec<u8> = writer.flush_chunk_into_buffer::<XRayByteOrder>(OgfSwiDataChunk::CHUNK_ID)?;
    let mut file = overwrite_generated_test_resource_as_file(&filename)?;

    file.write_all(&contents)?;
    file.flush()?;

    let slice: FileSlice = open_generated_test_resource_as_slice(&filename)?;
    let mut reader: ChunkReader = ChunkReader::from_slice(slice)?
      .read_children()?
      .into_iter()
      .next()
      .expect("expect the written chunk to be present");

    OgfSwiDataChunk::read::<XRayByteOrder, _>(&mut reader)
  }

  #[test]
  fn round_trips_windows_and_reserved_words() -> XrfResult {
    let chunk: OgfSwiDataChunk = OgfSwiDataChunk {
      // Non zero so a writer that dropped them would be caught.
      reserved: [1, 2, 3, 4],
      windows: vec![
        OgfSlideWindow {
          offset: 0,
          num_tris: 512,
          num_verts: 300,
        },
        OgfSlideWindow {
          offset: 1536,
          num_tris: 128,
          num_verts: 90,
        },
      ],
    };

    let read: OgfSwiDataChunk = write_then_read("windows.chunk", &chunk)?;

    assert_eq!(read.reserved, chunk.reserved);
    assert_eq!(read.windows, chunk.windows);

    Ok(())
  }

  #[test]
  fn round_trips_empty_window_list() -> XrfResult {
    let chunk: OgfSwiDataChunk = OgfSwiDataChunk {
      reserved: [0; 4],
      windows: vec![],
    };

    let read: OgfSwiDataChunk = write_then_read("empty.chunk", &chunk)?;

    assert!(read.windows.is_empty());

    Ok(())
  }

  #[test]
  fn each_window_is_eight_bytes() -> XrfResult {
    // The engine reads the payload as a flat `count * sizeof(FSlideWindow)` block, so a record that
    // is not exactly 8 bytes would silently desynchronise every following one.
    let mut writer: ChunkWriter = ChunkWriter::new();

    OgfSlideWindow {
      offset: 7,
      num_tris: 8,
      num_verts: 9,
    }
    .write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.flush_raw_into_buffer()?.len(), 8);

    Ok(())
  }

  #[test]
  fn rejects_window_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    // Four reserved words, then a count no payload can satisfy.
    let mut reader: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::from_bytes(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255])?;

    let error: String = OgfSwiDataChunk::read::<XRayByteOrder, _>(&mut reader)
      .expect_err("expect the declared window count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("ogf slide windows declares 4294967295 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}

use xrf_error::{XrfError, XrfResult};

use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_data_source::ChunkDataSource;

/// Finding a chunk among siblings by the id it stores, which [`ChunkReader::read_child_by_index`] does not: that
/// selects an ordinal position.
impl<T: ChunkDataSource> ChunkReader<T> {
  /// The first chunk storing `id`, or `None` when no sibling does.
  pub fn find_optional_by_id(chunks: &[Self], id: u32) -> Option<Self> {
    chunks.iter().find(|chunk| chunk.id == id).cloned()
  }

  /// The first chunk storing `id`.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when no sibling stores it.
  pub fn find_required_by_id(chunks: &[Self], id: u32) -> XrfResult<Self> {
    Self::find_optional_by_id(chunks, id)
      .ok_or_else(|| XrfError::new_not_found_error(format!("Chunk with ID {id} was not found")))
  }

  /// The chunk storing the first of `ids` any sibling stores, with the id that matched, or `None` when none does.
  ///
  /// `ids` is a preference order: an earlier id wins even when a sibling storing a later one comes first.
  pub fn find_one_of_optional_by_id(chunks: &[Self], ids: &[u32]) -> Option<(u32, Self)> {
    ids
      .iter()
      .find_map(|&id| Self::find_optional_by_id(chunks, id).map(|chunk| (id, chunk)))
  }

  /// The chunk storing the first of `ids` any sibling stores, with the id that matched.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when no sibling stores any of them.
  pub fn find_one_of_required_by_id(chunks: &[Self], ids: &[u32]) -> XrfResult<(u32, Self)> {
    Self::find_one_of_optional_by_id(chunks, ids).ok_or_else(|| {
      XrfError::new_not_found_error(format!(
        "Chunk with one of IDs {} was not found",
        ids.iter().map(u32::to_string).collect::<Vec<String>>().join(",")
      ))
    })
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfResult;

  use crate::reader::chunk_reader::ChunkReader;
  use crate::source::chunk_memory_source::InMemoryChunkDataSource;

  /// Siblings storing ids 3, then 5, then 3 again, each with a one-byte payload naming its place.
  fn siblings() -> XrfResult<Vec<ChunkReader<InMemoryChunkDataSource>>> {
    let bytes: Vec<u8> = [(3u32, 0u8), (5, 1), (3, 2)]
      .iter()
      .flat_map(|&(id, place)| [&id.to_le_bytes()[..], &1u32.to_le_bytes(), &[place]].concat())
      .collect();

    ChunkReader::from_vec(bytes)?.read_children()
  }

  #[test]
  fn finds_the_first_sibling_storing_an_id() -> XrfResult {
    let mut found: ChunkReader<InMemoryChunkDataSource> = ChunkReader::find_required_by_id(&siblings()?, 3)?;

    assert_eq!(found.read_remaining()?, [0], "the first of the two siblings storing it");
    assert!(ChunkReader::find_optional_by_id(&siblings()?, 4).is_none());
    assert_eq!(
      ChunkReader::find_required_by_id(&siblings()?, 4)
        .expect_err("no sibling stores 4")
        .to_string(),
      "Not found error: Chunk with ID 4 was not found"
    );

    Ok(())
  }

  #[test]
  fn prefers_an_earlier_id_over_an_earlier_sibling() -> XrfResult {
    let (id, mut found) = ChunkReader::find_one_of_required_by_id(&siblings()?, &[4, 5, 3])?;

    assert_eq!(id, 5, "4 is stored by no sibling, and 5 is preferred over 3");
    assert_eq!(found.read_remaining()?, [1]);
    assert!(ChunkReader::find_one_of_optional_by_id(&siblings()?, &[1, 2]).is_none());
    assert_eq!(
      ChunkReader::find_one_of_required_by_id(&siblings()?, &[1, 2])
        .expect_err("no sibling stores either")
        .to_string(),
      "Not found error: Chunk with one of IDs 1,2 was not found"
    );

    Ok(())
  }
}

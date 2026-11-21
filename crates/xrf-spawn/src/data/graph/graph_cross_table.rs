use std::fs::File;
use std::io::Write;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xrf_chunk::{
  ChunkDataSource, ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkSizePackedIterator, ChunkWriter,
  InMemoryChunkDataSource, XRayRead, XRayWrite,
};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{assert_length, narrow_format_size};

use crate::data::graph::graph_cross_table_cell::GraphCrossTableCell;

/// One level's cross table, `CGameLevelCrossTable`: a header naming the level and the game graph it was built against,
/// then a cell for each of the level's AI nodes, in node order.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphCrossTable {
  pub version: u32,
  /// The level's AI node count, which the cells match.
  pub nodes_count: u32,
  /// The level's game vertex count.
  pub vertices_count: u32,
  pub level_guid: Uuid,
  pub game_guid: Uuid,
  #[serde(skip)] // A cell a node: millions on a level, no use in JSON.
  pub cells: Vec<GraphCrossTableCell>,
}

impl GraphCrossTable {
  /// The header's bytes: version, both counts and both GUIDs.
  pub const HEADER_SIZE: u64 = 4 + 4 + 4 + 16 + 16;

  /// Import cross tables from a separate gct chunk file; an empty file, as a graph of no level exports, holds none.
  pub fn import_list(file: &mut File) -> XrfResult<Vec<Self>> {
    let mut cross_tables: Vec<Self> = Vec::new();

    if file.metadata()?.len() == 0 {
      return Ok(cross_tables);
    }

    for cross_table_reader in ChunkSizePackedIterator::from_current(&mut ChunkReader::from_file(file.try_clone()?)?) {
      let mut cross_table_reader: ChunkReader<InMemoryChunkDataSource> = cross_table_reader?;

      cross_tables.push(cross_table_reader.read_xr()?);
      cross_table_reader.assert_read("Expect cross table chunk to be ended")?;
    }

    Ok(cross_tables)
  }

  /// Export cross-tables as a separate gct chunk file's bytes.
  pub fn export_list(cross_tables: &[Self], file: &mut dyn Write) -> XrfResult {
    let mut cross_tables_writer: ChunkWriter = ChunkWriter::new();

    for cross_table in cross_tables {
      let mut cross_table_writer: ChunkWriter = ChunkWriter::new();
      cross_table_writer.write_xr(cross_table)?;

      cross_tables_writer.write_u32(narrow_format_size(
        cross_table_writer.bytes_written() + 4,
        "graph cross table",
      )?)?;
      cross_tables_writer.write_all(&cross_table_writer.flush_raw_into_buffer()?)?;
    }

    cross_tables_writer.flush_raw_into(file)?;

    Ok(())
  }
}

impl ChunkReadWriteList for GraphCrossTable {
  /// Read cross tables list data from the chunk.
  fn read_list<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<Self>> {
    let mut cross_tables: Vec<Self> = Vec::new();

    for cross_table_reader in ChunkSizePackedIterator::from_current(reader) {
      let mut cross_table_reader: ChunkReader<D> = cross_table_reader?;

      cross_tables.push(cross_table_reader.read_xr()?);
      cross_table_reader.assert_read("Expect cross table chunk to be ended")?;
    }

    Ok(cross_tables)
  }

  /// Write cross tables list data into the writer.
  fn write_list(writer: &mut ChunkWriter, cross_tables: &[Self]) -> XrfResult {
    for table in cross_tables {
      let mut table_writer: ChunkWriter = ChunkWriter::new();
      table_writer.write_xr(table)?;

      writer.write_u32(narrow_format_size(
        table_writer.bytes_written() + 4,
        "graph cross table",
      )?)?;
      writer.write_all(&table_writer.buffer)?;
    }

    Ok(())
  }
}

impl ChunkReadWrite for GraphCrossTable {
  /// Read a cross table, refusing one whose cells are not one a node.
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let version: u32 = reader.read_u32()?;
    let nodes_count: u32 = reader.read_u32()?;
    let vertices_count: u32 = reader.read_u32()?;
    let level_guid: Uuid = Uuid::from_u128(reader.read_u128()?);
    let game_guid: Uuid = Uuid::from_u128(reader.read_u128()?);
    let remaining: u64 = reader.read_bytes_remain();

    if remaining != u64::from(nodes_count) * GraphCrossTableCell::SIZE {
      return Err(XrfError::new_parsing_error(format!(
        "Cross table of level {level_guid} declares {nodes_count} nodes but holds {remaining} bytes of cells"
      )));
    }

    // Millions of cells on a level, decoded in one pass over their bytes rather than one read per field; the check
    // above leaves no partial cell.
    let cells: Vec<GraphCrossTableCell> = reader.read_remaining_with(|bytes| {
      bytes
        .as_chunks::<{ GraphCrossTableCell::SIZE as usize }>()
        .0
        .iter()
        .map(GraphCrossTableCell::decode)
        .collect()
    })?;

    Ok(Self {
      version,
      nodes_count,
      vertices_count,
      level_guid,
      game_guid,
      cells,
    })
  }

  /// Write a cross table, refusing one whose cells are not one a node.
  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    assert_length(
      &self.cells,
      self.nodes_count as usize,
      "Expect a cross table cell for each node",
    )?;

    writer.write_u32(self.version)?;
    writer.write_u32(self.nodes_count)?;
    writer.write_u32(self.vertices_count)?;
    writer.write_u128(self.level_guid.as_u128())?;
    writer.write_u128(self.game_guid.as_u128())?;

    let mut cells: Vec<u8> = Vec::with_capacity(self.cells.len() * GraphCrossTableCell::SIZE as usize);

    for cell in &self.cells {
      cells.extend_from_slice(&cell.encode());
    }

    writer.write_all(&cells)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write, read_to_string};
  use std::path::Path;

  use serde_json::to_string_pretty;
  use uuid::{Uuid, uuid};
  use xrf_chunk::{ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter, InMemoryChunkDataSource};
  use xrf_error::{MapXrfErr, XrfResult};
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::data::graph::graph_cross_table::GraphCrossTable;
  use crate::data::graph::graph_cross_table_cell::GraphCrossTableCell;

  fn table(level_guid: Uuid, game_guid: Uuid, nodes_count: u32, vertices_count: u32) -> GraphCrossTable {
    GraphCrossTable {
      version: 16,
      nodes_count,
      vertices_count,
      level_guid,
      game_guid,
      cells: (0..nodes_count)
        .map(|node| GraphCrossTableCell {
          game_vertex_id: (node % 7) as u16,
          distance: node as f32 * 0.7,
        })
        .collect(),
    }
  }

  #[test]
  fn test_read_write_list() -> XrfResult {
    let filename: String = String::from("read_write_list.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: Vec<GraphCrossTable> = vec![
      table(
        uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
        uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
        5,
        35,
      ),
      table(
        uuid!("cce55023-10b1-426f-9247-bb680e5fe0b7"),
        uuid!("dde55023-10b1-426f-9247-bb680e5fe0b7"),
        3,
        345,
      ),
      table(
        uuid!("aa125023-10b1-426f-9247-bb680e5fe0b7"),
        uuid!("bbe55023-10b1-426f-9247-bb680e5fe0b7"),
        0,
        345,
      ),
    ];

    GraphCrossTable::write_list(&mut writer, &original)?;

    assert_eq!(writer.bytes_written(), 3 * (4 + 44) + 8 * 6);

    writer.flush_chunk_into(
      &mut GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), &filename))?,
      0,
    )?;

    let file: FileSlice = GeneratedResources::open_slice(&GeneratedResources::sample_name(file!(), &filename))?;
    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(GraphCrossTable::read_list(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let original: GraphCrossTable = table(
      uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
      uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
      11,
      4000,
    );

    original.write(&mut writer)?;

    assert_eq!(writer.bytes_written(), 44 + 11 * 6);
    assert_eq!(
      GraphCrossTable::read(&mut ChunkReader::from_bytes(&writer.buffer)?)?,
      original
    );

    Ok(())
  }

  #[test]
  fn refuses_to_write_cells_other_than_one_a_node() {
    let mut cross_table: GraphCrossTable = table(Uuid::nil(), Uuid::nil(), 4, 1);

    cross_table.nodes_count = 5;

    let error: String = cross_table
      .write(&mut ChunkWriter::new())
      .expect_err("expect four cells for five nodes to be refused")
      .to_string();

    assert!(
      error.contains("cross table cell for each node"),
      "Unexpected error: {error}"
    );
  }

  #[test]
  fn refuses_to_read_cells_other_than_one_a_node() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    table(Uuid::nil(), Uuid::nil(), 4, 1).write(&mut writer)?;

    // The node count raised from 4 to 5 over the same four cells.
    writer.buffer[4] = 5;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;
    let error: String = GraphCrossTable::read(&mut reader)
      .expect_err("expect five nodes over four cells to be refused")
      .to_string();

    assert!(
      error.contains("declares 5 nodes but holds 24 bytes"),
      "Unexpected error: {error}"
    );

    Ok(())
  }

  #[test]
  fn refuses_to_read_a_partial_cell() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    table(Uuid::nil(), Uuid::nil(), 4, 1).write(&mut writer)?;
    writer.buffer.extend_from_slice(&[0, 0, 0]);

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;
    let error: String = GraphCrossTable::read(&mut reader)
      .expect_err("expect four nodes over four and a half cells to be refused")
      .to_string();

    assert!(
      error.contains("declares 4 nodes but holds 27 bytes"),
      "Unexpected error: {error}"
    );

    Ok(())
  }

  // An unreached cell's distance is `u32::MAX` as a float, a NaN that must come back bit for bit.
  fn assert_table_agrees_with_cells() -> XrfResult {
    let mut original: GraphCrossTable = table(Uuid::nil(), Uuid::nil(), 5, 300);

    original.cells[1].game_vertex_id = 0x1234;
    original.cells[3].distance = f32::from_bits(u32::MAX);
    original.cells[4].distance = -0.0;

    let mut table_writer: ChunkWriter = ChunkWriter::new();
    let mut cells_writer: ChunkWriter = ChunkWriter::new();

    original.write(&mut table_writer)?;

    for cell in &original.cells {
      cells_writer.write_xr(cell)?;
    }

    assert_eq!(
      &table_writer.buffer[GraphCrossTable::HEADER_SIZE as usize..],
      cells_writer.buffer.as_slice()
    );

    let read: GraphCrossTable = GraphCrossTable::read(&mut ChunkReader::from_bytes(&table_writer.buffer)?)?;
    let mut cells_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&cells_writer.buffer)?;
    let bits = |cell: &GraphCrossTableCell| (cell.game_vertex_id, cell.distance.to_bits());

    assert_eq!(read.cells.len(), original.cells.len());

    for (cell, expected) in read.cells.iter().zip(&original.cells) {
      let one: GraphCrossTableCell = cells_reader.read_xr()?;

      assert_eq!(bits(cell), bits(&one));
      assert_eq!(bits(cell), bits(expected));
    }

    cells_reader.assert_read("Expect every cell to be read")
  }

  #[test]
  fn reads_and_writes_cells_as_each_cell_does() -> XrfResult {
    assert_table_agrees_with_cells()
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let config_path: &Path = &GeneratedResources::sample_path(file!(), "import_export.gct");
    let mut file: File = GeneratedResources::overwrite(config_path.to_str().expect("Valid path"))?;

    let original: Vec<GraphCrossTable> = vec![
      table(
        uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
        uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
        23,
        62,
      ),
      table(
        uuid!("cce55023-10b1-426f-9247-bb680e5fe0b7"),
        uuid!("dde55023-10b1-426f-9247-bb680e5fe0b7"),
        22,
        52,
      ),
    ];

    GraphCrossTable::export_list(&original, &mut file)?;

    assert_eq!(
      GraphCrossTable::import_list(&mut GeneratedResources::open(config_path.to_str().unwrap())?,)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: GraphCrossTable = table(
      uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
      uuid!("89e55024-10b1-426f-9247-bb680e5fe0b8"),
      0,
      26324,
    );

    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original).map_serde_err()?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(
      original,
      serde_json::from_str::<GraphCrossTable>(&serialized).map_serde_err()?
    );

    Ok(())
  }
}

//! How every packer cuts a drawn run of indices into clusters.

use crate::data::visual::geometry::visual_clusters::VisualClusters;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_cluster_table::VisualClusterTable;

/// A strip of quads along x, two triangles a quad, their corners at heights of nought and one.
fn new_strip(quads: u32) -> (Vec<u32>, Vec<[f32; 3]>) {
  let positions: Vec<[f32; 3]> = (0..=quads)
    .flat_map(|quad| [[quad as f32, 0.0, 0.0], [quad as f32, 1.0, 0.0]])
    .collect();
  let indices: Vec<u32> = (0..quads)
    .flat_map(|quad| {
      let at: u32 = quad * 2;

      [at, at + 1, at + 2, at + 1, at + 3, at + 2]
    })
    .collect();

  (indices, positions)
}

/// The table's words and floats, read back out of the buffer it was written into.
fn new_read(table: &VisualClusterTable) -> (Vec<u32>, Vec<f32>) {
  let mut builder: VisualBufferBuilder = VisualBufferBuilder::new();
  let clusters: VisualClusters = table.write_into(&mut builder);
  let buffer: Vec<u8> = builder.into_buffer();
  let read = |offset: u32, length: u32| {
    buffer[offset as usize..(offset + length) as usize]
      .as_chunks::<4>()
      .0
      .to_vec()
  };

  (
    read(clusters.ranges.byte_offset, clusters.ranges.byte_length)
      .iter()
      .map(|bytes| u32::from_le_bytes(*bytes))
      .collect(),
    read(clusters.spheres.byte_offset, clusters.spheres.byte_length)
      .iter()
      .map(|bytes| f32::from_le_bytes(*bytes))
      .collect(),
  )
}

#[test]
fn cuts_a_run_into_clusters_of_consecutive_triangles_the_last_holding_what_is_left() {
  let (indices, positions) = new_strip(150);
  let mut table: VisualClusterTable = VisualClusterTable::default();

  table.push_run(&indices, &positions, 0, indices.len() as u32, 7);

  let (ranges, _) = new_read(&table);

  // 300 triangles: 128, 128, then 44, each naming its drawable.
  assert_eq!(ranges, vec![0, 128, 7, 0, 384, 128, 7, 0, 768, 44, 7, 0]);
}

#[test]
fn bounds_each_cluster_by_its_own_vertices() {
  let (indices, positions) = new_strip(150);
  let mut table: VisualClusterTable = VisualClusterTable::default();

  table.push_run(
    &indices,
    &positions,
    0,
    indices.len() as u32,
    VisualClusters::NO_DRAWABLE,
  );

  let (_, spheres) = new_read(&table);

  // The first cluster's 64 quads span x 0 to 64 and y 0 to 1: centred on their box, reaching its corners.
  assert_eq!(&spheres[0..3], &[32.0, 0.5, 0.0]);
  assert!((spheres[3] - (32.0_f32.powi(2) + 0.25).sqrt()).abs() < 1e-4);
  // The last holds quads 128 to 149.
  assert_eq!(&spheres[8..11], &[139.0, 0.5, 0.0]);
}

// Runs cut one after another share the table and never a cluster.
#[test]
fn numbers_the_clusters_of_later_runs_after_earlier_ones() {
  let (indices, positions) = new_strip(4);
  let mut table: VisualClusterTable = VisualClusterTable::default();

  table.push_run(&indices, &positions, 0, 12, 1);
  table.push_run(&indices, &positions, 12, 12, 2);

  assert_eq!(new_read(&table).0, vec![0, 4, 1, 0, 12, 4, 2, 0]);

  table.push_run(&indices, &positions, 24, 0, 3);

  assert_eq!(table.get_count(), 2, "an empty run has none");
}

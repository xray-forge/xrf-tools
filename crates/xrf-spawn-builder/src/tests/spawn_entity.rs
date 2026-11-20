use crate::entity::spawn_artefact_buffer::SpawnArtefactBuffer;

#[test]
fn packs_kept_nodes_and_reads_the_previous_zone_past_the_end() {
  let mut buffer: SpawnArtefactBuffer = SpawnArtefactBuffer::default();

  assert_eq!(buffer.fill(&[1, 2, 3, 4], |node| node != 2), 3);
  // Kept nodes packed to the front; the last push stays behind them.
  assert_eq!(buffer.read(4), [1, 3, 4, 4]);

  // A zone wanting more points than it kept takes the earlier zone's nodes after its own.
  assert_eq!(buffer.fill(&[9], |_| true), 1);
  assert_eq!(buffer.read(3), [9, 3, 4]);
}

#[test]
fn drops_old_nodes_when_a_flood_outgrows_the_reserve() {
  let mut buffer: SpawnArtefactBuffer = SpawnArtefactBuffer::default();
  let flooded: Vec<u32> = (1..=1100).collect();

  buffer.fill(&[7; 1024], |_| true);

  assert_eq!(buffer.fill(&flooded, |_| true), 1100);
  // Grown by half to 1536: past the flood, memory reads as never written, then past capacity too.
  assert_eq!(buffer.read(1600)[1099..1102], [1100, 0, 0]);
  assert_eq!(buffer.read(1600)[1599], 0);
}

use crate::scene::static_scene::static_row::StaticRow;

/// The fields `records.wgsl`'s `row_band`, `row_bands`, `row_windows` and `row_group` read back out of a band word.
fn unpack(word: u32) -> [u32; 4] {
  [word & 15, (word >> 4) & 15, (word >> 8) & 0xFFFF, word >> 24]
}

// Zaton's helicopter wreck is a progressive mesh of 1421 slide windows: packed eight bits wide they spilled into the
// visibility group's byte, which hid its rows as a released object's.
#[test]
fn static_row_keeps_a_mesh_of_many_windows_out_of_the_group_byte() {
  assert_eq!(unpack(StaticRow::pack_band(3, 4, 1421)), [3, 4, 1421, 0]);
  assert_eq!(
    unpack(StaticRow::pack_band(0, 1, 1) | StaticRow::pack_group(5)),
    [0, 1, 1, 5]
  );
}

#[test]
fn static_row_grades_a_mesh_of_more_windows_than_a_band_word_counts_as_its_most() {
  assert_eq!(
    unpack(StaticRow::pack_band(1, 4, 100_000)),
    [1, 4, StaticRow::MAX_WINDOWS, 0]
  );
}

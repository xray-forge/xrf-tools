//! Holds the buffer layout invariant: every section starts where a typed array view can read it.

use crate::data::visual::geometry::visual_section::VisualSection;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;

#[test]
fn reports_the_range_each_section_occupies() {
  let mut builder: VisualBufferBuilder = VisualBufferBuilder::new();

  let positions: VisualSection = builder.push_f32_section(&[1.0, 2.0, 3.0]);
  let indices: VisualSection = builder.push_u16_section(&[0, 1, 2]);

  assert_eq!(
    positions,
    VisualSection {
      byte_offset: 0,
      byte_length: 12
    }
  );
  assert_eq!(
    indices,
    VisualSection {
      byte_offset: 12,
      byte_length: 6
    }
  );
  assert_eq!(builder.length(), 18);
}

#[test]
fn aligns_every_section_to_four_bytes() {
  // A three index section is six bytes, so the next section can only start aligned if the builder pads.
  // A `Float32Array` view throws on an offset that is not a multiple of four, so this is the invariant
  // the whole layout rests on rather than a tidiness preference.
  let mut builder: VisualBufferBuilder = VisualBufferBuilder::new();

  builder.push_u16_section(&[0, 1, 2]);

  let positions: VisualSection = builder.push_f32_section(&[1.0]);

  builder.push_u16_section(&[7]);

  let trailing: VisualSection = builder.push_f32_section(&[2.0]);

  assert_eq!(
    positions.byte_offset, 8,
    "expect the six byte section to be padded to eight"
  );
  assert_eq!(
    trailing.byte_offset, 16,
    "expect the two byte section to be padded to four"
  );
  assert_eq!(positions.byte_offset % 4, 0);
  assert_eq!(trailing.byte_offset % 4, 0);
}

#[test]
fn writes_values_little_endian() {
  // Typed array views read in host order, and every supported target is little endian, so the packer
  // must not follow the big endian convention the X-Ray chunk readers use for some formats.
  let mut builder: VisualBufferBuilder = VisualBufferBuilder::new();

  builder.push_u16_section(&[0x0102]);

  assert_eq!(builder.into_buffer(), vec![0x02, 0x01]);
}

#[test]
fn produces_an_empty_buffer_for_an_empty_section() {
  let mut builder: VisualBufferBuilder = VisualBufferBuilder::new();

  let section: VisualSection = builder.push_f32_section(&[]);

  assert_eq!(section.byte_length, 0);
  assert!(builder.into_buffer().is_empty());
}

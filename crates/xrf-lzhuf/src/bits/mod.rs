//! Big-endian bit runs, read out of a stream and written into one, most significant bit of each byte first.

pub(crate) mod bit_reader;
pub(crate) mod bit_writer;

pub(crate) use crate::bits::bit_reader::BitReader;
pub(crate) use crate::bits::bit_writer::BitWriter;

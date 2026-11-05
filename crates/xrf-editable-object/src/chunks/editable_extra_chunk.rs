/// A chunk no field of its container takes, kept as the bytes it was stored as and written back where the file stored
/// it: an older layout the editor still reads, a duplicate it ignores, or one nothing here knows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableExtraChunk {
  pub id: u32,
  /// The payload, without its header.
  pub bytes: Vec<u8>,
}

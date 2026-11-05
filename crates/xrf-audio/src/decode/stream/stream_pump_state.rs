/// Where a pump stands after a fill.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamPumpState {
  /// The ring is full; more fits once the voice has read some.
  Full,
  /// Nothing more will come: the sound ended, the decoder failed, or the voice let go of the stream.
  Done,
}

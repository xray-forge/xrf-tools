use std::sync::Arc;

use rtrb::Producer;
use xrf_error::{XrfError, XrfResult};

use crate::decode::sound_decoder::SoundDecoder;
use crate::decode::sound_format::SoundFormat;
use crate::decode::stream::stream_shared::StreamShared;

/// A stream's decoding: it opens the sound and seeks on its first fill, then decodes ahead into the ring its voice
/// reads until the ring is full, the sound ends without looping, or the decoder fails. A stream's own thread runs it;
/// an offline session runs it inline before each block, so a render with streams comes out the same every time.
pub struct StreamPump {
  opening: Option<(Arc<[u8]>, u64)>,
  decoder: Option<SoundDecoder>,
  producer: Producer<f32>,
  shared: Arc<StreamShared>,
  /// Decoded samples not yet in the ring, from `offset` on.
  pending: Vec<f32>,
  offset: usize,
}

/// Where a pump stands after a fill.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamPumpState {
  /// The ring is full; more fits once the voice has read some.
  Full,
  /// Nothing more will come: the sound ended, the decoder failed, or the voice let go of the stream.
  Done,
}

impl StreamPump {
  pub(crate) fn new(bytes: Arc<[u8]>, frame: u64, producer: Producer<f32>, shared: Arc<StreamShared>) -> Self {
    Self {
      opening: Some((bytes, frame)),
      decoder: None,
      producer,
      shared,
      pending: Vec::new(),
      offset: 0,
    }
  }

  /// Decodes into the ring until it is full or nothing more will come.
  pub fn fill(&mut self) -> StreamPumpState {
    if let Some((bytes, frame)) = self.opening.take() {
      match Self::open(bytes, frame, self.shared.format) {
        Ok(decoder) => self.decoder = Some(decoder),
        Err(error) => {
          self.shared.fail(error.to_string());
          return StreamPumpState::Done;
        }
      }
    }

    let Some(decoder) = self.decoder.as_mut() else {
      return StreamPumpState::Done;
    };

    loop {
      if self.producer.is_abandoned() {
        return StreamPumpState::Done;
      }

      if self.offset == self.pending.len() {
        // The decoder replaces what `pending` held, emptying it at the end.
        self.offset = 0;

        match decoder.decode_next(&mut self.pending) {
          Ok(true) => {}
          Ok(false) if self.shared.is_looped() && decoder.get_frames_count() > 0 => {
            if let Err(error) = decoder.seek(0) {
              self.shared.fail(error.to_string());
              return StreamPumpState::Done;
            }
          }
          Ok(false) => {
            self.shared.end();
            return StreamPumpState::Done;
          }
          Err(error) => {
            self.shared.fail(error.to_string());
            return StreamPumpState::Done;
          }
        }

        continue;
      }

      let count: usize = self.producer.slots().min(self.pending.len() - self.offset);

      if count == 0 {
        return StreamPumpState::Full;
      }

      if let Ok(chunk) = self.producer.write_chunk_uninit(count) {
        self.offset += chunk.fill_from_iter(self.pending[self.offset..self.offset + count].iter().copied());
      }
    }
  }

  fn open(bytes: Arc<[u8]>, frame: u64, format: SoundFormat) -> XrfResult<SoundDecoder> {
    let mut decoder: SoundDecoder = SoundDecoder::open(bytes)?;

    if decoder.get_format() != format {
      return Err(XrfError::new_read_error(format!(
        "The sound is {:?}, not the {format:?} it was started as",
        decoder.get_format()
      )));
    }

    decoder.seek(frame)?;

    Ok(decoder)
  }
}

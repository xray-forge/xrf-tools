use crate::decode::sound_channels::SoundChannels;
use crate::decode::stream::stream_reader::StreamReader;
use crate::voice::cursor::stereo_frame::StereoFrame;

/// A voice's place in a stream: the four frames around the playing one, taken from the ring as the voice moves on.
#[derive(Debug)]
pub(crate) struct StreamCursor {
  reader: StreamReader,
  /// The frames before, at and after the playing one: `[previous, current, next, after]`.
  window: [StereoFrame; 4],
  /// How far between `current` and `next` the next output frame is read.
  fraction: f64,
  /// Frames of the window filled since the voice started, up to the three from `current` on.
  filled: usize,
  /// Frames taken from the stream, counted from where it started.
  taken: u64,
  /// Silent frames moved in after the stream drained, so its last frames pass through the window.
  tail: u8,
}

impl StreamCursor {
  /// Silent frames it takes for the last of the stream to pass through `current`.
  const TAIL: u8 = 3;

  pub(crate) fn new(reader: StreamReader) -> Self {
    Self {
      reader,
      window: [StereoFrame::SILENCE; 4],
      fraction: 0.0,
      filled: 0,
      taken: 0,
      tail: 0,
    }
  }

  pub(crate) fn get_sample_rate(&self) -> u32 {
    self.reader.get_sample_rate()
  }

  pub(crate) fn get_channels(&self) -> SoundChannels {
    self.reader.get_channels()
  }

  /// The source frame playing now, wrapped into the sound for a looping stream.
  pub(crate) fn get_frame(&self) -> u64 {
    let frames_count: u64 = self.reader.get_frames_count().max(1);
    let played: u64 = self.taken.saturating_sub(self.filled.min(3) as u64);

    (self.reader.get_start_frame() + played) % frames_count
  }

  /// Writes the next frames, `step` source frames apart, into `out` in the stream's own channels. Frames the decoder
  /// thread has not produced yet are left as they were and the voice waits on them. Answers `false` once the stream
  /// has drained and played out.
  pub(crate) fn render(&mut self, out: &mut [f32], step: f64) -> bool {
    let channels: usize = self.get_channels().get_count();

    if !self.fill() {
      return !self.is_played_out();
    }

    for at in 0..out.len() / channels {
      let [previous, current, next, after] = self.window;
      let t: f32 = self.fraction as f32;

      // A mono stream is interpolated in its one channel.
      if channels == 1 {
        out[at] = StereoFrame::hermite_sample(previous.left, current.left, next.left, after.left, t);
      } else {
        StereoFrame::hermite(previous, current, next, after, t).write(out, channels, at);
      }

      if !self.step(step) {
        return !self.is_played_out();
      }
    }

    true
  }

  /// Moves on `frames` output frames, `step` source frames apart, rendering nothing: a voice no one hears. Frames the
  /// decoder thread has not produced yet hold it where it is. Answers `false` once the stream has drained and played
  /// out.
  pub(crate) fn skip(&mut self, frames: usize, step: f64) -> bool {
    if !self.fill() {
      return !self.is_played_out();
    }

    for _ in 0..frames {
      if !self.step(step) {
        return !self.is_played_out();
      }
    }

    true
  }

  /// Moves `step` source frames on, answering whether the frames to move to were there.
  fn step(&mut self, step: f64) -> bool {
    self.fraction += step;

    while self.fraction >= 1.0 {
      if !self.advance() {
        return false;
      }

      self.fraction -= 1.0;
    }

    true
  }

  /// Fills the window's frames from `current` on, answering whether it is full.
  fn fill(&mut self) -> bool {
    while self.filled < 3 {
      let Some(frame) = self.take() else {
        return false;
      };

      self.window[1 + self.filled] = frame;
      self.filled += 1;
    }

    true
  }

  /// Moves the window one frame on, answering whether there was a frame to move to.
  fn advance(&mut self) -> bool {
    let Some(frame) = self.take() else {
      return false;
    };

    self.window.rotate_left(1);
    self.window[3] = frame;

    true
  }

  /// The stream's next frame, silence once it has drained until its last frames have played, or `None` when there is
  /// nothing to take yet or any more.
  fn take(&mut self) -> Option<StereoFrame> {
    if let Some(frame) = self.reader.pop_frame() {
      self.taken += 1;
      return Some(frame);
    }

    if self.reader.is_drained() && self.tail < Self::TAIL {
      self.tail += 1;
      return Some(StereoFrame::SILENCE);
    }

    None
  }

  fn is_played_out(&self) -> bool {
    self.reader.is_drained() && self.tail >= Self::TAIL
  }
}

use crate::control::audio_refusal::AudioRefusal;
use crate::voice::audio_voice_id::AudioVoiceId;

/// Which of the mixer's slots hold a voice, and how many voices each has held, kept by the control side alone.
///
/// Capped voices hold at most a set number of slots together, so the rest of the pool stays free for the others.
#[derive(Debug)]
pub(crate) struct VoiceSlots {
  /// Each slot's latest generation; the first voice in a slot is generation 1, so a status never written reads 0.
  generations: Box<[u32]>,
  is_used: Box<[bool]>,
  /// Whether the voice a slot holds counts against the cap.
  is_capped: Box<[bool]>,
  free: Vec<u32>,
  capped_most: usize,
  capped_playing: usize,
}

impl VoiceSlots {
  /// `count` slots, of which capped voices hold at most `capped_most`.
  pub(crate) fn new(count: usize, capped_most: usize) -> Self {
    let count: u32 = u32::try_from(count).expect("a mixer has fewer voices than u32 counts");

    Self {
      generations: vec![0; count as usize].into_boxed_slice(),
      is_used: vec![false; count as usize].into_boxed_slice(),
      is_capped: vec![false; count as usize].into_boxed_slice(),
      // Taken from the end, so slots fill from the first.
      free: (0..count).rev().collect(),
      capped_most,
      capped_playing: 0,
    }
  }

  /// A free slot's next voice, counted against the cap when `is_capped`.
  ///
  /// # Errors
  ///
  /// Refuses with [`AudioRefusal::VoicesExhausted`] when every slot plays, and with [`AudioRefusal::VoicesCapped`] when
  /// a capped voice finds the cap held.
  pub(crate) fn take(&mut self, is_capped: bool) -> Result<AudioVoiceId, AudioRefusal> {
    if is_capped && self.capped_playing >= self.capped_most {
      return Err(AudioRefusal::VoicesCapped);
    }

    let slot: u32 = self.free.pop().ok_or(AudioRefusal::VoicesExhausted)?;
    let generation: &mut u32 = &mut self.generations[slot as usize];

    *generation = generation.wrapping_add(1).max(1);
    self.is_used[slot as usize] = true;
    self.is_capped[slot as usize] = is_capped;
    self.capped_playing += usize::from(is_capped);

    Ok(AudioVoiceId {
      slot,
      generation: *generation,
    })
  }

  /// Frees the slot of a voice the mixer handed back, or of one that never reached it.
  pub(crate) fn give_back(&mut self, id: AudioVoiceId) {
    if self.is_current(id) {
      let slot: usize = id.slot as usize;

      self.is_used[slot] = false;
      self.capped_playing -= usize::from(self.is_capped[slot]);
      self.free.push(id.slot);
    }
  }

  /// Whether `id` is the voice its slot holds now.
  pub(crate) fn is_current(&self, id: AudioVoiceId) -> bool {
    let slot: usize = id.slot as usize;

    self.is_used.get(slot) == Some(&true) && self.generations[slot] == id.generation
  }

  pub(crate) fn get_count(&self) -> usize {
    self.generations.len()
  }

  pub(crate) fn get_playing_count(&self) -> usize {
    self.generations.len() - self.free.len()
  }
}

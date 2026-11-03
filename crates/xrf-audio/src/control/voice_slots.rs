use crate::voice::audio_voice_id::AudioVoiceId;

/// Which of the mixer's slots hold a voice, and how many voices each has held, kept by the control side alone.
#[derive(Debug)]
pub(crate) struct VoiceSlots {
  /// Each slot's latest generation; the first voice in a slot is generation 1, so a status never written reads 0.
  generations: Box<[u32]>,
  is_used: Box<[bool]>,
  free: Vec<u32>,
}

impl VoiceSlots {
  pub(crate) fn new(count: usize) -> Self {
    let count: u32 = u32::try_from(count).expect("a mixer has fewer voices than u32 counts");

    Self {
      generations: vec![0; count as usize].into_boxed_slice(),
      is_used: vec![false; count as usize].into_boxed_slice(),
      // Taken from the end, so slots fill from the first.
      free: (0..count).rev().collect(),
    }
  }

  /// A free slot's next voice, or `None` when every slot plays.
  pub(crate) fn take(&mut self) -> Option<AudioVoiceId> {
    let slot: u32 = self.free.pop()?;
    let generation: &mut u32 = &mut self.generations[slot as usize];

    *generation = generation.wrapping_add(1).max(1);
    self.is_used[slot as usize] = true;

    Some(AudioVoiceId {
      slot,
      generation: *generation,
    })
  }

  /// Frees the slot of a voice the mixer handed back, or of one that never reached it.
  pub(crate) fn give_back(&mut self, id: AudioVoiceId) {
    if self.is_current(id) {
      self.is_used[id.slot as usize] = false;
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

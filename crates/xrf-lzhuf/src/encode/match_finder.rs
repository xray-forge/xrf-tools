use crate::encode::SourceMatch;
use crate::lzhuf_constants::{MATCH_LENGTH_MAX, MATCH_LENGTH_THRESHOLD, RING_BUFFER_SIZE};

/// Bytes that must agree before a match is worth coding, which is also the hash width.
const MATCH_LENGTH_MIN: usize = MATCH_LENGTH_THRESHOLD + 1;

/// Size of the hash table, as a power of two.
const HASH_BITS: u32 = 15;
const HASH_SIZE: usize = 1 << HASH_BITS;

/// Candidates examined per position before settling for the best match found so far.
const MAX_CHAIN_DEPTH: usize = 128;

/// Mask that wraps a position into the sliding window.
const WINDOW_MASK: usize = RING_BUFFER_SIZE - 1;

/// Finds the longest back-reference for a position, over hash chains of equal three-byte prefixes.
pub(crate) struct MatchFinder<'a> {
  source: &'a [u8],
  /// Latest position holding each hash, stored as position + 1 so zero means empty.
  head: Box<[u32]>,
  /// Previous position sharing a hash, by window slot, stored as position + 1.
  previous: Box<[u32]>,
}

impl<'a> MatchFinder<'a> {
  pub(crate) fn new(source: &'a [u8]) -> Self {
    Self {
      source,
      // Built through a vector rather than `Box::new([0; N])`, which would put a table of this size on
      // the stack before moving it. Test threads have room for that; a program's main thread may not.
      head: vec![0; HASH_SIZE].into_boxed_slice(),
      previous: vec![0; RING_BUFFER_SIZE].into_boxed_slice(),
    }
  }

  /// Record `position` so later positions can match against it.
  pub(crate) fn insert(&mut self, position: usize) {
    let Some(hash) = self.hash_at(position) else {
      return;
    };

    self.previous[position & WINDOW_MASK] = self.head[hash];
    self.head[hash] = (position as u32) + 1;
  }

  /// Find the longest codable match for `position`, or `None` when literals are the better deal.
  pub(crate) fn find(&self, position: usize) -> Option<SourceMatch> {
    let longest_possible: usize = MATCH_LENGTH_MAX.min(self.source.len() - position);

    if longest_possible < MATCH_LENGTH_MIN {
      return None;
    }

    let earliest: usize = position.saturating_sub(RING_BUFFER_SIZE);
    let mut best: Option<SourceMatch> = None;
    let mut best_length: usize = MATCH_LENGTH_THRESHOLD;
    let mut candidate: u32 = self.head[self.hash_at(position)?];
    let mut depth: usize = 0;

    while candidate != 0 && depth < MAX_CHAIN_DEPTH {
      let start: usize = (candidate - 1) as usize;

      // Chains run newest first, so the first candidate out of the window ends the search.
      if start < earliest {
        break;
      }

      let length: usize = self.common_prefix(start, position, longest_possible);

      if length > best_length {
        best_length = length;
        best = Some(SourceMatch {
          length,
          distance: position - start - 1,
        });

        if length == longest_possible {
          break;
        }
      }

      candidate = self.previous[start & WINDOW_MASK];
      depth += 1;
    }

    best
  }

  /// Length the bytes at `start` and `position` share, capped at `limit`.
  ///
  /// `start + offset` may reach at or past `position`, which codes an overlapping run: the decoder
  /// produces those bytes before it needs them, one at a time.
  fn common_prefix(&self, start: usize, position: usize, limit: usize) -> usize {
    (0..limit)
      .take_while(|offset| self.source[start + offset] == self.source[position + offset])
      .count()
  }

  fn hash_at(&self, position: usize) -> Option<usize> {
    let window: &[u8] = self.source.get(position..position + MATCH_LENGTH_MIN)?;
    let key: u32 = u32::from(window[0]) << 16 | u32::from(window[1]) << 8 | u32::from(window[2]);

    Some((key.wrapping_mul(0x9E37_79B1) >> (u32::BITS - HASH_BITS)) as usize)
  }
}

#[cfg(test)]
mod tests {
  use super::{MATCH_LENGTH_MIN, MatchFinder};
  use crate::lzhuf_constants::{MATCH_LENGTH_MAX, RING_BUFFER_SIZE};

  /// Record every position up to `position`, the way the encoder does as it advances.
  fn finder_primed_to(source: &[u8], position: usize) -> MatchFinder<'_> {
    let mut finder: MatchFinder = MatchFinder::new(source);

    for index in 0..position {
      finder.insert(index);
    }

    finder
  }

  #[test]
  fn finds_a_repeat_and_measures_the_distance_the_decoder_expects() {
    let source: &[u8] = b"abcdef__abcdef";
    let finder: MatchFinder = finder_primed_to(source, 8);
    let found = finder.find(8).expect("repeat of the opening bytes");

    assert_eq!(found.length, 6);
    // Distance 0 is the byte just produced, so a repeat eight bytes back is distance seven.
    assert_eq!(found.distance, 7);
  }

  #[test]
  fn ignores_repeats_too_short_to_code() {
    let source: &[u8] = b"ab_ab";
    let finder: MatchFinder = finder_primed_to(source, 3);

    assert!(finder.find(3).is_none(), "two bytes never beat the threshold");
    const { assert!(MATCH_LENGTH_MIN > 2) };
  }

  #[test]
  fn codes_a_run_as_one_overlapping_match() {
    let source: Vec<u8> = vec![b'z'; 100];
    let finder: MatchFinder = finder_primed_to(&source, 1);
    let found = finder.find(1).expect("run of identical bytes");

    // The match reaches into bytes it produces itself, which is how a run costs one command.
    assert_eq!(found.distance, 0);
    assert_eq!(found.length, MATCH_LENGTH_MAX);
  }

  /// Lay out a phrase, filler, then the same phrase again, so its repeat sits at exactly `distance`.
  fn source_repeating_at_distance(distance: usize) -> Vec<u8> {
    let phrase: &[u8] = b"marker_bytes";
    let mut source: Vec<u8> = phrase.to_vec();

    source.extend(std::iter::repeat_n(b'.', distance + 1 - phrase.len()));
    source.extend_from_slice(phrase);

    source
  }

  #[test]
  fn finds_a_repeat_at_the_furthest_reachable_distance() {
    let distance: usize = RING_BUFFER_SIZE - 1;
    let source: Vec<u8> = source_repeating_at_distance(distance);
    let position: usize = distance + 1;
    let finder: MatchFinder = finder_primed_to(&source, position);
    let found = finder.find(position).expect("repeat still inside the window");

    assert_eq!(found.distance, distance);
  }

  #[test]
  fn never_reaches_past_the_window() {
    let distance: usize = RING_BUFFER_SIZE;
    let source: Vec<u8> = source_repeating_at_distance(distance);
    let position: usize = distance + 1;
    let finder: MatchFinder = finder_primed_to(&source, position);

    assert!(finder.find(position).is_none(), "one byte too far to reference");
  }

  #[test]
  fn finds_nothing_at_the_tail_of_the_source() {
    let source: &[u8] = b"aaaaaa";
    let finder: MatchFinder = finder_primed_to(source, source.len() - 1);

    assert!(finder.find(source.len() - 1).is_none(), "one byte left to code");
  }
}

use crate::chart::light_chart::LightChart;

/// The texels of one lightmap page charts take, a bit each, row by row, and each row's longest run left free, which
/// rules out rows quickly.
pub struct LightmapPage {
  rows: Vec<[u64; Self::WORDS]>,
  longest_free: Vec<u32>,
}

impl Default for LightmapPage {
  /// An empty page.
  fn default() -> Self {
    Self {
      rows: vec![[0; Self::WORDS]; LightChart::PAGE_SIDE as usize],
      longest_free: vec![LightChart::PAGE_SIDE; LightChart::PAGE_SIDE as usize],
    }
  }
}

impl LightmapPage {
  /// The words a row of [`LightChart::PAGE_SIDE`] bits takes.
  const WORDS: usize = (LightChart::PAGE_SIDE / 64) as usize;

  /// The first place, by row and then by column, from row `from` on, a mask of `rows` spans `width` texels wide fits
  /// where no texel it takes is taken; none where it fits nowhere. Each place is tried widest span first, which rules
  /// most out soonest.
  pub fn find(&self, rows: &[Option<(u32, u32)>], width: u32, from: u32) -> Option<(u32, u32)> {
    let side: u32 = LightChart::PAGE_SIDE;
    let height: u32 = rows.len() as u32;

    if width > side || height > side {
      return None;
    }

    let mut spans: Vec<(usize, u32, u32)> = rows
      .iter()
      .enumerate()
      .filter_map(|(offset, span)| span.map(|(first, last)| (offset, first, last - first + 1)))
      .collect();

    spans.sort_by_key(|(_, _, length)| std::cmp::Reverse(*length));

    let reach: [u64; Self::WORDS] = Self::get_prefix(side - width + 1);

    for y in from..=side - height {
      if let Some((offset, _, length)) = spans.first()
        && self.longest_free[y as usize + offset] < *length
      {
        continue;
      }

      let mut fits: [u64; Self::WORDS] = reach;

      for (offset, first, length) in &spans {
        let free: [u64; Self::WORDS] = Self::get_free_runs(&self.rows[y as usize + offset], *length);

        Self::and_assign(&mut fits, &Self::shift_down(&free, *first));

        if fits.iter().all(|word| *word == 0) {
          break;
        }
      }

      if let Some(x) = Self::get_first(&fits) {
        return Some((x, y));
      }
    }

    None
  }

  /// Takes the texels of a mask of `rows` spans at `x`, `y`.
  pub fn take(&mut self, rows: &[Option<(u32, u32)>], x: u32, y: u32) {
    for (offset, span) in rows.iter().enumerate() {
      let Some((first, last)) = span else {
        continue;
      };
      let row: usize = y as usize + offset;

      for texel in x + first..=x + last {
        self.rows[row][(texel / 64) as usize] |= 1 << (texel % 64);
      }

      self.longest_free[row] = Self::get_longest_free(&self.rows[row]);
    }
  }

  /// The bits of the first `count` texels.
  fn get_prefix(count: u32) -> [u64; Self::WORDS] {
    std::array::from_fn(|word| {
      let start: u32 = word as u32 * 64;

      match count.saturating_sub(start) {
        0 => 0,
        bits if bits >= 64 => u64::MAX,
        bits => (1 << bits) - 1,
      }
    })
  }

  /// The texels of `row` from which `length` texels in a row are free.
  fn get_free_runs(row: &[u64; Self::WORDS], length: u32) -> [u64; Self::WORDS] {
    let mut runs: [u64; Self::WORDS] = row.map(|word| !word);
    let mut covered: u32 = 1;

    while covered < length {
      let step: u32 = covered.min(length - covered);
      let shifted: [u64; Self::WORDS] = Self::shift_down(&runs, step);

      Self::and_assign(&mut runs, &shifted);
      covered += step;
    }

    runs
  }

  /// Each texel takes the bit `by` texels on, a texel past the row's end counting as taken.
  fn shift_down(bits: &[u64; Self::WORDS], by: u32) -> [u64; Self::WORDS] {
    let (words, rest) = ((by / 64) as usize, by % 64);

    std::array::from_fn(|index| {
      let low: u64 = bits.get(index + words).copied().unwrap_or(0);
      let high: u64 = bits.get(index + words + 1).copied().unwrap_or(0);

      if rest == 0 {
        low
      } else {
        (low >> rest) | (high << (64 - rest))
      }
    })
  }

  fn and_assign(bits: &mut [u64; Self::WORDS], other: &[u64; Self::WORDS]) {
    for (word, other) in bits.iter_mut().zip(other) {
      *word &= other;
    }
  }

  /// The first texel set, none for none.
  fn get_first(bits: &[u64; Self::WORDS]) -> Option<u32> {
    bits
      .iter()
      .enumerate()
      .find(|(_, word)| **word != 0)
      .map(|(index, word)| index as u32 * 64 + word.trailing_zeros())
  }

  /// The longest run of free texels in `row`.
  fn get_longest_free(row: &[u64; Self::WORDS]) -> u32 {
    let (mut longest, mut current): (u32, u32) = (0, 0);

    for word in row {
      if *word == 0 {
        current += 64;
        continue;
      }

      for bit in 0..64 {
        if word & (1 << bit) == 0 {
          current += 1;
        } else {
          longest = longest.max(current);
          current = 0;
        }
      }
    }

    longest.max(current)
  }
}

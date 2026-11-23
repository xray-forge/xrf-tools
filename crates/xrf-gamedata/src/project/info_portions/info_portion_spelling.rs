/// Which known info portion a name nothing gives may have been meant as.
///
/// A typo between the side giving a portion and the side reading it leaves one name read and never given beside one
/// given, a character or two apart, such as `pri_a28_evacuation_start` read against `pri_a28_evacutaion_start`. A
/// sibling of a numbered family is as close, so the closest name is a suggestion, not a verdict.
pub(crate) struct InfoPortionSpelling;

impl InfoPortionSpelling {
  /// The most edits between a name and the one it may have been meant as.
  const MAX_EDITS: usize = 2;
  /// The leading characters a misspelling shares with its intended name: a level's prefix is rarely what is mistyped,
  /// and it keeps the search to names of the same quest line.
  const SHARED_PREFIX: usize = 4;

  /// The closest of `candidates` to `name`, at most two edits away, sharing its first four characters; the first in
  /// candidate order on a tie, which a caller makes deterministic by sorting them, and `None` where none is that close.
  pub(crate) fn find_closest<'a>(name: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let prefix: &str = name.get(..Self::SHARED_PREFIX)?;
    let mut closest: Option<&str> = None;
    let mut closest_edits: usize = Self::MAX_EDITS + 1;

    for candidate in candidates {
      if candidate == name || !candidate.starts_with(prefix) || candidate.len().abs_diff(name.len()) > Self::MAX_EDITS {
        continue;
      }

      if let Some(edits) = Self::count_edits(name, candidate)
        && edits < closest_edits
      {
        closest = Some(candidate);
        closest_edits = edits;
      }
    }

    closest
  }

  /// The optimal string alignment distance between two names: characters inserted, removed or replaced, and two
  /// neighbours swapped, as in `evacutaion`, each one edit; `None` once every way through exceeds the bound.
  ///
  /// A row whose least distance passes the bound ends the count: the next row grows from it by an edit, and a swap
  /// reaches back to the row before, whose least is at most one below it.
  fn count_edits(left: &str, right: &str) -> Option<usize> {
    let left: &[u8] = left.as_bytes();
    let right: &[u8] = right.as_bytes();
    let mut before: Vec<usize> = vec![0; right.len() + 1];
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    let mut current: Vec<usize> = vec![0; right.len() + 1];

    for row in 0..left.len() {
      current[0] = row + 1;

      for column in 0..right.len() {
        let replaced: usize = previous[column] + usize::from(left[row] != right[column]);
        let mut edits: usize = replaced.min(previous[column + 1] + 1).min(current[column] + 1);

        if row > 0 && column > 0 && left[row] == right[column - 1] && left[row - 1] == right[column] {
          edits = edits.min(before[column - 1] + 1);
        }

        current[column + 1] = edits;
      }

      if current.iter().min().is_some_and(|least| *least > Self::MAX_EDITS) {
        return None;
      }

      std::mem::swap(&mut before, &mut previous);
      std::mem::swap(&mut previous, &mut current);
    }

    Some(previous[right.len()]).filter(|edits| *edits <= Self::MAX_EDITS)
  }
}

#[cfg(test)]
mod tests {
  use super::InfoPortionSpelling;

  #[test]
  fn finds_the_closest_given_name_within_two_edits() {
    let given: [&str; 4] = [
      "zat_b12_item_5_spawned",
      "zat_b12_items_spawned_all",
      "jup_b12_item_3_spawned",
      "zat_b51_finishing_category_",
    ];

    assert_eq!(
      InfoPortionSpelling::find_closest("zat_b12_item_3_spawned", given),
      Some("zat_b12_item_5_spawned")
    );
    assert_eq!(
      InfoPortionSpelling::find_closest("zat_b51_finishing_category_1", given),
      Some("zat_b51_finishing_category_")
    );
    assert_eq!(InfoPortionSpelling::find_closest("pri_a28_start", given), None);
  }

  /// Two swapped neighbours are one edit, so a swap plus another typo is still within reach.
  #[test]
  fn counts_two_swapped_neighbours_as_one_edit() {
    assert_eq!(
      InfoPortionSpelling::count_edits("pri_a28_evacuation_start", "pri_a28_evacutaion_start"),
      Some(1)
    );
    assert_eq!(
      InfoPortionSpelling::count_edits("pri_a28_evacuation_start", "pri_a28_evacutaion_stars"),
      Some(2)
    );
    assert_eq!(InfoPortionSpelling::count_edits("abcdef", "abxyzf"), None);
  }
}

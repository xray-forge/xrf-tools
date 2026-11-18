use crate::ai::level_ai_parts::LevelAiParts;

/// Each part is named by its lowest node, whatever order its links are joined in.
#[test]
fn names_each_part_by_its_lowest_node() {
  for joins in [[(4, 3), (3, 1), (5, 2)], [(1, 3), (2, 5), (3, 4)]] {
    let mut parts: LevelAiParts = LevelAiParts::new(6);

    for (first, second) in joins {
      parts.join(first, second);
    }

    assert_eq!(parts.into_names(), [0, 1, 2, 1, 1, 2], "{joins:?}");
  }
}

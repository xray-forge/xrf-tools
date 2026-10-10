use crate::scene::level::grass_cards::GrassCards;

// The cards hold what their reach's area holds, or more than their tufts asked where the grass stands denser: grown at
// once, kept while what is asked moves a little, and shrunk back once it falls to half the room.
#[test]
fn sizes_the_cards_to_what_their_tufts_ask() {
  let by_area: u32 = GrassCards::get_capacity(25.0, 0, 0);

  assert_eq!(by_area, 32_768);
  assert_eq!(GrassCards::get_capacity(25.0, by_area, 25_000), by_area);

  let grown: u32 = GrassCards::get_capacity(25.0, by_area, 40_000);

  assert_eq!(grown, 65_536);
  assert_eq!(GrassCards::get_capacity(25.0, grown, 36_000), grown);
  assert_eq!(GrassCards::get_capacity(25.0, grown, 20_000), by_area);
}

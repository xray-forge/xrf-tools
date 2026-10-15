use crate::scene::level::grass_cards::GrassCards;

// The cards hold more than their tufts asked, as a power of two: grown at once, kept while what is asked moves a little,
// and shrunk back once it falls to half the room.
#[test]
fn sizes_the_cards_to_what_their_tufts_ask() {
  let least: u32 = GrassCards::get_capacity(0, 0);

  assert_eq!(least, 16_384);

  let grown: u32 = GrassCards::get_capacity(least, 40_000);

  assert_eq!(grown, 65_536);
  assert_eq!(GrassCards::get_capacity(grown, 36_000), grown);
  assert_eq!(GrassCards::get_capacity(grown, 20_000), 32_768);
}

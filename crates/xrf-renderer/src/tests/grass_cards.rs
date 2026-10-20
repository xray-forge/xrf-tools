use crate::contract::render_grass_settings::RenderGrassSettings;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::scene::level::grass_cards::GrassCards;
use crate::scene::level::level_shadows::LevelShadows;

// The cards hold more than their tufts asked, as a power of two: grown at once, kept while what is asked moves a
// little, and shrunk back once it falls to half the room.
#[test]
fn sizes_the_cards_to_what_their_tufts_ask() {
  let least: u32 = GrassCards::get_capacity(0, 0);

  assert_eq!(least, 16_384);

  let grown: u32 = GrassCards::get_capacity(least, 40_000);

  assert_eq!(grown, 65_536);
  assert_eq!(GrassCards::get_capacity(grown, 36_000), grown);
  assert_eq!(GrassCards::get_capacity(grown, 20_000), 32_768);
}

// Drawn into the cascades alone, the cards stand no further than the last cascade's width they shadow.
#[test]
fn stands_the_cards_no_further_than_the_cascades_they_shadow() {
  let grass: RenderGrassSettings = RenderGrassSettings::default();
  let mut shadows: RenderShadowSettings = RenderShadowSettings::default();

  assert_eq!(LevelShadows::get_grass_reach(&shadows, 2), Some(40.0));
  assert_eq!(LevelShadows::get_grass_reach(&shadows, 1), Some(20.0));
  assert_eq!(LevelShadows::get_grass_reach(&shadows, 0), None);
  assert_eq!(GrassCards::get_reach(&grass, None).0, grass.shadow_radius);
  assert_eq!(
    GrassCards::get_reach(&grass, Some(40.0)).0,
    40.0_f32.min(grass.shadow_radius)
  );

  shadows.cascades = vec![30.0];
  assert_eq!(LevelShadows::get_grass_reach(&shadows, 2), Some(30.0));

  shadows.cascades.clear();
  assert_eq!(LevelShadows::get_grass_reach(&shadows, 2), None);
}

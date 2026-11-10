use glam::Vec4;

use crate::contract::feature::lighting::render_shadow_settings::RenderShadowSettings;
use crate::contract::feature::scene::render_grass_settings::RenderGrassSettings;
use crate::feature::scene::grass::cards::grass_cards::GrassCards;
use crate::tests::shader::wgsl_probe::run_wgsl_probe;

/// The largest chance `card_chance` gives, its 24 bits' last step.
const CHANCE_MOST: f32 = 1.0 - 1.0 / 16_777_216.0;

/// The share of its size each tuft stands at as a card, a case's `x` metres from the eye and of chance `y`
/// (`grass/card_thinning`).
fn thinned(cases: &[(f32, f32)]) -> Option<Vec<f32>> {
  let probes: Vec<(Vec4, Vec4)> = cases
    .iter()
    .map(|(from_eye, chance)| (Vec4::new(*from_eye, *chance, 0.0, 0.0), Vec4::ZERO))
    .collect();
  let answered: Vec<Vec4> = run_wgsl_probe(
    "grass/card_thinning",
    "vec4<f32>(card_thinned(card_kept(probe.first.x), probe.first.y), 0.0, 0.0, 0.0)",
    &probes,
  )?;

  Some(answered.iter().map(|answer| answer.x).collect())
}

// Short of the thinning every tuft stands whole as a card, whatever its chance: there, none joins or leaves.
#[test]
fn stands_every_tuft_whole_short_of_the_thinning() {
  let mut cases: Vec<(f32, f32)> = Vec::new();

  for from_eye in [0.0, 10.0, 39.9, GrassCards::THIN_FROM] {
    for chance in [0.0, 0.25, 0.5, 0.75, 0.9, 0.99, CHANCE_MOST] {
      cases.push((from_eye, chance));
    }
  }

  let Some(answered) = thinned(&cases) else {
    return;
  };

  for (share, (from_eye, chance)) in answered.iter().zip(&cases) {
    assert!(*share == 1.0, "{chance} at {from_eye} m: {share}");
  }
}

// Past the thinning a tuft shrinks and grows smoothly as the eye moves, never popping, and far off only a few stand.
#[test]
fn thins_the_tufts_smoothly_past_where_the_thinning_starts() {
  const STEP: f32 = 0.25;
  let chances: [f32; 7] = [0.05, 0.1, 0.3, 0.5, 0.75, 0.9, 0.99];
  let distances: Vec<f32> = (0..=480).map(|step| 30.0 + step as f32 * STEP).collect();
  let cases: Vec<(f32, f32)> = chances
    .iter()
    .flat_map(|chance| distances.iter().map(move |from_eye| (*from_eye, *chance)))
    .collect();
  let Some(answered) = thinned(&cases) else {
    return;
  };

  for (row, chance) in answered.chunks(distances.len()).zip(chances) {
    for (index, pair) in row.windows(2).enumerate() {
      assert!(
        (pair[1] - pair[0]).abs() <= 0.1,
        "{chance} from {} m: {} to {}",
        distances[index],
        pair[0],
        pair[1]
      );
    }

    if chance <= 0.05 || chance >= 0.3 {
      assert_eq!(
        row[row.len() - 1],
        if chance <= 0.05 { 1.0 } else { 0.0 },
        "{chance} at 150 m"
      );
    }
  }
}

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

// The traced cards stand every tuft whole no further than the thinning starts, past which most tufts stand as no card,
// at any shadow radius.
#[test]
fn stands_every_tuft_whole_no_further_than_the_thinning_starts() {
  for shadow_radius in [5.0, 20.0, 45.0, 50.0, 60.0, 100.0, 150.0] {
    let grass: RenderGrassSettings = RenderGrassSettings {
      shadow_radius,
      ..RenderGrassSettings::default()
    };
    let (reach, fade): (f32, f32) = GrassCards::get_reach(&grass, None);
    let whole: f32 = GrassCards::get_whole_reach(&grass);

    assert!(whole <= GrassCards::THIN_FROM, "{shadow_radius}: {whole}");
    assert!(whole <= reach - fade, "{shadow_radius}: {whole}");
  }
}

// Drawn into the cascades alone, the cards stand no further than the last cascade's width they shadow.
#[test]
fn stands_the_cards_no_further_than_the_cascades_they_shadow() {
  let grass: RenderGrassSettings = RenderGrassSettings::default();
  let mut shadows: RenderShadowSettings = RenderShadowSettings::default();

  assert_eq!(shadows.get_reach(2), Some(40.0));
  assert_eq!(shadows.get_reach(1), Some(20.0));
  assert_eq!(shadows.get_reach(0), None);
  assert_eq!(GrassCards::get_reach(&grass, None).0, grass.shadow_radius);
  assert_eq!(
    GrassCards::get_reach(&grass, Some(40.0)).0,
    40.0_f32.min(grass.shadow_radius)
  );

  shadows.cascades = vec![30.0];
  assert_eq!(shadows.get_reach(2), Some(30.0));

  shadows.cascades.clear();
  assert_eq!(shadows.get_reach(2), None);
}

use glam::Vec4;

use crate::tests::shader::wgsl_probe::run_wgsl_probe;

// Cases are (offset from the site as a share of its radius, metres from the eye, puddle draw distance); answers are
// (tuft share, puddle drawn).
fn kept(cases: &[(f32, f32, f32)]) -> Option<Vec<Vec4>> {
  let probes: Vec<(Vec4, Vec4)> = cases
    .iter()
    .map(|(across, from_eye, distance)| (Vec4::new(*across, *from_eye, *distance, 0.0), Vec4::ZERO))
    .collect();

  run_wgsl_probe(
    "common/puddle_site",
    "vec4<f32>(puddle_tuft(probe.first.x, puddle_drawn(probe.first.y, probe.first.z)),
      puddle_drawn(probe.first.y, probe.first.z), 0.0, 0.0)",
    &probes,
  )
}

#[test]
fn stands_a_tuft_wherever_no_puddle_is_drawn() {
  let cases: [(f32, f32, f32); 5] = [
    (0.2, 50.0, 30.0),
    (0.6, 31.0, 30.0),
    (0.2, 10.0, 30.0),
    (0.2, 10.0, 300.0),
    (1.2, 10.0, 300.0),
  ];
  let Some(answered) = kept(&cases) else {
    return;
  };

  assert_eq!(answered[0].x, 1.0, "{:?}: {}", cases[0], answered[0]);
  assert_eq!(answered[1].x, 1.0, "{:?}: {}", cases[1], answered[1]);
  assert_eq!(answered[2].x, 0.0, "{:?}: {}", cases[2], answered[2]);
  assert_eq!(answered[3].x, 0.0, "{:?}: {}", cases[3], answered[3]);
  assert_eq!(answered[4].x, 1.0, "{:?}: {}", cases[4], answered[4]);
}

#[test]
fn shrinks_a_tuft_smoothly_as_a_puddle_reaches_it() {
  let across: Vec<(f32, f32, f32)> = (0..=120).map(|step| (step as f32 * 0.01, 10.0, 30.0)).collect();
  let along: Vec<(f32, f32, f32)> = (0..=200).map(|step| (0.2, 20.0 + step as f32 * 0.1, 30.0)).collect();

  for cases in [across, along] {
    let Some(answered) = kept(&cases) else {
      return;
    };

    for (index, pair) in answered.windows(2).enumerate() {
      assert!(
        (pair[1].x - pair[0].x).abs() <= 0.2,
        "{:?} to {:?}: {} to {}",
        cases[index],
        cases[index + 1],
        pair[0].x,
        pair[1].x
      );
    }
  }
}

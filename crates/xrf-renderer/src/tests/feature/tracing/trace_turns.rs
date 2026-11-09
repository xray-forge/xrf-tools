//! The traces' turns (`common/trace_turns`) on the GPU: under ray budgets a quad tracing one ray a frame passes it
//! from pixel to pixel, and each pixel asks the scene for its ray once in its `turns`; the two must meet.

use glam::Vec4;

use crate::tests::shader::gpu_probe::run_gpu_probe;

/// The turns a pixel's scene ray waits, a still, dim one's and the list's wait included.
const TURNS: [u32; 5] = [2, 4, 8, 16, 32];

// Each four frames from a multiple of four a quad's rays start once at each of its pixels, so one ray a frame walks
// them all.
#[test]
fn a_quad_s_rays_start_at_each_pixel_in_four_frames() {
  // Each start's bits ride a float, as the probe's cases are floats.
  let starts: [u32; 5] = [0, 4, 1_000, 1 << 20, u32::MAX - 3];
  let cases: Vec<(Vec4, Vec4)> = starts
    .iter()
    .flat_map(|start| {
      (0..4).map(move |picked| {
        (
          Vec4::new(picked as f32, 0.0, 0.0, 0.0),
          Vec4::new(f32::from_bits(*start), 0.0, 0.0, 0.0),
        )
      })
    })
    .collect();
  let body: &str = "let start: u32 = bitcast<u32>(second.x);
  var seen: u32 = 0u;

  for (var frame: u32 = 0u; frame < 4u; frame++) {
    seen |= 1u << quad_turn(start + frame, u32(first.x));
  }

  answer = vec3<f32>(f32(seen), 0.0, 0.0);";
  let Some(answered) = run_gpu_probe("common/trace_turns", body, &cases) else {
    return;
  };

  for (answer, (first, second)) in answered.iter().zip(&cases) {
    assert_eq!(answer.x, 15.0, "picked {} from {}", first.x, second.x.to_bits());
  }
}

// A quad tracing one ray a frame: within four times a pixel's turns some pixel of it both traces and takes its turn to
// ask the scene, at every place of the 8 × 8 dither, for every pixel its noise picks to start from, from any frame. A
// quad that never meets its turn holds what it reflected the frame before for as long as that holds.
#[test]
fn a_one_ray_quad_takes_its_scene_turn() {
  let mut cases: Vec<(Vec4, Vec4)> = Vec::new();

  for y in 0..4 {
    for x in 0..4 {
      for picked in 0..4 {
        for turns in TURNS {
          for start in [0.0, 1_001.0, 1_048_573.0] {
            cases.push((
              Vec4::new(x as f32, y as f32, picked as f32, turns as f32),
              Vec4::new(start, 0.0, 0.0, 0.0),
            ));
          }
        }
      }
    }
  }

  let body: &str = "let quad: vec2<u32> = vec2<u32>(u32(first.x), u32(first.y)) * 2u;
  let picked: u32 = u32(first.z);
  let turns: u32 = u32(first.w);
  let start: u32 = u32(second.x);
  var met: f32 = -1.0;

  for (var frame: u32 = start; frame < start + turns * 4u; frame++) {
    let corner: u32 = QUAD_ORDER[quad_turn(frame, picked)];
    let pixel: vec2<u32> = quad + vec2<u32>(corner & 1u, corner >> 1u);

    if (met < 0.0 && is_on_trace_turn(pixel, turns, frame)) {
      met = f32(frame - start);
    }
  }

  answer = vec3<f32>(met, 0.0, 0.0);";
  let Some(answered) = run_gpu_probe("common/trace_turns", body, &cases) else {
    return;
  };
  let starved: Vec<&Vec4> = answered
    .iter()
    .zip(&cases)
    .filter(|(answer, _)| answer.x < 0.0)
    .map(|(_, (first, _))| first)
    .collect();
  let by_turns: Vec<(u32, usize)> = TURNS
    .iter()
    .map(|turns| (*turns, starved.iter().filter(|first| first.w as u32 == *turns).count()))
    .collect();

  assert!(
    starved.is_empty(),
    "{} of {} cases never meet their turn, by turns {by_turns:?}",
    starved.len(),
    cases.len()
  );
}

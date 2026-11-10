//! The traces' turns (`common/trace_turns`) on the GPU: under ray budgets a quad tracing one ray a frame passes it
//! from pixel to pixel, and asks the scene for its ray once in its `turns`; the two must meet.

use std::collections::BTreeSet;

use glam::Vec4;

use crate::contract::feature::tracing::render_reflection_settings::RenderReflectionSettings;
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::shader::fixtures::get_constant;
use crate::tests::shader::gpu_probe::run_gpu_probe;

/// Every `turns` a reflection's pixel can wait for its scene ray (`is_trace_turn`): each trace interval the settings
/// take, by one or a still, dim pixel's `DIM_TURNS`, by each wait the list stretches its turns by, up to
/// `TRACE_REQUEST_WAIT_MOST`.
fn every_turns() -> Vec<u32> {
  let library: ShaderLibrary = ShaderLibrary::default();
  let dim: u32 = get_constant(&library, "frame/reflections", "DIM_TURNS");
  let wait_most: u32 = get_constant(&library, "common/reflection_trace", "TRACE_REQUEST_WAIT_MOST");
  let mut every: BTreeSet<u32> = BTreeSet::new();

  for interval in 1..=RenderReflectionSettings::TRACE_INTERVAL_MOST {
    for scale in [1, dim] {
      for wait in 1..=wait_most {
        every.insert(interval * scale * wait);
      }
    }
  }

  every.into_iter().collect()
}

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

// A quad tracing one ray a frame traces each of its pixels on frames of every residue of four within 64 frames, so a
// pixel traces each cell of its block in turn, whatever its noise picks.
#[test]
fn a_one_ray_quad_traces_each_pixel_on_every_residue_of_four() {
  let cases: Vec<(Vec4, Vec4)> = [0_u32, 1_000, 1 << 20]
    .iter()
    .flat_map(|start| {
      (0..4).map(move |picked| {
        (
          Vec4::new(picked as f32, 0.0, 0.0, 0.0),
          Vec4::new(*start as f32, 0.0, 0.0, 0.0),
        )
      })
    })
    .collect();
  // Each corner's residues of four seen, four bits a corner.
  let body: &str = "let start: u32 = u32(second.x);
  var seen: u32 = 0u;

  for (var frame: u32 = start; frame < start + 64u; frame++) {
    let corner: u32 = QUAD_ORDER[quad_turn(frame, u32(first.x))];

    seen |= 1u << (corner * 4u + frame % 4u);
  }

  answer = vec3<f32>(f32(seen), 0.0, 0.0);";
  let Some(answered) = run_gpu_probe("common/trace_turns", body, &cases) else {
    return;
  };

  for (answer, (first, second)) in answered.iter().zip(&cases) {
    assert_eq!(answer.x, 65_535.0, "picked {} from {}", first.x, second.x);
  }
}

// A quad tracing one ray a frame asks the scene within its `turns` frames, whatever `turns` the interval, a dim
// pixel's scale and the list's wait make: some pixel of it both traces and takes its turn, at every place of the 8 × 8
// dither, for every pixel its noise picks to start from, from any frame. A quad that misses its turn holds what it
// reflected the frame before until it meets one; past `turns` frames it lags the pixels about it.
#[test]
fn a_one_ray_quad_asks_the_scene_within_its_turns() {
  const STARTS: [u32; 12] = [0, 1, 2, 3, 5, 8, 13, 101, 997, 4_093, 65_537, 1_048_573];
  let every: Vec<u32> = every_turns();
  let cases_of = |turns: u32| -> Vec<(Vec4, Vec4)> {
    let mut cases: Vec<(Vec4, Vec4)> = Vec::new();

    for y in 0..4 {
      for x in 0..4 {
        for picked in 0..4 {
          for start in STARTS {
            cases.push((
              Vec4::new(x as f32, y as f32, picked as f32, turns as f32),
              Vec4::new(start as f32, 0.0, 0.0, 0.0),
            ));
          }
        }
      }
    }

    cases
  };
  // Each case's first frame on which its quad's traced pixel takes its turn, counted from its start; none within 64
  // times its turns as -1.
  let body: &str = "let quad: vec2<u32> = vec2<u32>(u32(first.x), u32(first.y)) * 2u;
  let picked: u32 = u32(first.z);
  let turns: u32 = u32(first.w);
  let start: u32 = u32(second.x);
  var met: f32 = -1.0;

  for (var frame: u32 = start; frame < start + turns * 64u; frame++) {
    let corner: u32 = QUAD_ORDER[quad_turn(frame, picked)];

    if (is_on_quad_trace_turn(quad + vec2<u32>(corner & 1u, corner >> 1u), turns, frame)) {
      met = f32(frame - start);
      break;
    }
  }

  answer = vec3<f32>(met, 0.0, 0.0);";
  // The most frames any quad waits at each `turns`, none met as `u32::MAX`; a probe holds as many `turns` as one
  // dispatch's workgroups can, a case each.
  let per_probe: usize = 65_535 / cases_of(1).len();
  let mut worst: Vec<(u32, u32)> = Vec::new();

  for group in every.chunks(per_probe) {
    let cases: Vec<(Vec4, Vec4)> = group.iter().flat_map(|turns| cases_of(*turns)).collect();
    let Some(answered) = run_gpu_probe("common/trace_turns", body, &cases) else {
      return;
    };

    for (turns, answers) in group.iter().zip(answered.chunks(cases.len() / group.len())) {
      let most: u32 = answers
        .iter()
        .map(|answer| if answer.x < 0.0 { u32::MAX } else { answer.x as u32 })
        .max()
        .unwrap_or(0);

      worst.push((*turns, most));
    }
  }

  let late: Vec<&(u32, u32)> = worst.iter().filter(|(turns, most)| most >= turns).collect();

  assert!(
    late.is_empty(),
    "{} of {} turns wait their turns or longer (turns, most frames): {:?}",
    late.len(),
    worst.len(),
    &late[..late.len().min(16)]
  );
}

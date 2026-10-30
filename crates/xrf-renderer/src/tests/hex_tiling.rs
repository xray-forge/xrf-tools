//! The terrain's hex tiling (`common/hex_tiling`), run on the GPU: its copies, their blend, and the engine's copy that
//! leaves a coordinate where it is.

use glam::{Vec2, Vec4};

use crate::tests::wgsl_probe::run_wgsl_probe;

/// The tiles about `probe.first.xy`, at a rate of one, turned freely.
const TILES: &str = "hex_tiles(probe.first.xy, 1.0, 1.0)";

/// Coordinates near the origin and as far out as a terrain's details reach, on and off the grid's corners.
fn coordinates() -> Vec<Vec2> {
  vec![
    Vec2::new(0.13, 0.71),
    Vec2::new(0.5, 0.5),
    Vec2::new(-2.37, 4.02),
    Vec2::new(17.31, 9.94),
    Vec2::new(311.7, 486.2),
    Vec2::new(512.03, 1.27),
  ]
}

fn run(answer: &str, cases: &[(Vec4, Vec4)]) -> Option<Vec<Vec4>> {
  run_wgsl_probe("common/hex_tiling", &answer.replace("TILES", TILES), cases)
}

fn at(uv: Vec2, second: Vec4) -> (Vec4, Vec4) {
  (uv.extend(0.0).extend(0.0), second)
}

#[test]
fn the_engines_copy_leaves_every_coordinate_and_direction_as_it_is() {
  let cases: Vec<(Vec4, Vec4)> = coordinates()
    .into_iter()
    .map(|uv| at(uv, Vec4::new(0.3, -0.7, 0.0, 0.0)))
    .collect();
  let Some(answered) = run(
    "vec4<f32>(hex_at(hex_unmoved(), probe.first.xy), hex_back(hex_unmoved(), probe.second.xy))",
    &cases,
  ) else {
    return;
  };

  // Exactly: the default tiling must draw the terrain bit for bit as before.
  for ((first, second), found) in cases.iter().zip(answered) {
    assert_eq!(found, Vec4::new(first.x, first.y, second.x, second.y));
  }
}

#[test]
fn a_direction_turned_into_a_copy_turns_back() {
  let cases: Vec<(Vec4, Vec4)> = coordinates()
    .into_iter()
    .map(|uv| at(uv, Vec4::new(0.6, 0.8, 0.0, 0.0)))
    .collect();
  let Some(answered) = run(
    "vec4<f32>(hex_back(hex_copy(TILES, 1u), hex_along(hex_copy(TILES, 1u), probe.second.xy)), \
     hex_along(hex_copy(TILES, 1u), probe.second.xy))",
    &cases,
  ) else {
    return;
  };

  for found in answered {
    assert!(
      (found.truncate().truncate() - Vec2::new(0.6, 0.8)).length() < 1e-5,
      "{found}"
    );
    assert!(
      (Vec2::new(found.z, found.w).length() - 1.0).abs() < 1e-5,
      "a turn keeps lengths: {found}"
    );
  }
}

#[test]
fn the_copies_nearness_and_their_weights_each_sum_to_one() {
  let cases: Vec<(Vec4, Vec4)> = coordinates()
    .into_iter()
    .map(|uv| at(uv, Vec4::new(0.2, 0.9, 0.5, 0.0)))
    .collect();
  let Some(nearness) = run("vec4<f32>(TILES.nearness, 0.0)", &cases) else {
    return;
  };
  let Some(weights) = run("vec4<f32>(hex_weights(TILES, probe.second.xyz, 7.0), 0.0)", &cases) else {
    return;
  };

  for (near, weight) in nearness.into_iter().zip(weights) {
    assert!((near.x + near.y + near.z - 1.0).abs() < 1e-4, "{near}");
    assert!(near.min_element() >= -1e-5, "{near}");
    assert!((weight.x + weight.y + weight.z - 1.0).abs() < 1e-4, "{weight}");
  }
}

#[test]
fn a_sharper_blend_gives_the_nearest_copy_more_of_its_seam() {
  // Off the corners, where the copies blend; equal brightness, so nearness alone decides.
  let cases: Vec<(Vec4, Vec4)> = coordinates()
    .into_iter()
    .map(|uv| at(uv, Vec4::new(0.5, 0.5, 0.5, 0.0)))
    .collect();
  let nearest_weight: &str = "max(max(WEIGHTS.x, WEIGHTS.y), WEIGHTS.z)";
  let (Some(soft), Some(sharp)) = (
    run(
      &format!(
        "vec4<f32>({})",
        nearest_weight.replace("WEIGHTS", "hex_weights(TILES, probe.second.xyz, 2.0)")
      ),
      &cases,
    ),
    run(
      &format!(
        "vec4<f32>({})",
        nearest_weight.replace("WEIGHTS", "hex_weights(TILES, probe.second.xyz, 16.0)")
      ),
      &cases,
    ),
  ) else {
    return;
  };

  for (soft, sharp) in soft.iter().zip(&sharp) {
    assert!(sharp.x >= soft.x - 1e-6, "{soft} against {sharp}");
  }

  assert!(
    soft.iter().zip(&sharp).any(|(soft, sharp)| sharp.x > soft.x + 0.05),
    "{soft:?} against {sharp:?}"
  );
}

#[test]
fn a_corner_of_the_grid_shows_its_own_copy_alone() {
  // The origin is a corner: its copy shows whole there, however bright the others read.
  let Some(answered) = run(
    "vec4<f32>(hex_weights(TILES, probe.second.xyz, 7.0), f32(hex_nearest(TILES)))",
    &[at(Vec2::ZERO, Vec4::new(0.1, 0.9, 0.9, 0.0))],
  ) else {
    return;
  };

  assert_eq!(answered[0], Vec4::new(1.0, 0.0, 0.0, 0.0));
}

#[test]
fn a_copy_reads_a_step_away_as_its_neighbour_does_but_across_a_seam() {
  // Each coordinate's nearest copy and the copy nearest a step away read the step's end alike unless the step crossed
  // into another hexagon, which only a few of a fine sweep's steps do.
  let step: Vec2 = Vec2::new(0.003, 0.002);
  let cases: Vec<(Vec4, Vec4)> = (0..400)
    .map(|index| Vec2::new(index as f32 * 0.0137 + 3.0, (index as f32 * 0.31).sin() * 2.0 + 5.0))
    .map(|uv| at(uv, step.extend(0.0).extend(0.0)))
    .collect();
  let Some(answered) = run(
    "vec4<f32>(hex_at(hex_copy(TILES, hex_nearest(TILES)), probe.first.xy + probe.second.xy), \
     hex_at(hex_copy(hex_tiles(probe.first.xy + probe.second.xy, 1.0, 1.0), \
     hex_nearest(hex_tiles(probe.first.xy + probe.second.xy, 1.0, 1.0))), probe.first.xy + probe.second.xy))",
    &cases,
  ) else {
    return;
  };
  let alike: usize = answered
    .iter()
    .filter(|found| (Vec2::new(found.x, found.y) - Vec2::new(found.z, found.w)).length() < 1e-4)
    .count();

  assert!(alike * 10 >= answered.len() * 9, "{alike} of {} alike", answered.len());
}

#[test]
fn copies_stand_upright_without_turning_and_differ_from_hexagon_to_hexagon() {
  let cases: Vec<(Vec4, Vec4)> = coordinates().into_iter().map(|uv| at(uv, Vec4::ZERO)).collect();
  let Some(upright) = run(
    "vec4<f32>(hex_tiles(probe.first.xy, 1.0, 0.0).first.turn, hex_tiles(probe.first.xy, 1.0, 0.0).third.turn)",
    &cases,
  ) else {
    return;
  };
  // The first two coordinates lie close enough to share a corner, so only those standing apart are compared.
  let Some(shifts) = run("vec4<f32>(TILES.first.shift, TILES.second.shift)", &cases[2..]) else {
    return;
  };

  for turn in upright {
    assert_eq!(turn, Vec4::new(1.0, 0.0, 1.0, 0.0));
  }

  // A copy's shift is its own: no two corners of their triangles draw the same.
  let distinct: Vec<Vec2> = shifts
    .iter()
    .flat_map(|shift| [Vec2::new(shift.x, shift.y), Vec2::new(shift.z, shift.w)])
    .collect();

  for (index, shift) in distinct.iter().enumerate() {
    assert!(
      distinct[index + 1..]
        .iter()
        .all(|other| (*other - *shift).length() > 1e-3),
      "{distinct:?}"
    );
  }
}

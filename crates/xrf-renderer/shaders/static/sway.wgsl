#import "common/foliage_wind"
#import "generated/structs"

// A tree's vertex as the wind sways it, its wind passed in, so the G-buffer's draws and the trace's swaying trees move
// it alike.

// `calc_cyclic`: a wave from minus one to one over each whole turn, a parabola rather than a sine.
fn cyclic(phase: f32) -> f32 {
  let f: f32 = fract(phase) * 2.8284271 - 1.4142136;

  return f * f - 1.0;
}

// What a vertex is to the foliage motion, in a `foliage` pair's `x`: not foliage, a trunk, or a branch or leaf.
const FOLIAGE_NONE: f32 = 0.0;
const FOLIAGE_TRUNK: f32 = 1.0;
const FOLIAGE_BRANCH: f32 = 2.0;

// What a tree's vertex is to the foliage motion: a branch or a leaf where its surface is a cut-out, its trunk where it
// is not, nothing where it stands still.
fn tree_foliage_kind(is_still: bool, is_branch: bool) -> f32 {
  return select(select(FOLIAGE_TRUNK, FOLIAGE_BRANCH, is_branch), FOLIAGE_NONE, is_still);
}

// `deffer_tree_*.vs`: a tree's vertex in the world moved across the ground by the wind, as far as its height over the
// tree's foot times the wave at its place, and as much of that as its rigidity lets it; under the enhanced motion,
// by its trunk's swing or its branch's toss, `foliage` its kind and its texture's `v`.
fn tree_swayed(wind: Wind, world: vec3<f32>, foot: f32, rigidity: f32, foliage: vec2<f32>) -> vec3<f32> {
  if (is_foliage_enhanced(wind, foliage)) {
    return foliage_swayed(wind, world, foot, foliage, wind.foliage_anim);
  }

  return swayed_by(world, foot, rigidity, wind.wind, wind.wave);
}

// The same vertex as the frame before's wind swayed it.
fn tree_swayed_before(wind: Wind, world: vec3<f32>, foot: f32, rigidity: f32, foliage: vec2<f32>) -> vec3<f32> {
  if (is_foliage_enhanced(wind, foliage)) {
    return foliage_swayed(wind, world, foot, foliage, wind.foliage_previous_anim);
  }

  return swayed_by(world, foot, rigidity, wind.previous_wind, wind.previous_wave);
}

// Whether the enhanced motion moves a vertex of this kind.
fn is_foliage_enhanced(wind: Wind, foliage: vec2<f32>) -> bool {
  return wind.foliage_trees.w > 0.5 && foliage.x > 0.5;
}

// A tree's vertex moved by the enhanced motion at the fields' drift, read in the engine's space and carried back.
fn foliage_swayed(wind: Wind, world: vec3<f32>, foot: f32, foliage: vec2<f32>, anim: vec4<f32>) -> vec3<f32> {
  let setup: FoliageSetup = FoliageSetup(wind.foliage_wind, wind.foliage_grass, wind.foliage_trees, anim);
  let height: f32 = world.y - foot;
  var moved: vec3<f32>;

  if (foliage.x > 1.5) {
    moved = foliage_branches(vec2<f32>(world.x, -world.z), foot, height, foliage.y, setup);
  } else {
    let trunk: vec3<f32> = foliage_trunk(foot, height, setup);

    moved = vec3<f32>(trunk.x, 0.0, trunk.y);
  }

  return world + vec3<f32>(moved.x, moved.y, -moved.z);
}

fn swayed_by(world: vec3<f32>, foot: f32, rigidity: f32, lean_wind: vec4<f32>, wave: vec4<f32>) -> vec3<f32> {
  let phase: f32 = cyclic(wave.w + dot(world, wave.xyz));
  let lean: vec2<f32> = lean_wind.xz * (world.y - foot) * phase * rigidity;

  return world + vec3<f32>(lean.x, 0.0, lean.y);
}

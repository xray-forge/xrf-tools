// What placing the puddles and drawing them share about a site: each world cell of the level surface holds at most
// one, its keep, shape and jitter drawn from one hash of the cell.

// How far the surface may rise and fall across a site and still hold water: a little, and a share of its radius, a
// gentle slope of some four degrees, which a puddle's water still lies on.
const SITE_LEVEL: vec2<f32> = vec2<f32>(0.15, 0.07);

// A hash of a world cell, from nothing to one each way, three ways: `x` ranks the site's keep, `y` and `z` turn and
// stretch its puddle.
fn site_hash(cell: vec2<f32>) -> vec3<f32> {
  var mixed: vec3<f32> = fract(vec3<f32>(cell.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));

  mixed += dot(mixed, mixed.yxz + 33.33);

  return fract((mixed.xxy + mixed.yzz) * mixed.zyx);
}

// Where a site stands in its world cell, from nothing to one each way: a hash of its own, so its spot does not follow
// its keep or its shape.
fn site_jitter(cell: vec2<f32>) -> vec2<f32> {
  let mixed: vec3<f32> = fract(vec3<f32>(cell.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
  let stirred: vec3<f32> = mixed + dot(mixed, mixed.yzx + 33.33);

  return fract((stirred.xx + stirred.yz) * stirred.zy);
}

// The world cell, whole metres over the cell size, of the first cell of a map whose corner is `corner`: the hashes
// are of world cells, so a site keeps its puddle as the map moves.
fn site_origin(corner: vec2<f32>, cell: f32) -> vec2<f32> {
  return floor(corner / cell + 0.5);
}

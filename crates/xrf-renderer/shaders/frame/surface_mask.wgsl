// Places the puddles on the level's surface seen from overhead, once each time it is drawn: its lowest heights, so a
// railing or a post over a deck is not taken for the deck; then a site a cell of the world, where a puddle may stand
// whole: a spot in the cell, and the largest disc about it that is one level surface with no water near. Heights are
// metres in renderer space.

#import "common/rain_cover"
#import "generated/frame/surface_mask"

// Texels a side the lowest height is taken over: at half a metre a texel, a metre and a half.
const LOWEST_REACH: i32 = 1;
// Metres a cell of sites is across, which divides the map's step so cells stay where they are in the world.
const SITE_CELL: f32 = 8.0;
// The radii a site is tried at, largest first, metres; the share more of each around it that must be level too.
const SITE_RADII: vec4<f32> = vec4<f32>(3.5, 2.5, 1.8, 1.2);
const SITE_MARGIN: f32 = 1.25;
// How far the surface may rise and fall across a site and still hold water: a little, and a share of its radius, a
// gentle slope of some four degrees, which a puddle's water still lies on.
const SITE_LEVEL: vec2<f32> = vec2<f32>(0.15, 0.07);
// Metres past a site's edge no water may lie within, and how far below the site it must be at most.
const SITE_SHORE: vec2<f32> = vec2<f32>(3.0, 1.0);
const GOLDEN_ANGLE: f32 = 2.39996323;
const NOTHING: f32 = -1e8;

// The height a texel of the overhead depth stands at, nothing where nothing stands.
fn height_at(texel: vec2<i32>) -> f32 {
  let last: vec2<i32> = vec2<i32>(textureDimensions(surface)) - 1;
  let depth: f32 = textureLoad(surface, clamp(texel, vec2<i32>(0), last), 0);

  return select(shape.window.w - (1.0 - depth) * shape.shape.y, NOTHING * 2.0, depth <= 0.0);
}

@compute @workgroup_size(8, 8)
fn surface_lowest(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec2<u32> = textureDimensions(lowest);

  if (any(id.xy >= size)) {
    return;
  }

  var least: f32 = 1e9;

  for (var y: i32 = -LOWEST_REACH; y <= LOWEST_REACH; y++) {
    for (var x: i32 = -LOWEST_REACH; x <= LOWEST_REACH; x++) {
      least = min(least, height_at(vec2<i32>(id.xy) + vec2<i32>(x, y)));
    }
  }

  textureStore(lowest, id.xy, vec4<f32>(least, 0.0, 0.0, 0.0));
}

// The lowest height at a world place, nothing outside the map.
fn lowest_at(place: vec2<f32>) -> f32 {
  let uv: vec2<f32> = (place - shape.window.xy) / (shape.window.z * 2.0) + 0.5;

  if (any(uv < vec2<f32>(0.0)) || any(uv >= vec2<f32>(1.0))) {
    return NOTHING * 2.0;
  }

  return textureLoad(heights, vec2<i32>(uv * shape.shape.x), 0).x;
}

// The water's height at a world place, nothing where there is none.
fn water_at(place: vec2<f32>) -> f32 {
  return overhead_height(water, shape.window, shape.shape.x, shape.shape.y, vec3<f32>(place.x, 0.0, place.y));
}

// A hash of a cell, from nothing to one each way.
fn cell_hash(cell: vec2<f32>) -> vec2<f32> {
  let mixed: vec3<f32> = fract(vec3<f32>(cell.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
  let stirred: vec3<f32> = mixed + dot(mixed, mixed.yzx + 33.33);

  return fract((stirred.xx + stirred.yz) * stirred.zy);
}

// Whether a disc about a centre at a height is one level surface, every sample of it within reach of the centre's
// height, and how far its heights spread.
fn is_level(centre: vec2<f32>, height: f32, radius: f32) -> bool {
  let reach: f32 = SITE_LEVEL.x + SITE_LEVEL.y * radius;
  let around: f32 = radius * SITE_MARGIN;

  for (var tap: u32 = 0u; tap < 32u; tap++) {
    let along: f32 = sqrt((f32(tap) + 0.5) / 32.0) * around;
    let angle: f32 = f32(tap) * GOLDEN_ANGLE;
    let there: f32 = lowest_at(centre + vec2<f32>(cos(angle), sin(angle)) * along);

    if (abs(there - height) > reach) {
      return false;
    }
  }

  return true;
}

// Whether water lies within `SITE_SHORE.x` of a disc's edge, or anywhere under it, no further than `SITE_SHORE.y`
// below its height.
fn is_near_water(centre: vec2<f32>, height: f32, radius: f32) -> bool {
  for (var ring: u32 = 0u; ring < 3u; ring++) {
    let along: f32 = (radius + SITE_SHORE.x) * f32(ring + 1u) / 3.0;

    for (var tap: u32 = 0u; tap < 12u; tap++) {
      let angle: f32 = f32(tap) * 0.5235988 + f32(ring) * 0.26;

      if (water_at(centre + vec2<f32>(cos(angle), sin(angle)) * along) > height - SITE_SHORE.y) {
        return true;
      }
    }
  }

  return false;
}

// A cell's site: the world place of its spot in `xy`, its height in `z`, and the largest radius it holds whole in
// `w`, none where none fits.
@compute @workgroup_size(8, 8)
fn surface_sites(@builtin(global_invocation_id) id: vec3<u32>) {
  if (any(id.xy >= textureDimensions(sites))) {
    return;
  }

  let corner: vec2<f32> = shape.window.xy - shape.window.z + vec2<f32>(id.xy) * SITE_CELL;
  let jitter: vec2<f32> = cell_hash(floor(corner / SITE_CELL + 0.5));
  let centre: vec2<f32> = corner + (0.15 + jitter * 0.7) * SITE_CELL;
  let height: f32 = lowest_at(centre);
  var site: vec4<f32> = vec4<f32>(centre, height, 0.0);

  if (height > NOTHING) {
    for (var index: u32 = 0u; index < 4u; index++) {
      let radius: f32 = SITE_RADII[index];

      if (is_level(centre, height, radius) && !is_near_water(centre, height, radius)) {
        site.w = radius;

        break;
      }
    }
  }

  textureStore(sites, id.xy, site);
}

// Places the puddles on the level's surface seen from overhead. Each time it is drawn: its lowest heights, so a railing
// or a post over a deck is not taken for the deck; then a site a cell of the world, where a puddle may stand whole: a
// spot in the cell, and the largest disc about it that is one level surface with no water near. Each frame: which
// sites hold a puddle and how big, by the rain. Heights are metres in renderer space.

#import "common/rain_cover"
#import "common/puddle_site"
#import "generated/frame/puddle_sites"

// Texels a side the lowest height is taken over: at half a metre a texel, a metre and a half.
const LOWEST_REACH: i32 = 1;
// The radii a site is tried at, largest first, metres; the share more of each around it that must be level too.
const SITE_RADII: vec4<f32> = vec4<f32>(3.5, 2.5, 1.8, 1.2);
const SITE_MARGIN: f32 = 1.25;
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

  let cell: f32 = shape.shape.z;
  let corner: vec2<f32> = shape.window.xy - shape.window.z;
  let jitter: vec2<f32> = site_jitter(site_origin(corner, cell) + vec2<f32>(id.xy));
  let centre: vec2<f32> = corner + (vec2<f32>(id.xy) + 0.15 + jitter * 0.7) * cell;
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

// Which sites hold a puddle, decided once a frame for each, so every pixel of a puddle sees the same: a share of them,
// set by the puddles setting and a broader noise at the site, more in a storm and after a long rain soaked the level;
// of those, no more than a few about each, the first by their hash keeping theirs so no stretch gathers a lake. A kept
// site's puddle grows from `PUDDLE_GROWTH` of its radius as the puddles fill, and a site newly within the share or the
// neighbour limit grows in rather than popping.
const SITE_SHARE: f32 = 0.7;
const STORM_SHARE: f32 = 0.6;
const STORM_GROWTH: f32 = 0.15;
const SOAK_SHARE: f32 = 0.5;
const SITE_NEIGHBOURS: f32 = 2.0;
const SOAK_NEIGHBOURS: f32 = 3.0;
const SITE_GROW_IN: f32 = 0.15;
const PUDDLE_GROWTH: f32 = 0.35;
// Metres the regional noise repeats over, turned off the cells' grid, and how far it moves the share either way.
const REGION_TILE: f32 = 240.0;
const REGION_TURN: mat2x2<f32> = mat2x2<f32>(0.8, 0.6, -0.6, 0.8);
const REGION_SHAPE: f32 = 0.6;

// The share of the sites holding a puddle about a site's spot, in renderer space.
fn share_at(spot: vec2<f32>) -> f32 {
  // The engine's axes, as the noise was laid out for: `z` negated.
  let place: vec2<f32> = vec2<f32>(spot.x, -spot.y);
  let region: f32 = textureSampleLevel(region_noise, noise_sampler, REGION_TURN * place / REGION_TILE, 0.0).r;

  return SITE_SHARE * wet.puddles.y / 0.8 * (1.0 + (region - 0.5) * 2.0 * REGION_SHAPE) *
    (1.0 + STORM_SHARE * wet.puddle_state.z) * (1.0 + SOAK_SHARE * wet.puddle_state.w);
}

// A cell's puddle: its site's spot in `xy`, its height in `z`, and its radius this frame in `w`, none where it holds
// none.
@compute @workgroup_size(8, 8)
fn puddle_keep(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec2<i32> = vec2<i32>(textureDimensions(puddles));

  if (any(vec2<i32>(id.xy) >= size)) {
    return;
  }

  let at: vec2<i32> = vec2<i32>(id.xy);
  let site: vec4<f32> = textureLoad(kept_sites, at, 0);
  let origin: vec2<f32> = site_origin(wet.surface.xy - wet.surface.z, wet.surface_shape.z);
  let rank: f32 = site_hash(origin + vec2<f32>(at)).x;
  let share: f32 = share_at(site.xy);
  var radius: f32 = 0.0;

  if (site.w > 0.0 && rank < share && wet.puddle_state.y > 0.0) {
    var before: f32 = 0.0;

    for (var y: i32 = -1; y <= 1; y++) {
      for (var x: i32 = -1; x <= 1; x++) {
        let other: vec2<i32> = at + vec2<i32>(x, y);

        if ((x == 0 && y == 0) || any(other < vec2<i32>(0)) || any(other >= size)) {
          continue;
        }

        let neighbour: vec4<f32> = textureLoad(kept_sites, other, 0);
        let other_rank: f32 = site_hash(origin + vec2<f32>(other)).x;

        if (neighbour.w > 0.0 && other_rank < rank && other_rank < share_at(neighbour.xy)) {
          before += 1.0;
        }
      }
    }

    let room: f32 = saturate(SITE_NEIGHBOURS + SOAK_NEIGHBOURS * wet.puddle_state.w + 1.0 - before);
    let growth: f32 = mix(PUDDLE_GROWTH, 1.0, wet.puddle_state.y) * (1.0 + STORM_GROWTH * wet.puddle_state.z);

    radius = site.w * growth * saturate((share - rank) / SITE_GROW_IN) * room;
  }

  textureStore(puddles, at, vec4<f32>(site.xyz, radius));
}

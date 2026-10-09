#import "common/camera"
#import "common/octahedral"
#import "common/rain_cover"
#import "common/fullscreen"
#import "common/wet"

// `rain_patch_normal` (`r3_rendertarget_draw_rain.cpp`): where the rain reaches a surface near the camera, splashes on
// what faces up and water running down what stands, as a normal bent in view space, and how wet it is in alpha, weighed
// by the albedo's brightness. Computed in the engine's axes, so the ripples run as the game's do.

// The G-buffer and its material marks, the rain's cover, `s_water` (a volume of rippling normals as its slices) and
// `s_waterFall` (the normals of water running down), the enhanced wetting's rain ripples, puddle noise and puddle
// normals, and the settings.
#import "generated/frame/wet_patch"

// Metres a point may stand under its cover's texel and still take the rain, as a slope spans one.
const COVER_BIAS: f32 = 0.2;

// The engines' values: metres past which the rain wets nothing, how fast the splashes' volume runs, how far the water
// tilts the normal up, how strongly and how fast the flow bends and runs.
fn engine_value(vanilla: f32, extended: f32) -> f32 {
  return select(vanilla, extended, wet.is_extended > 0.5);
}

// Whether the rain reaches a point: four taps of the cover about it, as `shadow_rain` jitters four, each open where
// the point stands at or above the first thing over its column.
fn rain_open(world: vec3<f32>) -> f32 {
  let texel: f32 = wet.window.z * 2.0 / RAIN_COVER_RESOLUTION;
  var open: f32 = 0.0;

  for (var tap: u32 = 0u; tap < 4u; tap++) {
    let offset: vec2<f32> = vec2<f32>(f32(tap & 1u), f32(tap >> 1u)) - 0.5;
    let column: vec3<f32> = world + vec3<f32>(texel * offset.x, 0.0, texel * offset.y);

    open += select(0.0, 1.0, world.y + COVER_BIAS >= rain_cover_height(cover, wet.window, column));
  }

  return open * 0.25;
}

// How far falling rain leans off straight down, as a tangent, and the taps and metres the disc its spread is read over
// spans, at least and at most: an overhang a height above a point shelters it from a disc that height by the spread
// across, so a pipe high overhead only thins the rain under it where a low roof stops it, and a fence's wire or a
// bush's twig, a sliver of the disc, shelters nothing.
const RAIN_SPREAD: f32 = 0.35;
const RAIN_SPREAD_TAPS: u32 = 12u;
const RAIN_SPREAD_LEAST: f32 = 0.5;
const RAIN_SPREAD_REACH: f32 = 4.0;
// The share of the disc open from which the rain wets a point whole: whatever stands over the rest of it is too thin
// or too sparse to keep it dry.
const RAIN_SPREAD_OPEN: f32 = 0.7;
// How far the ground's tilt is trusted, as the least of its up, so a wall's foot does not lift its plane off the disc.
const RAIN_SPREAD_UP: f32 = 0.3;
const GOLDEN_ANGLE: f32 = 2.39996323;

// How much of the rain reaches a point, falling with its spread: the share of a disc of the cover open over the
// ground's plane through it, the disc as wide as the spread makes the overhang straight above it, turned by where it
// lies so its taps do not line up from one point to the next.
fn rain_reach(world: vec3<f32>, normal: vec3<f32>) -> f32 {
  let texel: f32 = wet.window.z * 2.0 / RAIN_COVER_RESOLUTION;
  let over: f32 = rain_cover_height(cover, wet.window, world) - world.y - COVER_BIAS;
  let radius: f32 = clamp(over * RAIN_SPREAD, RAIN_SPREAD_LEAST, RAIN_SPREAD_REACH);
  // The plane's rise along the ground, metres a metre, each way.
  let tilt: vec2<f32> = -normal.xz / max(normal.y, RAIN_SPREAD_UP);
  let turn: f32 = fract(sin(dot(floor(world.xz / texel), vec2<f32>(12.9898, 78.233))) * 43758.5453) * 6.2831853;
  var open: f32 = 0.0;

  for (var tap: u32 = 0u; tap < RAIN_SPREAD_TAPS; tap++) {
    let along: f32 = sqrt((f32(tap) + 0.5) / f32(RAIN_SPREAD_TAPS)) * radius;
    let angle: f32 = f32(tap) * GOLDEN_ANGLE + turn;
    let offset: vec2<f32> = vec2<f32>(cos(angle), sin(angle)) * along;
    let ground: f32 = world.y + dot(tilt, offset);

    open += select(0.0, 1.0, ground + COVER_BIAS >= rain_cover_height(cover, wet.window,
      world + vec3<f32>(offset.x, 0.0, offset.y)));
  }

  return smoothstep(0.0, RAIN_SPREAD_OPEN, open / f32(RAIN_SPREAD_TAPS));
}

// `GetNVNMap`: the splashes' volume at a place and a moment, repeating every way and blended between its slices, its
// normal in `w`, `y` and `z`, laid flat.
fn splash_normal(place: vec2<f32>, moment: f32) -> vec3<f32> {
  let layers: f32 = f32(textureNumLayers(splash));
  let along: f32 = fract(moment) * layers - 0.5;
  let first: f32 = floor(along);
  let near: i32 = i32((first + layers) % layers);
  let far: i32 = i32((first + 1.0) % layers);
  let water: vec4<f32> = mix(
    textureSampleLevel(splash, wet_sampler, place, near, 0.0),
    textureSampleLevel(splash, wet_sampler, place, far, 0.0),
    along - first,
  ) - 0.5;

  return vec3<f32>(water.w * 6.0, engine_value(0.0, 0.1), water.z * 6.0);
}

// `GetWaterNMap`: the flow's normal at a point of it, laid flat.
fn flow_normal(at: vec2<f32>) -> vec3<f32> {
  let water: vec3<f32> = (textureSampleLevel(flow, wet_sampler, at, 0.0).xzy - 0.5) * 2.0 * engine_value(0.3, 0.4);

  return vec3<f32>(water.x, engine_value(0.0, 0.1), water.z);
}

@fragment
fn fs_wet_patch(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  // Reversed: nought where nothing was drawn.
  if (depth <= 0.0) {
    discard;
  }

  let albedo: vec3<f32> = textureLoad(albedo_target, texel, 0).rgb;
  let normal: vec3<f32> = normalize(octahedral_decode(textureLoad(normal_target, texel, 0).xy));
  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let size: vec2<f32> = camera.viewport.xy;
  let ndc: vec2<f32> = vec2<f32>(in.clip.x / size.x * 2.0 - 1.0, 1.0 - in.clip.y / size.y * 2.0);
  let world: vec3<f32> = camera_unproject(ndc, depth);
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let world_normal: vec3<f32> = normalize(rotation * normal);
  // The engine's axes: `z` negated.
  let place: vec3<f32> = vec3<f32>(world.x, world.y, -world.z);
  let facing: vec3<f32> = vec3<f32>(world_normal.x, world_normal.y, -world_normal.z);
  let fade: f32 = 1.0 - smoothstep(5.0, engine_value(20.0, 25.0), -position.z);
  // `-dot(Ldynamic_dir, N)`, the rain falling straight down.
  let up: f32 = facing.y;
  let wetness: f32 = rain_open(world) * fade * fade * wet.density * saturate(up * 10.0 + 10.0 * 0.5 + 0.5);
  let upward: f32 = max(up, 0.0);
  let splashed: vec3<f32> = splash_normal(place.xz, wet.time * engine_value(3.0, 1.0));
  let along: vec3<f32> = place / 2.0;
  let slide: f32 = ceil((1.0 - upward) * 10.0) * 0.1 * engine_value(0.5, 0.3);
  let fall_x: vec3<f32> = flow_normal(vec2<f32>(along.z, along.y + wet.time * slide));
  let fall_z: vec3<f32> = flow_normal(vec2<f32>(along.x, along.y + wet.time * slide));
  // Nothing on the weapon in hand, which the viewer has none of.
  let applied: f32 = wetness * smoothstep(0.8, 0.9, length(position));
  let water: vec3<f32> = splashed * (upward * applied) + fall_x.yxz * (abs(facing.x) * applied) +
    fall_z.zxy * (abs(facing.z) * applied);
  let bent: vec3<f32> = (camera.view * vec4<f32>(water.x, water.y, -water.z, 0.0)).xyz;

  return vec4<f32>(normalize(normal + bent), wetness * dot(albedo, vec3<f32>(0.33)));
}

// The enhanced wetting's ripples on what faces up: how large, how fast and how slow at least, and how strong; and its
// running water on what stands: how large its columns, how fast and how slow at least, and how strong.
const RIPPLE_SCALE: f32 = 0.5;
const RIPPLE_SPEED: f32 = 1.4;
const RIPPLE_LEAST_SPEED: f32 = 0.7;
const RIPPLE_STRENGTH: f32 = 1.25;
const FALL_SCALE: f32 = 0.8;
const FALL_SPEED: f32 = 1.5;
const FALL_LEAST_SPEED: f32 = 0.2;
const FALL_STRENGTH: f32 = 0.35;
const FALL_COLUMNS: f32 = 50.0;
// Metres within which rain ripples, and over which the wetting fades out; and the share of the cover's half width
// over which the cover gives way to open sky, so its square's edge never shows.
const RIPPLE_REACH: f32 = 15.0;
// Metres out to which water runs down what stands, fading: no further, where its narrow columns alias into a sheen.
const FALL_REACH: f32 = 25.0;
const WET_REACH: vec2<f32> = vec2<f32>(200.0, 250.0);
// The share of the puddles' distance over which they fade out.
const PUDDLE_FADE: f32 = 0.15;
// The reflectivity setting at which a puddle's water reflects as water does, the default; a lower one reflects less,
// a higher one more.
const WATER_REFLECTIVITY: f32 = 0.4;
const COVER_FADE: vec2<f32> = vec2<f32>(0.6, 0.95);
// Ripple layers: how fast each runs, and where the second and third are moved to.
const RIPPLE_LAYER_SPEEDS: vec3<f32> = vec3<f32>(1.05, 1.31, 1.58);
const RIPPLE_LAYER_OFFSETS: vec4<f32> = vec4<f32>(0.5, 0.25, 0.31, 0.5);
// Metres a puddle's noise repeats over, and its ripples' normals; and how strongly rain ripples a puddle.
const PUDDLE_TILE: f32 = 60.0;
const PUDDLE_RIPPLE_TILE: f32 = 12.0;
const PUDDLE_RIPPLE_SHARE: f32 = 0.5;
// Puddles stand at sites the level surface's placing found room for, a site a cell of `SITE_CELL` metres: how many
// of them hold one at the default puddles setting, and how far the setting and a broader noise over
// `PUDDLE_REGION_TILE` metres, turned off the cells' grid, move that, whole stretches wetter or drier.
const SITE_CELL: f32 = 8.0;
const SITE_SHARE: f32 = 0.7;
// How many sites about one, in the cells around it, may hold puddles before it holds none, those first by their hash
// keeping theirs, so no stretch gathers a lake of them: after a short rain, and how many more after a long one soaked
// the level; a site past the limit by less than one grows in by that much rather than popping. And how much more of
// the sites a long rain fills.
const SITE_NEIGHBOURS: f32 = 2.0;
const SOAK_NEIGHBOURS: f32 = 3.0;
const SOAK_SHARE: f32 = 0.5;
const PUDDLE_REGION_TILE: f32 = 240.0;
const PUDDLE_REGION_TURN: mat2x2<f32> = mat2x2<f32>(0.8, 0.6, -0.6, 0.8);
const PUDDLE_REGION_SHAPE: f32 = 0.6;
// A puddle's shape about its site: as long as wide at most by `PUDDLE_STRETCH`, its border moved in and out by a
// noise every `PUDDLE_EDGE_TILE` metres and by the puddles' own over `PUDDLE_TILE`, a share of its radius; how small it
// starts as the level wets, against its whole size; and the share of its radius its border softens over.
const PUDDLE_STRETCH: f32 = 1.6;
const PUDDLE_EDGE_TILE: f32 = 8.0;
const PUDDLE_EDGE_SHAPE: f32 = 0.35;
const PUDDLE_NOISE_SHAPE: f32 = 0.25;
const PUDDLE_GROWTH: f32 = 0.35;
const PUDDLE_SOFTNESS: f32 = 0.12;
// How much more of the sites a storm fills, and how much bigger it grows each puddle, within the margin the placing
// tested around them; and the share over which a site newly within the share grows in rather than popping.
const STORM_SHARE: f32 = 0.6;
const STORM_GROWTH: f32 = 0.15;
const SITE_GROW_IN: f32 = 0.15;
// How deep a puddle reads at its middle, against its rim, so its water darkens inward; and how far past its rim the
// ground around it stays darker and glossier with the water it soaked, a share of its radius, and how glossy.
const PUDDLE_DEEPENING: f32 = 0.6;
const PUDDLE_HALO: f32 = 0.4;
const PUDDLE_HALO_GLOSS: f32 = 0.25;
// How much the coat's coverage varies with the puddles' own noise, for floating film and debris.
const PUDDLE_FILM: f32 = 0.15;

// Metres a point may stand off its site's height, beside how far the placing let the ground rise and fall across it
// (a little and a share of its radius), and still be in its water; and how far over the level surface's lowest there
// it may stand: a stone or a post in a puddle stays dry, as does what lies under anything standing over the ground.
const PUDDLE_DEPTH: f32 = 0.15;
const SITE_LEVEL: vec2<f32> = vec2<f32>(0.15, 0.07);
const PUDDLE_TOP: f32 = 0.25;
// The least a surface faces up to hold a puddle.
const PUDDLE_UP: f32 = 0.85;

// A hash of a place, from nothing to one each way.
fn hash22(place: vec2<f32>) -> vec2<f32> {
  var p3: vec3<f32> = fract(vec3<f32>(place.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));

  p3 += dot(p3, p3.yzx + 19.19);

  return fract((p3.xx + p3.yz) * p3.zy);
}

// One layer of rain ripples: each texel a ring arriving on its own clock (arrival, normal y, normal x, clock),
// spreading and fading; `setup` is its speed, strength and how many rings a ripple spreads.
fn ripple_layer(texel: vec4<f32>, setup: vec3<f32>) -> vec2<f32> {
  let normal: vec2<f32> = texel.yz * 2.0 - 1.0;
  let moment: f32 = fract(texel.w + wet.time * setup.x);
  let age: f32 = moment - 1.0 + texel.x;
  let rings: f32 = clamp(age * setup.z, 0.0, 4.0);
  let height: f32 = saturate(0.7 - moment) * texel.x * sin(rings * 3.141592) * smoothstep(4.0, 0.0, rings);

  return normal * height * setup.y;
}

// How a place on the ground changes to the next pixel across and down, which the maps are filtered by.
struct PlaceGradients {
  across: vec2<f32>,
  down: vec2<f32>,
};

// Rain ripples at a place, three layers over each other, fading out to `RIPPLE_REACH` metres away; `gradients` are the
// place's.
fn rain_ripples(place: vec2<f32>, gradients: PlaceGradients, setup: vec3<f32>, distance: f32) -> vec2<f32> {
  let near: f32 = RIPPLE_REACH - distance;

  if (near < 0.0) {
    return vec2<f32>(0.0);
  }

  let first: vec4<f32> = textureSampleGrad(ripples, wet_sampler, place, gradients.across, gradients.down);
  let second: vec4<f32> = textureSampleGrad(ripples, wet_sampler, place * 0.61 + RIPPLE_LAYER_OFFSETS.xy,
    gradients.across * 0.61, gradients.down * 0.61);
  let third: vec4<f32> = textureSampleGrad(ripples, wet_sampler, place * 0.87 + RIPPLE_LAYER_OFFSETS.zw,
    gradients.across * 0.87, gradients.down * 0.87);
  let summed: vec2<f32> = ripple_layer(first, vec3<f32>(RIPPLE_LAYER_SPEEDS.x * setup.x, setup.yz)) +
    ripple_layer(second, vec3<f32>(RIPPLE_LAYER_SPEEDS.y * setup.x, setup.yz)) +
    ripple_layer(third, vec3<f32>(RIPPLE_LAYER_SPEEDS.z * setup.x, setup.yz));

  return clamp(summed * near / RIPPLE_REACH, vec2<f32>(-1.0), vec2<f32>(1.0));
}

// Water running down a wall at a place on it: narrow columns, each sliding at its own pace, faster as it pours;
// `gradients` are the place's.
fn running_water(place: vec2<f32>, gradients: PlaceGradients) -> vec3<f32> {
  let scaled: vec2<f32> = place * FALL_SCALE;
  let column: f32 = ceil(scaled.x * FALL_COLUMNS);
  var offset: f32 = hash22(vec2<f32>(column, 1.0)).x;

  offset = select(offset, offset * 3.0, offset < FALL_LEAST_SPEED);
  offset += wet.time * offset * wet.density * FALL_SPEED;

  let uv: vec2<f32> = vec2<f32>((fract(scaled.x * FALL_COLUMNS) + column) / FALL_COLUMNS, scaled.y + offset);

  let texel: vec4<f32> = textureSampleGrad(flow, wet_sampler, uv, gradients.across * FALL_SCALE,
    gradients.down * FALL_SCALE);

  return (texel.xzy * 2.0 - 1.0) * FALL_STRENGTH;
}

// Metres either side the ground's slope under a puddle is read at: across a puddle, past its bumps.
const GROUND_SPAN: f32 = 1.5;

// The level surface's lowest height at a place in the world, none outside its map.
fn lowest_at(place: vec2<f32>) -> f32 {
  let uv: vec2<f32> = clamp((place - wet.surface.xy) / (wet.surface.z * 2.0) + 0.5, vec2<f32>(0.0), vec2<f32>(0.999));

  return textureLoad(surface_lowest, vec2<i32>(uv * wet.surface_shape.x), 0).x;
}

// The ground's own up under a point in the world, from the level surface's lowest heights `GROUND_SPAN` metres either
// way: its slope across a puddle, past its bumps, which a puddle's water lies along so that the ground rising beyond
// it runs alongside its reflection rather than across it.
fn ground_up(world: vec3<f32>) -> vec3<f32> {
  let across: f32 = lowest_at(world.xz - vec2<f32>(GROUND_SPAN, 0.0)) - lowest_at(world.xz + vec2<f32>(GROUND_SPAN, 0.0));
  let down: f32 = lowest_at(world.xz - vec2<f32>(0.0, GROUND_SPAN)) - lowest_at(world.xz + vec2<f32>(0.0, GROUND_SPAN));

  return normalize(vec3<f32>(across, 2.0 * GROUND_SPAN, down));
}

// Whether a point is the topmost surface of the level there, by the level surface's lowest heights about it.
fn is_topmost(world: vec3<f32>) -> bool {
  let uv: vec2<f32> = (world.xz - wet.surface.xy) / (wet.surface.z * 2.0) + 0.5;

  if (any(uv < vec2<f32>(0.0)) || any(uv >= vec2<f32>(1.0))) {
    return false;
  }

  let at: vec2<i32> = vec2<i32>(uv * wet.surface_shape.x);
  let last: vec2<i32> = vec2<i32>(i32(wet.surface_shape.x) - 1);
  var highest: f32 = -1e9;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<i32> = vec2<i32>(i32(corner & 1u), i32(corner >> 1u));

    highest = max(highest, textureLoad(surface_lowest, clamp(at + offset, vec2<i32>(0), last), 0).x);
  }

  return world.y > highest - PUDDLE_TOP;
}

// A hash of a cell, from nothing to one each way, three ways.
fn site_hash(cell: vec2<f32>) -> vec3<f32> {
  var mixed: vec3<f32> = fract(vec3<f32>(cell.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));

  mixed += dot(mixed, mixed.yxz + 33.33);

  return fract((mixed.xxy + mixed.yzz) * mixed.zyx);
}

// How many sites holding puddles in the cells about one come before it by their hash.
fn sites_before(cell: vec2<f32>, origin: vec2<f32>, rank: f32, share: f32, cells: f32) -> f32 {
  var before: f32 = 0.0;

  for (var y: i32 = -1; y <= 1; y++) {
    for (var x: i32 = -1; x <= 1; x++) {
      let other: vec2<f32> = cell + vec2<f32>(f32(x), f32(y));

      if ((x == 0 && y == 0) || any(other < vec2<f32>(0.0)) || any(other >= vec2<f32>(cells))) {
        continue;
      }

      let hash: f32 = site_hash(origin + other).x;

      if (hash < rank && hash < share && textureLoad(puddle_sites, vec2<i32>(other), 0).w > 0.0) {
        before += 1.0;
      }
    }
  }

  return before;
}

// How much of a puddle a point is, how deep into it, and how much of the damp halo around it: the sites of its cell
// and those about it, each a whole stretched blob about its spot as big as the puddles have filled it (`growth`),
// those the share kept with no more than `neighbours` before them about it, a site newly within either growing in; `border` moves each blob's edge in and out, a share of its
// radius; `footprint` is the metres a pixel spans there, over which a border is softened at least, so a far one does
// not shimmer. Nothing outside the level surface's map.
fn site_puddle(world: vec3<f32>, growth: f32, share: f32, neighbours: f32, border: f32, footprint: f32)
  -> vec3<f32> {
  let cells: f32 = wet.surface.z * 2.0 / SITE_CELL;
  let corner: vec2<f32> = wet.surface.xy - wet.surface.z;
  let at: vec2<f32> = floor((world.xz - corner) / SITE_CELL);
  let origin: vec2<f32> = floor(corner / SITE_CELL + 0.5);
  var puddle: vec3<f32> = vec3<f32>(0.0);

  for (var y: i32 = -1; y <= 1; y++) {
    for (var x: i32 = -1; x <= 1; x++) {
      let cell: vec2<f32> = at + vec2<f32>(f32(x), f32(y));

      if (any(cell < vec2<f32>(0.0)) || any(cell >= vec2<f32>(cells))) {
        continue;
      }

      let site: vec4<f32> = textureLoad(puddle_sites, vec2<i32>(cell), 0);
      let hash: vec3<f32> = site_hash(origin + cell);

      if (site.w <= 0.0 || hash.x >= share || length(world.xz - site.xy) > site.w * 2.0) {
        continue;
      }

      let room: f32 = saturate(neighbours + 1.0 - sites_before(cell, origin, hash.x, share, cells));

      if (room <= 0.0) {
        continue;
      }

      let turn: f32 = hash.y * 6.2831853;
      let across: vec2<f32> = vec2<f32>(cos(turn), sin(turn));
      let offset: vec2<f32> = world.xz - site.xy;
      let stretched: vec2<f32> = vec2<f32>(dot(offset, across), dot(offset, vec2<f32>(-across.y, across.x))) /
        vec2<f32>(1.0, mix(1.0, 1.0 / PUDDLE_STRETCH, hash.z));
      let radius: f32 = site.w * growth * saturate((share - hash.x) / SITE_GROW_IN) * room;
      let distance: f32 = length(stretched) / radius + border;
      let softness: f32 = max(PUDDLE_SOFTNESS, 1.5 * footprint / radius);
      let band: f32 = PUDDLE_DEPTH + SITE_LEVEL.x + SITE_LEVEL.y * site.w;
      let seated: f32 = 1.0 - smoothstep(band * 0.75, band, abs(world.y - site.z));

      let inside: f32 = (1.0 - smoothstep(1.0 - softness, 1.0, distance)) * seated;
      let deep: f32 = saturate((1.0 - distance) / PUDDLE_DEEPENING) * inside;
      let halo: f32 = (1.0 - smoothstep(1.0, 1.0 + PUDDLE_HALO, distance)) * seated;

      puddle = max(puddle, vec3<f32>(inside, deep, halo));
    }
  }

  return puddle;
}

// What the enhanced wetting writes: into the light target, as the engine's wetting does, the normal packed into `xy`
// (its ripples and running water bent into it, a puddle levelling it), the gloss the rain adds into `z`, and how much
// of a puddle the point is into `w`; into the wet surface, what the enhanced rain adds over it: how much of the
// surface a puddle's clear coat of water covers, how much a puddle, and the puddle's ripple across the ground in the
// world's `x` and `z`, which shifts what it reflects rather than bending the normal the reflections are traced and
// held by. The coat's coverage is the puddle's scaled by the reflectivity setting, against water's.
struct WetPatchTargets {
  @location(0) patched: vec4<f32>,
  @location(1) surface: vec4<f32>,
};

// The enhanced wetting: the level's wetness wetting what the rain reaches out to the distance, ripples on what faces up
// and water running down what stands near the camera, and puddles on flat, low terrain, rippling.
@fragment
fn fs_wet_patch_enhanced(in: FullscreenVarying) -> WetPatchTargets {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    discard;
  }

  let marks: f32 = textureLoad(material_target, texel, 0).a;
  let is_flora: bool = has_mark(marks, MARK_PLANT);
  let is_terrain: bool = has_mark(marks, MARK_TERRAIN);
  let gloss: f32 = textureLoad(albedo_target, texel, 0).a;
  var normal: vec3<f32> = normalize(octahedral_decode(textureLoad(normal_target, texel, 0).xy));
  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let size: vec2<f32> = camera.viewport.xy;
  let ndc: vec2<f32> = vec2<f32>(in.clip.x / size.x * 2.0 - 1.0, 1.0 - in.clip.y / size.y * 2.0);
  let world: vec3<f32> = camera_unproject(ndc, depth);
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let world_normal: vec3<f32> = normalize(rotation * normal);
  // The engine's axes: `z` negated.
  let place: vec3<f32> = vec3<f32>(world.x, world.y, -world.z);
  let facing: vec3<f32> = vec3<f32>(world_normal.x, world_normal.y, -world_normal.z);
  let distance: f32 = -position.z;
  let wetness: f32 = wet.puddles.x;
  let soaked: f32 = saturate(wetness * 2.0);
  let pouring: f32 = clamp(wet.density * RIPPLE_SPEED, select(0.0, RIPPLE_LEAST_SPEED, wet.density > 0.0), 2.0);
  // The cover gives way to open sky towards its square's edge, past which nothing is known to stand over the rain.
  let edge: f32 = max(abs(world.x - wet.window.x), abs(world.z - wet.window.y)) / wet.window.z;
  let open: f32 = max(rain_reach(world, world_normal), smoothstep(COVER_FADE.x, COVER_FADE.y, edge));
  let kept: f32 = select(1.0, 0.0, is_flora);
  let gradients: PlaceGradients = PlaceGradients(dpdx(place.xz), dpdy(place.xz));
  let footprint: f32 = max(length(gradients.across), length(gradients.down));
  let wall_x: PlaceGradients = PlaceGradients(dpdx(place.zy), dpdy(place.zy));
  let wall_z: PlaceGradients = PlaceGradients(dpdx(place.xy), dpdy(place.xy));

  // Ripples on what faces up, water running down what stands, as much as the rain pours and reaches.
  let rippled: vec2<f32> = rain_ripples(place.xz * RIPPLE_SCALE,
    PlaceGradients(gradients.across * RIPPLE_SCALE, gradients.down * RIPPLE_SCALE),
    vec3<f32>(pouring, RIPPLE_STRENGTH * wet.puddles.w, 6.0), distance);
  var weights: vec3<f32> = facing * kept;
  var ripple: vec2<f32> = vec2<f32>(0.0);
  var water: vec3<f32> = vec3<f32>(rippled.x, 0.0, rippled.y) * smoothstep(0.75, 0.8, weights.y);

  weights = vec3<f32>(saturate(abs(weights.x) - 0.15), weights.y, saturate(abs(weights.z) - 0.15));
  weights = vec3<f32>(weights.x * f32(weights.x > weights.z), weights.y, weights.z * f32(weights.x < weights.z));
  // Down walls, rocks and objects, never terrain, whose bumps tilt its gentlest slopes past the threshold and lay the
  // columns across open ground.
  let falling: f32 = select(saturate((FALL_REACH - distance) / FALL_REACH), 0.0, is_terrain);

  water += (running_water(place.zy, wall_x).yxz * weights.x + running_water(place.xy, wall_z).zxy * weights.z) *
    falling;

  let reaching: f32 = kept * saturate(pouring) * open;

  water *= reaching;

  // The gloss the wetness gives what the rain reaches, none on what faces down, fading out in the distance; terrain
  // already glossy keeps a puddle's.
  let reached: f32 = saturate(open + f32(gloss > 0.3));
  var rain_gloss: f32 = (0.15 + saturate(pouring) * 0.05) * soaked * kept * reached;

  rain_gloss = select(rain_gloss, 0.15, is_terrain && gloss > 0.3);
  rain_gloss *= saturate(weights.y * 1.5) * smoothstep(WET_REACH.y, WET_REACH.x, length(position));

  // Puddles: whole puddles at the sites the level surface's placing found room for, level, whole and away from water,
  // terrain, a road or a bridge's deck alike; fewer where the setting and the stretch are drier, as big as the level
  // is wet, where the rain reaches, rippled by the rain.
  var puddle: f32 = 0.0;
  var film: f32 = 1.0;
  let fill: f32 = wet.puddle_state.y;
  let storm: f32 = wet.puddle_state.z;
  var deep: f32 = 0.0;

  // On the ground alone, terrain facing up and the topmost surface there: never a roof, a deck or a ceiling.
  let is_ground: bool = is_terrain && world_normal.y > PUDDLE_UP && is_topmost(world);

  if (is_ground && fill > 0.0 && length(position) < wet.puddle_state.x) {
    let noise: f32 = textureSampleGrad(puddle_noise, wet_sampler, place.xz / PUDDLE_TILE,
      gradients.across / PUDDLE_TILE, gradients.down / PUDDLE_TILE).r;
    let edge: f32 = textureSampleGrad(puddle_noise, wet_sampler, place.zx / PUDDLE_EDGE_TILE + 0.37,
      gradients.across.yx / PUDDLE_EDGE_TILE, gradients.down.yx / PUDDLE_EDGE_TILE).r;
    let region: f32 = textureSampleGrad(puddle_noise, wet_sampler,
      PUDDLE_REGION_TURN * place.xz / PUDDLE_REGION_TILE, PUDDLE_REGION_TURN * gradients.across / PUDDLE_REGION_TILE,
      PUDDLE_REGION_TURN * gradients.down / PUDDLE_REGION_TILE).r;
    let soak: f32 = wet.puddle_state.w;
    let share: f32 = SITE_SHARE * wet.puddles.y / 0.8 * (1.0 + (region - 0.5) * 2.0 * PUDDLE_REGION_SHAPE) *
      (1.0 + STORM_SHARE * storm) * (1.0 + SOAK_SHARE * soak);
    let neighbours: f32 = SITE_NEIGHBOURS + SOAK_NEIGHBOURS * soak;
    let growth: f32 = mix(PUDDLE_GROWTH, 1.0, fill) * (1.0 + STORM_GROWTH * storm);
    let border: f32 = (edge - 0.5) * PUDDLE_EDGE_SHAPE + (noise - 0.5) * PUDDLE_NOISE_SHAPE;

    // The cover only reaches some tens of metres; a site is on open ground already, so a puddle does not wait for it.
    let found: vec3<f32> = site_puddle(world, growth, share, neighbours, border, footprint) *
      smoothstep(wet.puddle_state.x, wet.puddle_state.x * (1.0 - PUDDLE_FADE), length(position));

    puddle = found.x;
    deep = found.y;
    film = 1.0 - PUDDLE_FILM * noise;
    rain_gloss = max(rain_gloss, PUDDLE_HALO_GLOSS * found.z * (1.0 - found.x));

    if (puddle > 0.0) {
      let pace: f32 = wet.time * (0.01 + saturate(wet.density * 1.5) * 0.008);
      let uv: vec2<f32> = place.xz / PUDDLE_RIPPLE_TILE;
      let across: vec2<f32> = gradients.across / PUDDLE_RIPPLE_TILE;
      let down: vec2<f32> = gradients.down / PUDDLE_RIPPLE_TILE;
      let waves: vec3<f32> = textureSampleGrad(puddle_normal, wet_sampler, uv + vec2<f32>(0.0, pace), across, down).xyz +
        textureSampleGrad(puddle_normal, wet_sampler, uv - vec2<f32>(0.33, pace), across, down).xyz - 1.0;
      let drops: vec2<f32> = rain_ripples(uv * 1.3, PlaceGradients(across * 1.3, down * 1.3),
        vec3<f32>(0.85, 1.0, 10.0), distance) * wet.density * 3.0 * wet.puddles.w * PUDDLE_RIPPLE_SHARE;
      let calm: vec2<f32> = waves.xy * saturate(0.05 + saturate(wet.density * 1.5) * 0.1);
      let surface: vec2<f32> = calm * 0.666 + drops * 0.15;

      normal = normalize(mix(normal, (camera.view * vec4<f32>(ground_up(world), 0.0)).xyz, puddle));
      ripple = surface * puddle;
      water *= 1.0 - puddle;
    }
  }

  let bent: vec3<f32> = (camera.view * vec4<f32>(water.x, water.y, -water.z, 0.0)).xyz;

  // The light target's `w` carries how deep into a puddle the point is, which tints it; the wet surface's `g` how much
  // of a puddle.
  return WetPatchTargets(
    vec4<f32>(octahedral_encode(normalize(normal + bent)), rain_gloss, max(deep, puddle * 0.01)),
    vec4<f32>(puddle * film * wet.puddles.z / WATER_REFLECTIVITY, puddle, ripple.x, -ripple.y),
  );
}

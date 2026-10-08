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

// How far falling rain leans off straight down, as a tangent, and the taps and metres at most its spread is read over:
// an overhang a height above a point shelters it from a disc that height by the spread across, so a pipe high overhead
// only thins the rain under it where a low roof stops it.
const RAIN_SPREAD: f32 = 0.35;
const RAIN_SPREAD_TAPS: u32 = 12u;
const RAIN_SPREAD_REACH: f32 = 4.0;
const GOLDEN_ANGLE: f32 = 2.39996323;

// How much of the rain reaches a point, falling with its spread: the share of a disc of the cover open over it, the disc
// as wide as the spread makes the overhang straight above it, if any.
fn rain_reach(world: vec3<f32>) -> f32 {
  let texel: f32 = wet.window.z * 2.0 / RAIN_COVER_RESOLUTION;
  let over: f32 = rain_cover_height(cover, wet.window, world) - world.y - COVER_BIAS;
  let radius: f32 = clamp(over * RAIN_SPREAD, texel, RAIN_SPREAD_REACH);
  var open: f32 = 0.0;

  for (var tap: u32 = 0u; tap < RAIN_SPREAD_TAPS; tap++) {
    let along: f32 = sqrt((f32(tap) + 0.5) / f32(RAIN_SPREAD_TAPS)) * radius;
    let angle: f32 = f32(tap) * GOLDEN_ANGLE;
    let column: vec3<f32> = world + vec3<f32>(cos(angle) * along, 0.0, sin(angle) * along);

    open += select(0.0, 1.0, world.y + COVER_BIAS >= rain_cover_height(cover, wet.window, column));
  }

  return open / f32(RAIN_SPREAD_TAPS);
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
const WET_REACH: vec2<f32> = vec2<f32>(200.0, 250.0);
const COVER_FADE: vec2<f32> = vec2<f32>(0.6, 0.95);
// Ripple layers: how fast each runs, and where the second and third are moved to.
const RIPPLE_LAYER_SPEEDS: vec3<f32> = vec3<f32>(1.05, 1.31, 1.58);
const RIPPLE_LAYER_OFFSETS: vec4<f32> = vec4<f32>(0.5, 0.25, 0.31, 0.5);
// Metres a puddle's noise repeats over, and its ripples' normals; the up of a surface's slope from which water starts
// to gather and at which it gathers fully (about ten and four degrees off level); how hard a puddle's
// border is, from soft to sharp; and how strongly rain ripples a puddle.
const PUDDLE_TILE: f32 = 60.0;
const PUDDLE_RIPPLE_TILE: f32 = 12.0;
const PUDDLE_FLATNESS: vec2<f32> = vec2<f32>(0.985, 0.998);
const PUDDLE_BORDER: f32 = 0.7;
// How far a slope raises the noise a puddle must reach (past all of it where the slope is steep), and how far a finer
// noise every `PUDDLE_EDGE_TILE` metres moves it either way, so a border follows the noise's curves rather than the
// line where the ground starts to tilt.
const PUDDLE_SLOPE_RISE: f32 = 1.2;
const PUDDLE_EDGE_TILE: f32 = 8.0;
const PUDDLE_EDGE_SHIFT: f32 = 0.35;
// Metres a broader noise repeats over, turned off the puddles' grid so neither repeats with the other, how much of it
// keeps puddles, and how far it raises the noise a puddle must reach where it keeps none: whole stretches stay dry.
const PUDDLE_REGION_TILE: f32 = 240.0;
const PUDDLE_REGION_TURN: mat2x2<f32> = mat2x2<f32>(0.8, 0.6, -0.6, 0.8);
const PUDDLE_REGION_KEPT: vec2<f32> = vec2<f32>(0.35, 0.65);
const PUDDLE_REGION_RISE: f32 = 0.9;
const PUDDLE_RIPPLE_SHARE: f32 = 0.5;
// Metres either side the depth is read at to find a surface's slope, so the slope spans its triangles and turns
// smoothly from one to the next as a vertex normal would, and never sees its bumps; and the frame pixels that span is
// held between.
const SURFACE_REACH: f32 = 3.0;
const SURFACE_SPAN: vec2<f32> = vec2<f32>(4.0, 640.0);

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

// The view space point the depth shows at a pixel, none where only the sky is.
fn depth_point(at: vec2<i32>) -> vec4<f32> {
  let clamped: vec2<i32> = clamp(at, vec2<i32>(0), vec2<i32>(camera.viewport.xy) - 1);
  let depth: f32 = textureLoad(depth_target, clamped, 0);

  return vec4<f32>(camera_view_position(vec2<f32>(clamped) + 0.5, max(depth, 1e-7)), f32(depth > 0.0));
}

// The step across `span` pixels along `axis` from a point, on whichever side lies nearer its depth so an edge behind
// or before it does not tilt it; none where neither side shows anything.
fn surface_step(texel: vec2<i32>, position: vec3<f32>, axis: vec2<i32>, span: i32) -> vec3<f32> {
  let ahead: vec4<f32> = depth_point(texel + axis * span);
  let behind: vec4<f32> = depth_point(texel - axis * span);
  let forward: vec3<f32> = ahead.xyz - position;
  let backward: vec3<f32> = position - behind.xyz;
  let is_forward: bool = ahead.w > 0.5 && (behind.w < 0.5 || abs(forward.z) < abs(backward.z));

  return select(select(vec3<f32>(0.0), backward, behind.w > 0.5), forward, is_forward);
}

// How far a surface faces up in the world, from its depth `SURFACE_REACH` metres either side: its slope over its
// triangles, smooth from one to the next; none where it cannot be told.
fn surface_up(texel: vec2<i32>, position: vec3<f32>) -> f32 {
  // Metres a frame pixel spans at the point's distance.
  let pixel: f32 = 2.0 * -position.z / (camera.projection[1][1] * camera.viewport.y);
  let span: i32 = i32(clamp(SURFACE_REACH / max(pixel, 1e-5), SURFACE_SPAN.x, SURFACE_SPAN.y));
  let facing: vec3<f32> = cross(surface_step(texel, position, vec2<i32>(0, 1), span),
    surface_step(texel, position, vec2<i32>(1, 0), span));

  if (dot(facing, facing) < 1e-12) {
    return 0.0;
  }

  return dot(normalize(facing), camera.view[1].xyz);
}

// The enhanced wetting: the level's wetness glossing what the rain reaches out to the distance, ripples on what faces
// up and water running down what stands near the camera, and puddles on flat, low terrain, rippling. The normal goes
// out packed into `xy`, the gloss the rain adds into `z`, and how much of a puddle the point is into `w`.
@fragment
fn fs_wet_patch_enhanced(in: FullscreenVarying) -> @location(0) vec4<f32> {
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
  let open: f32 = max(rain_reach(world), smoothstep(COVER_FADE.x, COVER_FADE.y, edge));
  let kept: f32 = select(1.0, 0.0, is_flora);
  let gradients: PlaceGradients = PlaceGradients(dpdx(place.xz), dpdy(place.xz));
  let wall_x: PlaceGradients = PlaceGradients(dpdx(place.zy), dpdy(place.zy));
  let wall_z: PlaceGradients = PlaceGradients(dpdx(place.xy), dpdy(place.xy));

  // Ripples on what faces up, water running down what stands, as much as the rain pours and reaches.
  let rippled: vec2<f32> = rain_ripples(place.xz * RIPPLE_SCALE,
    PlaceGradients(gradients.across * RIPPLE_SCALE, gradients.down * RIPPLE_SCALE),
    vec3<f32>(pouring, RIPPLE_STRENGTH * wet.puddles.w, 6.0), distance);
  var weights: vec3<f32> = facing * kept;
  var water: vec3<f32> = vec3<f32>(rippled.x, 0.0, rippled.y) * smoothstep(0.75, 0.8, weights.y);

  weights = vec3<f32>(saturate(abs(weights.x) - 0.15), weights.y, saturate(abs(weights.z) - 0.15));
  weights = vec3<f32>(weights.x * f32(weights.x > weights.z), weights.y, weights.z * f32(weights.x < weights.z));
  water += running_water(place.zy, wall_x).yxz * weights.x + running_water(place.xy, wall_z).zxy * weights.z;
  water *= kept * saturate(pouring) * open;

  // The gloss the wetness gives what the rain reaches, none on what faces down, fading out in the distance; terrain
  // already glossy keeps a puddle's.
  let reached: f32 = saturate(open + f32(gloss > 0.3));
  var rain_gloss: f32 = (0.15 + saturate(pouring) * 0.05) * soaked * kept * reached;

  rain_gloss = select(rain_gloss, 0.15, is_terrain && gloss > 0.3);
  rain_gloss *= saturate(weights.y * 1.5) * smoothstep(WET_REACH.y, WET_REACH.x, length(position));

  // Puddles: noise over level terrain the rain reaches, the steeper the less of it, none over whole dry stretches, as
  // wet as the level, rippled by the rain.
  var puddle: f32 = 0.0;

  if (is_terrain && wetness > 0.0) {
    let flatness: f32 = smoothstep(PUDDLE_FLATNESS.x, PUDDLE_FLATNESS.y, surface_up(texel, position));
    let noise: f32 = textureSampleGrad(puddle_noise, wet_sampler, place.xz / PUDDLE_TILE,
      gradients.across / PUDDLE_TILE, gradients.down / PUDDLE_TILE).r;
    let edge: f32 = textureSampleGrad(puddle_noise, wet_sampler, place.zx / PUDDLE_EDGE_TILE + 0.37,
      gradients.across.yx / PUDDLE_EDGE_TILE, gradients.down.yx / PUDDLE_EDGE_TILE).r;
    let region: f32 = textureSampleGrad(puddle_noise, wet_sampler,
      PUDDLE_REGION_TURN * place.xz / PUDDLE_REGION_TILE, PUDDLE_REGION_TURN * gradients.across / PUDDLE_REGION_TILE,
      PUDDLE_REGION_TURN * gradients.down / PUDDLE_REGION_TILE).r;
    let dry: f32 = 1.0 - smoothstep(PUDDLE_REGION_KEPT.x, PUDDLE_REGION_KEPT.y, region);
    let threshold: f32 = 1.0 - wet.puddles.y * 1.1 + (1.0 - flatness) * PUDDLE_SLOPE_RISE +
      (edge - 0.5) * PUDDLE_EDGE_SHIFT + dry * PUDDLE_REGION_RISE;

    puddle = saturate((noise - threshold) * wetness) * open;
    puddle = smoothstep(0.0, saturate(0.3 - PUDDLE_BORDER * 0.3), puddle);

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
      let surface: vec2<f32> = calm * 0.666 + drops * 0.333;
      let level: vec3<f32> = normalize(vec3<f32>(surface.x, 1.0, -surface.y));

      normal = normalize(mix(normal, (camera.view * vec4<f32>(level, 0.0)).xyz, puddle));
    }
  }

  let bent: vec3<f32> = (camera.view * vec4<f32>(water.x, water.y, -water.z, 0.0)).xyz;

  return vec4<f32>(octahedral_encode(normalize(normal + bent)), rain_gloss, puddle);
}

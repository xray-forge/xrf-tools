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
// Metres within which rain ripples, out of which the cover counts as open, and over which the wetting fades out.
const RIPPLE_REACH: f32 = 15.0;
const COVER_REACH: f32 = 30.0;
const WET_REACH: vec2<f32> = vec2<f32>(200.0, 250.0);
// Ripple layers: how fast each runs, and where the second and third are moved to.
const RIPPLE_LAYER_SPEEDS: vec3<f32> = vec3<f32>(1.05, 1.31, 1.58);
const RIPPLE_LAYER_OFFSETS: vec4<f32> = vec4<f32>(0.5, 0.25, 0.31, 0.5);
// Metres a puddle's noise repeats over, and its ripples' normals; and how flat terrain must lie to gather water.
const PUDDLE_TILE: f32 = 60.0;
const PUDDLE_RIPPLE_TILE: f32 = 12.0;
const PUDDLE_FLATNESS: f32 = 0.997;
// Metres around a point the cover is read at to tell how low it lies against its surroundings, and how strongly; and
// how hard a puddle's border is, from soft to sharp.
const PUDDLE_HOLLOW_REACH: f32 = 1.5;
const PUDDLE_HOLLOW_STRENGTH: f32 = 4.0;
const PUDDLE_BORDER: f32 = 0.7;

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

// Rain ripples at a place, three layers over each other, fading out to `RIPPLE_REACH` metres away.
fn rain_ripples(place: vec2<f32>, setup: vec3<f32>, distance: f32) -> vec2<f32> {
  let near: f32 = RIPPLE_REACH - distance;

  if (near < 0.0) {
    return vec2<f32>(0.0);
  }

  let first: vec4<f32> = textureSampleLevel(ripples, wet_sampler, place, 0.0);
  let second: vec4<f32> = textureSampleLevel(ripples, wet_sampler, place * 0.61 + RIPPLE_LAYER_OFFSETS.xy, 0.0);
  let third: vec4<f32> = textureSampleLevel(ripples, wet_sampler, place * 0.87 + RIPPLE_LAYER_OFFSETS.zw, 0.0);
  let summed: vec2<f32> = ripple_layer(first, vec3<f32>(RIPPLE_LAYER_SPEEDS.x * setup.x, setup.yz)) +
    ripple_layer(second, vec3<f32>(RIPPLE_LAYER_SPEEDS.y * setup.x, setup.yz)) +
    ripple_layer(third, vec3<f32>(RIPPLE_LAYER_SPEEDS.z * setup.x, setup.yz));

  return clamp(summed * near / RIPPLE_REACH, vec2<f32>(-1.0), vec2<f32>(1.0));
}

// Water running down a wall at a place on it: narrow columns, each sliding at its own pace, faster as it pours.
fn running_water(place: vec2<f32>) -> vec3<f32> {
  let scaled: vec2<f32> = place * FALL_SCALE;
  let column: f32 = ceil(scaled.x * FALL_COLUMNS);
  var offset: f32 = hash22(vec2<f32>(column, 1.0)).x;

  offset = select(offset, offset * 3.0, offset < FALL_LEAST_SPEED);
  offset += wet.time * offset * wet.density * FALL_SPEED;

  let uv: vec2<f32> = vec2<f32>((fract(scaled.x * FALL_COLUMNS) + column) / FALL_COLUMNS, scaled.y + offset);

  return (textureSampleLevel(flow, wet_sampler, uv, 0.0).xzy * 2.0 - 1.0) * FALL_STRENGTH;
}

// How low a point lies against the cover around it, from nothing on a crest through a half on flat ground to one in a
// hollow; a half outside the cover.
fn puddle_hollow(world: vec3<f32>) -> f32 {
  let here: f32 = rain_cover_height(cover, wet.window, world);

  if (here < -1e8) {
    return 0.5;
  }

  var around: f32 = 0.0;

  for (var tap: u32 = 0u; tap < 4u; tap++) {
    let angle: f32 = f32(tap) * 1.5707963;
    let column: vec3<f32> = world + vec3<f32>(cos(angle), 0.0, sin(angle)) * PUDDLE_HOLLOW_REACH;

    around += max(rain_cover_height(cover, wet.window, column), here);
  }

  return saturate(0.5 + (around * 0.25 - here) * PUDDLE_HOLLOW_STRENGTH);
}

// The surface's own normal in the world, from the depth around it: what lies under its bumps.
fn surface_normal(clip: vec2<f32>, position: vec3<f32>) -> vec3<f32> {
  let last: vec2<i32> = vec2<i32>(camera.viewport.xy) - 1;
  let right: vec2<i32> = min(vec2<i32>(clip) + vec2<i32>(1, 0), last);
  let below: vec2<i32> = min(vec2<i32>(clip) + vec2<i32>(0, 1), last);
  let across: vec3<f32> = camera_view_position(vec2<f32>(right) + 0.5, textureLoad(depth_target, right, 0)) - position;
  let down: vec3<f32> = camera_view_position(vec2<f32>(below) + 0.5, textureLoad(depth_target, below, 0)) - position;
  let facing: vec3<f32> = normalize(cross(down, across));
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));

  return normalize(rotation * facing);
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
  let is_flora: bool = has_mark(marks, MARK_FLORA);
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
  let open: f32 = rain_open(world);
  let kept: f32 = select(1.0, 0.0, is_flora);

  // Ripples on what faces up, water running down what stands, as much as the rain pours and reaches.
  let rippled: vec2<f32> = rain_ripples(place.xz * RIPPLE_SCALE,
    vec3<f32>(pouring, RIPPLE_STRENGTH * wet.puddles.w, 6.0), distance);
  var weights: vec3<f32> = facing * kept;
  var water: vec3<f32> = vec3<f32>(rippled.x, 0.0, rippled.y) * smoothstep(0.75, 0.8, weights.y);

  weights = vec3<f32>(saturate(abs(weights.x) - 0.15), weights.y, saturate(abs(weights.z) - 0.15));
  weights = vec3<f32>(weights.x * f32(weights.x > weights.z), weights.y, weights.z * f32(weights.x < weights.z));
  water += running_water(place.zy).yxz * weights.x + running_water(place.xy).zxy * weights.z;
  water *= kept * saturate(pouring) * open;

  // The gloss the wetness gives what the rain reaches, out of the cover's reach as if open, none on what faces down,
  // fading out in the distance; terrain already glossy keeps a puddle's.
  let reached: f32 = saturate(open + smoothstep(COVER_REACH - 20.0, COVER_REACH, distance) + f32(gloss > 0.3));
  var rain_gloss: f32 = (0.15 + saturate(pouring) * 0.05) * soaked * kept * reached;

  rain_gloss = select(rain_gloss, 0.15, is_terrain && gloss > 0.3);
  rain_gloss *= saturate(weights.y * 1.5) * smoothstep(WET_REACH.y, WET_REACH.x, length(position));

  // Puddles: noise over flat terrain, more in hollows, as wet as the level, rippled by the rain.
  var puddle: f32 = 0.0;

  if (is_terrain && wetness > 0.0) {
    let ground: vec3<f32> = surface_normal(in.clip.xy, position);
    let flatness: f32 = saturate((ground.y - PUDDLE_FLATNESS) * (102.0 + PUDDLE_FLATNESS));
    let noise: f32 = textureSampleLevel(puddle_noise, wet_sampler, place.xz / PUDDLE_TILE, 0.0).r;

    puddle = saturate((noise - (1.0 - wet.puddles.y * 1.1)) * wetness) * flatness * puddle_hollow(world);
    puddle = smoothstep(0.0, saturate(0.3 - PUDDLE_BORDER * 0.3), puddle);

    if (puddle > 0.0) {
      let pace: f32 = wet.time * (0.01 + saturate(wet.density * 1.5) * 0.008);
      let uv: vec2<f32> = place.xz / PUDDLE_RIPPLE_TILE;
      let waves: vec3<f32> = textureSampleLevel(puddle_normal, wet_sampler, uv + vec2<f32>(0.0, pace), 0.0).xyz +
        textureSampleLevel(puddle_normal, wet_sampler, uv - vec2<f32>(0.33, pace), 0.0).xyz - 1.0;
      let drops: vec2<f32> = rain_ripples(uv * 1.3, vec3<f32>(0.85, 1.0, 10.0), distance) * wet.density * 3.0 *
        wet.puddles.w;
      let calm: vec2<f32> = waves.xy * saturate(0.05 + saturate(wet.density * 1.5) * 0.1);
      let surface: vec2<f32> = calm * 0.666 + drops * 0.333;
      let level: vec3<f32> = normalize(vec3<f32>(surface.x, 1.0, -surface.y));

      normal = normalize(mix(normal, (camera.view * vec4<f32>(level, 0.0)).xyz, puddle));
    }
  }

  let bent: vec3<f32> = (camera.view * vec4<f32>(water.x, water.y, -water.z, 0.0)).xyz;

  return vec4<f32>(octahedral_encode(normalize(normal + bent)), rain_gloss, puddle);
}

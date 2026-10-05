enable wgpu_binding_array;

#import "common/camera"
#import "common/lighting"
#import "common/sky_box"
#import "static/pulling"
#import "generated/static/water"

// Water as OpenXRay's `water.vs`, `water.ps` and `waterd.ps` draw it, and Anomaly's programs over them: the wave
// lifting the surface, two scrolling normal layers bending the sky's reflection, the fresnel mixing it with the base by
// the base's alpha, lit per vertex by the hemisphere, the sun and the ambient. Soft water fades by the depth behind it,
// darkens with it towards `water_intensity`, lays foam in the shallows and is fogged. Drawn over the lit frame as it
// shows, which `water.ps` writes without the tonemap, and the distortion it causes into a target of its own.

@group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var texture_sampler: sampler;

// Group 3, the water's own, is declared from its passes' parameters.

// `watermove`: the wave's direction through the level, in renderer space, where the engine's `z` is negated.
const WAVE_DIRECTION: vec3<f32> = vec3<f32>(0.11, 0.13, -0.07);

// `watermove_tc`: the scroll's direction across the level, over the engine's `x` and `z`.
const SCROLL_DIRECTION: vec2<f32> = vec2<f32>(0.2111, 0.2333);

// `W_DISTORT_BASE_TILE_0` and `_1`, and `W_DISTORT_AMP_0` and `_1` (`shared/waterconfig.h`).
const LAYER_TILES: vec2<f32> = vec2<f32>(1.0, 1.1);
const LAYER_AMPLITUDES: vec2<f32> = vec2<f32>(0.15, 0.55);

// What the depth behind the water reads where nothing was drawn: farther than any water is deep.
const FAR_BEHIND: f32 = 1e6;

// The enhanced water's `Wave_Int`, across and down, at its calmest and in the strongest wind: how much of its waves'
// normal moves what lies under it, a share of the screen; its refraction strength scales both.
const WAVES_CALM: vec2<f32> = vec2<f32>(0.03, 0.1);
const WAVES_WINDY: vec2<f32> = vec2<f32>(0.1, 0.2);

// `G_SSR_WATER_SPECULAR_NORMAL`: how much of its waves' height the sun's highlight sees.
const SPECULAR_NORMAL: f32 = 0.2;

// The parallax at the module's default quality (`q_parallax[2]`): its most steps, looking along the surface, and the
// distance over which it falls to a single step (`G_SSR_PARALLAX_DISTANCE`), from three fifths of it.
const PARALLAX_STEPS: f32 = 16.0;
const PARALLAX_DISTANCE: f32 = 25.0;

// How many times the enhanced water tiles the base coordinate (`ssfx_water.vs`: `v.uv * 1.5`).
const ENHANCED_TILING: f32 = 1.5;

// Its variation: the second layer tiled by the golden ratio's inverse, so the two layers never repeat together, and a
// broad layer tiled about seven times as large, faded in over these metres to this share where a repeat shows most.
const SECOND_TILING: f32 = 0.618034;
const BROAD_TILING: f32 = 0.137;
const BROAD_NEAR: f32 = 5.0;
const BROAD_FAR: f32 = 60.0;
const BROAD_SHARE: f32 = 0.6;

// Its colour's second read: turned 37 degrees and tiled 0.382 times as finely, so its repeat never meets the first's,
// mixed in by a perlin noise over the level about 30 m across.
const COLOUR_TURN: mat2x2<f32> = mat2x2<f32>(0.798636, 0.601815, -0.601815, 0.798636);
const COLOUR_TILING: f32 = 0.382;
const COLOUR_NOISE: f32 = 0.035;
const COLOUR_BEND: vec2<f32> = vec2<f32>(1.3, -0.9);

// `ssfx_rain_ripples`: how fast each of its three layers ripples, where the second and third are moved to, and how far
// the ripples reach.
const RIPPLE_SPEEDS: vec3<f32> = vec3<f32>(1.05, 1.31, 1.58);
const RIPPLE_OFFSETS: vec4<f32> = vec4<f32>(0.5, 0.25, 0.31, 0.5);
const RIPPLE_REACH: f32 = 15.0;

// `G_SSR_WATER_FOG_MAXDEPTH`: the water's fog at which it is its colour alone.
const ENHANCED_FOG_DEPTH: f32 = 2.0;

// `G_SSR_WATER_REFLECTION_VIBRANCE`: how much of its colour the reflection keeps, and the luminance it greys towards.
const ENHANCED_VIBRANCE: f32 = 0.6;
const LUMINANCE: vec3<f32> = vec3<f32>(0.3, 0.38, 0.22);

// The reflection's march at the module's default quality (`q_steps[1]`): its steps, its refinements of a hit, how thick
// a surface it takes for one, and how far it reaches.
const MARCH_STEPS: i32 = 16;
const MARCH_REFINES: i32 = 2;
const MARCH_THICKNESS: f32 = 3.0;
const MARCH_REACH: f32 = 150.0;

// The flat surface's fresnel, cubed and by the reflectivity, below which the surface would show too little of a hit to
// march for it: looking that steeply down, the waves' normals cannot raise it into sight.
const MARCH_LEAST_SHOWN: f32 = 0.002;

// How much of the reflection the last frames keep, and how much less of it where the ray met the sky.
const REFLECTION_HISTORY: f32 = 0.97;
const REFLECTION_SKY_HISTORY: f32 = 0.2;

struct WaterVarying {
  // Invariant, so the depth pass and the surface pass place each surface alike.
  @builtin(position) @invariant clip: vec4<f32>,
  // In renderer space, lifted by the wave.
  @location(0) world: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) tangent: vec3<f32>,
  @location(3) binormal: vec3<f32>,
  @location(4) uv: vec2<f32>,
  // The vertex's baked light, then its sun occlusion.
  @location(5) baked: vec4<f32>,
  @location(6) hemi: f32,
  @location(7) @interpolate(flat) surface: u32,
};

struct WaterOutput {
  @location(0) color: vec4<f32>,
  // How far the water moves what is seen through it, around a half, blended in at a half.
  @location(1) distortion: vec4<f32>,
};

@vertex
fn vs_water(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> WaterVarying {
  let pulled: PulledVertex = pull(vertex_index, instance_index);
  let at: u32 = pulled.word;
  let place: Place = pulled.place;
  let matrix: mat4x4<f32> = place_matrix(place);
  let linear: mat3x3<f32> = mat3x3<f32>(place.m0.xyz, place.m1.xyz, place.m2.xyz);
  let scale: vec3<f32> = vec3<f32>(dot(place.m0.xyz, place.m0.xyz), dot(place.m1.xyz, place.m1.xyz),
    dot(place.m2.xyz, place.m2.xyz));
  let binormal: vec4<f32> = unpack4x8unorm(words[at]);
  let normal: vec4<f32> = unpack4x8unorm(words[at + 1u]);
  let tangent: vec4<f32> = unpack4x8unorm(words[at + 2u]);
  let position: vec3<f32> = vec3<f32>(bitcast<f32>(words[at + 5u]), bitcast<f32>(words[at + 6u]),
    bitcast<f32>(words[at + 7u]));
  // `D3DCOLOR`: blue, green, red and the sun's share.
  let color: vec4<f32> = unpack4x8unorm(words[at + 4u]);
  var world: vec3<f32> = (matrix * vec4<f32>(position, 1.0)).xyz;
  let phase: f32 = water.time + dot(world, WAVE_DIRECTION * water.wave_speed);

  world.y += sin(phase) * water.wave_height;

  var out: WaterVarying;

  out.clip = camera.view_projection * vec4<f32>(world, 1.0);
  out.world = world;
  out.normal = normalize(linear * (unpack_direction(normal) / scale));
  out.tangent = linear * unpack_direction(tangent);
  out.binormal = linear * unpack_direction(binormal);
  // The base coordinate's fraction rides in the tangent's and binormal's fourth bytes.
  out.uv = (unpack_shorts(words[at + 3u]) + vec2<f32>(tangent.w, binormal.w)) / 1024.0;
  out.baked = vec4<f32>(color.zyx, color.w);
  out.hemi = normal.w * place.info.x + place.info.y;
  out.surface = pulled.surface;

  return out;
}

// One normal layer's coordinates: the base's, tiled, scrolled around a circle by `timers.z`, `watermove_tc`.
fn scrolled(base: vec2<f32>, world: vec3<f32>, tile: f32, amplitude: f32) -> vec2<f32> {
  let angle: f32 = water.time / 10.0 + dot(vec2<f32>(world.x, -world.z), SCROLL_DIRECTION * amplitude);

  return base * tile + vec2<f32>(sin(angle), cos(angle)) * amplitude * water.ripple;
}

// Both keyframes' skies along a world direction, as water reflecting them reads the cubes: unturned by the sky's
// rotation, which turns only the sky box.
fn sky_cubes(direction: vec3<f32>) -> vec3<f32> {
  let lookup: vec3<f32> = cube_lookup(direction);

  return mix(
    textureSampleLevel(sky_cube_0, sky_clamp, lookup, 0.0).rgb,
    textureSampleLevel(sky_cube_1, sky_clamp, lookup, 0.0).rgb,
    lighting.sky.w,
  );
}

// Screen Space Shaders' `SSFX_calc_sky`: the sky turned as its box is, raised to fill a reflection's perspective, both
// keyframes' cubes tinted by the sky's colour, as the module reads it with Anomaly's own executable.
fn enhanced_sky(direction: vec3<f32>) -> vec3<f32> {
  var turned: vec3<f32> = sky_box_direction(direction, lighting.sky_params.x);

  turned.y = (turned.y - max(cos(turned.x) * 0.65, cos(turned.z) * 0.65)) * 2.1 + 0.35;

  let cubes: vec3<f32> = mix(
    textureSampleLevel(sky_cube_0, sky_clamp, turned, 0.0).rgb,
    textureSampleLevel(sky_cube_1, sky_clamp, turned, 0.0).rgb,
    lighting.sky.w,
  );

  return saturate(lighting.sky.rgb) * cubes;
}

fn sample_slot(slot: u32, uv: vec2<f32>) -> vec4<f32> {
  return textureSample(textures[slot], texture_sampler, uv);
}

// What both waters take of a fragment: its surface's textures, its normals, the eye, the fog, the sky it reflects, the
// light its vertices carry and the depth behind it.
struct WaterFragment {
  flags: u32,
  base: vec4<f32>,
  bent: vec3<f32>,
  foam: vec4<f32>,
  offset_texel: vec2<f32>,
  normal: vec3<f32>,
  surface_normal: vec3<f32>,
  to_point: vec3<f32>,
  // In view space.
  position: vec3<f32>,
  fog: f32,
  reflected: vec3<f32>,
  remapped: vec3<f32>,
  light: vec3<f32>,
  // The G-buffer's depth under the fragment, and how far past the surface it lies along the view.
  stored: f32,
  depth: f32,
};

// Samples a fragment's surface and works out what both waters read of it, discarding where the fog is total and where a
// nearer fold of the surface covers it.
fn read_water(in: WaterVarying) -> WaterFragment {
  let surface: Surface = surfaces[in.surface];
  let flags: u32 = surface.flags;
  let first: vec2<f32> = scrolled(in.uv, in.world, LAYER_TILES.x, LAYER_AMPLITUDES.x);
  let second: vec2<f32> = scrolled(in.uv, in.world, LAYER_TILES.y, LAYER_AMPLITUDES.y);
  // Sampled before any branch, where every derivative is taken alike; a texture the surface binds none of is ignored.
  let sampled: vec4<f32> = sample_slot(surface.base, in.uv);
  let normals: vec3<f32> = sample_slot(surface.detail, first).xyz + sample_slot(surface.detail, second).xyz - 1.0;
  let foam_texel: vec4<f32> = sample_slot(surface.bump, in.uv);
  let distorted: vec2<f32> = (sample_slot(surface.bump_companion, first).xy +
    sample_slot(surface.bump_companion, second).xy) * 0.5;
  let is_textured: f32 = camera.switches.x;
  var out: WaterFragment;

  out.flags = flags;
  out.base = vec4<f32>(mix(untextured_color(surface.color), sampled.rgb, is_textured), sampled.a);
  out.bent = select(vec3<f32>(0.0, 0.0, 1.0), normals, (flags & SURFACE_HAS_WATER_NORMAL) != 0u);
  out.foam = select(vec4<f32>(0.0), foam_texel, (flags & SURFACE_HAS_FOAM) != 0u);
  out.offset_texel = select(vec2<f32>(0.5), distorted, (flags & SURFACE_HAS_DISTORTION) != 0u);
  out.normal = normalize(in.normal);
  out.surface_normal = normalize(in.tangent * out.bent.x + in.binormal * out.bent.y + out.normal * out.bent.z);
  out.to_point = normalize(in.world - camera.position.xyz);
  out.position = (camera.view * vec4<f32>(in.world, 1.0)).xyz;
  out.fog = select(0.0, fog_amount(lighting, out.position), lighting.params.y > 0.5);

  // The far plane ends where fog is total, so nothing past it is drawn but the sky; a fold of the surface behind a
  // nearer one is not drawn either, whatever order the two are drawn in.
  if (out.fog >= 1.0 || in.clip.z < textureLoad(nearest_water, vec2<i32>(in.clip.xy), 0)) {
    discard;
  }

  // `vreflect.y = vreflect.y * 2 - 1`, the fake remapping of the DirectX 11 programs, which the fresnel reads as well.
  out.reflected = reflect(out.to_point, out.surface_normal);
  out.remapped = vec3<f32>(out.reflected.x, out.reflected.y * 2.0 - 1.0, out.reflected.z);

  // `c0`: the vertex's baked light, the hemisphere by its occlusion, the sun by its sun occlusion, and the ambient, as
  // `L_hemi_color`, `L_sun_color` and `L_ambient` bind them raw: the weather's own, which the console's light scales
  // and combine's doubling and floor never reach.
  let normal_view: vec3<f32> = (camera.view * vec4<f32>(out.normal, 0.0)).xyz;

  out.light = in.baked.rgb + lighting.forward_hemi.rgb * (0.5 + out.normal.y * 0.5) * in.hemi +
    lighting.forward_sun.rgb * dot(normal_view, lighting.to_sun.xyz) * in.baked.a + lighting.forward_ambient.rgb;
  out.stored = textureLoad(depth_target, vec2<i32>(in.clip.xy), 0);

  let behind: f32 = select(-camera_view_position(in.clip.xy, out.stored).z, FAR_BEHIND, out.stored <= 0.0);

  out.depth = behind + out.position.z;

  return out;
}

// `waterd.ps`: the distortion map at the normal layers' coordinates, gone where the base is opaque, faded by the depth
// behind soft water, then halved around nothing; blended in at nothing while the water does not distort.
fn write_distortion(fragment: WaterFragment) -> vec4<f32> {
  let is_soft: bool = (fragment.flags & SURFACE_IS_SOFT_WATER) != 0u;
  let opaque: vec2<f32> = mix(fragment.offset_texel, vec2<f32>(0.5), fragment.base.a);
  let shoal: vec2<f32> = mix(vec2<f32>(0.5), opaque, saturate(fragment.depth * 5.0));
  let offset: vec2<f32> = select(opaque, mix(opaque, shoal, water.soft), is_soft);

  return vec4<f32>(offset * 0.5 + 0.25, select(0.08, 0.0, is_soft), 0.5 * water.distorted);
}

@fragment
fn fs_water(in: WaterVarying) -> WaterOutput {
  let fragment: WaterFragment = read_water(in);
  let flags: u32 = fragment.flags;
  let base: vec4<f32> = fragment.base;
  let reflected: vec3<f32> = fragment.reflected;
  let to_point: vec3<f32> = fragment.to_point;
  let light: vec3<f32> = fragment.light;
  var lit: vec3<f32>;
  var plain_alpha: f32;

  if ((flags & SURFACE_IS_ANOMALY_WATER) != 0u) {
    // Anomaly's `water.ps`: the whole sky mixed with the base by its alpha, the sun's highlight over it, by the
    // switches its program defines; its fresnel is taken of the reflection before the sky's remapping.
    let power: f32 = pow(saturate(dot(reflected, to_point)), 9.0);
    let sky: vec3<f32> = sky_cubes(vec3<f32>(reflected.x, reflected.y * 2.0 - 1.0, reflected.z)) * water.reflection;
    let albedo: vec3<f32> = select(base.rgb, base.rgb * light, (flags & SURFACE_IS_TRANSPARENT) != 0u);
    var color: vec3<f32> = select(albedo, mix(sky, albedo, base.a), (flags & SURFACE_IS_REFLECTING) != 0u);

    if ((flags & SURFACE_IS_SPECULAR) != 0u) {
      // `specular_phong(v2point, Nw, L_sun_dir_w) * 4`.
      let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
      let sun: vec3<f32> = -(rotation * lighting.to_sun.xyz);

      color += lighting.sun.rgb * pow(abs(dot(normalize(to_point + sun), fragment.surface_normal)), 256.0) * 4.0;
    }

    lit = color * light * 2.0;
    plain_alpha = 0.55 + power * 0.25;
  } else {
    // OpenXRay's `water.ps`: the sky squared and doubled, a share of it by the fresnel, mixed with the base by its
    // alpha.
    let power: f32 = pow(saturate(dot(fragment.remapped, to_point)), 9.0);
    let sky: vec3<f32> = sky_cubes(fragment.remapped);
    let amount: f32 = (0.15 + power * 0.25) * water.reflection;

    lit = mix(sky * sky * 2.0 * amount, base.rgb, base.a) * light * 2.0;
    plain_alpha = 0.75 + power * 0.25;
  }

  // `NEED_SOFT_WATER` and `USE_SOFT_WATER`: the depth behind the surface, which fades, darkens and foams it.
  let depth: f32 = fragment.depth;
  let deepened: vec3<f32> = mix(vec3<f32>(water.intensity * 0.1), lit, plain_alpha);
  let faded: f32 = max(1.0 - exp(depth * -4.0), min(plain_alpha, saturate(depth)));
  let shallow: f32 = saturate(-depth * dot(fragment.normal, to_point));
  let is_foamed: bool = (flags & SURFACE_IS_ANOMALY_WATER) == 0u || (flags & SURFACE_IS_FOAMED) != 0u;
  let foamed: f32 = smoothstep(0.025, 0.05, shallow) * (1.0 - smoothstep(0.075, 0.1, shallow)) * fragment.foam.a *
    camera.switches.x * f32(is_foamed);
  let soft_color: vec3<f32> = mix(mix(deepened, fragment.foam.rgb * water.intensity, foamed), lit, 1.0 - water.soft);
  let soft_alpha: f32 = mix(mix(faded, fragment.foam.a, foamed), plain_alpha, 1.0 - water.soft);
  let is_soft: bool = (flags & SURFACE_IS_SOFT_WATER) != 0u;
  let color: vec3<f32> = select(lit, soft_color, is_soft);
  let seen: f32 = 1.0 - fragment.fog;
  // Plain `water` is written whole, `blend(false)`; soft water is faded by the fog twice over, as its alpha is.
  let alpha: f32 = select(1.0, soft_alpha * seen * seen, is_soft);
  let finished: vec3<f32> = mix(color, lighting.fog_color.rgb, fragment.fog);
  var out: WaterOutput;

  out.color = vec4<f32>(select(base.rgb, finished, lighting.params.y > 0.5), alpha);
  out.distortion = write_distortion(fragment);

  return out;
}

// The enhanced water's waves (`ssfx_water.ps`): its normal map scrolled twice over coordinates its parallax moves
// into its waves' height, the wind's layer leaning the first and the rain's ripples over both, at a pace and a strength
// the weather's wind sets; how far they move what lies under the water across the screen, the surface's normal in the
// world, the height its parallax met, and the first layer's tilt (`Waves_Normal.xy`), which moves where its colour is
// read.
struct EnhancedWaves {
  screen: vec2<f32>,
  normal: vec3<f32>,
  height: f32,
  tilt: vec2<f32>,
};

// The waves' height at coordinates, as the module's `ssfx_water_waves.ps` lays it into a target its parallax reads:
// two scrolls of the height map, the higher. Sampled here instead, which keeps the target and its pass out.
fn wave_height(coordinates: vec2<f32>) -> f32 {
  let at: vec2<f32> = coordinates * 0.35;
  let first: f32 = textureSampleLevel(height_map, texture_sampler, at + water.heights * vec2<f32>(0.065, 0.445), 0.0).r;
  let second: f32 = textureSampleLevel(height_map, texture_sampler, at - water.heights * vec2<f32>(0.105, 0.241), 0.0).b;

  return max(first, second);
}

// `Water_DoParallax`: steps along the eye's way across the surface until the waves' height rises over it, then the
// coordinates between the last two steps where it met them, and the height there.
fn march_parallax(start: vec2<f32>, step: vec2<f32>, share: f32) -> vec3<f32> {
  var at: vec2<f32> = start;
  var depth: f32 = 0.0;
  var height: f32 = 0.0;
  var last_height: f32 = 0.0;
  var sampled: f32 = 0.0;

  for (var taken: i32 = 0; taken <= i32(PARALLAX_STEPS); taken++) {
    last_height = height;
    at -= step;
    depth += share;
    sampled = wave_height(at);
    height = 1.0 - sampled;

    if (height <= depth) {
      break;
    }
  }

  let left: f32 = height - depth;
  let ratio: f32 = left / (left - saturate(last_height - depth + share));

  return vec3<f32>(mix(at, at + step, ratio), sampled);
}

// `ssfx_process_ripples`: one layer of rain ripples, each texel a ring that spreads and fades on its own clock.
fn ripple_layer(ripple: vec4<f32>, speed: f32) -> vec2<f32> {
  let clock: f32 = fract(ripple.w + water.time * speed);
  let spread: f32 = clamp((clock - 1.0 + ripple.x) * RIPPLE_REACH, 0.0, 4.0);
  let factor: f32 = saturate(0.7 - clock) * ripple.x * sin(spread * 3.141592) * (1.0 - smoothstep(0.0, 4.0, spread));

  return (ripple.yz * 2.0 - 1.0) * factor * water.ripples;
}

// `ssfx_rain_ripples`: three layers of ripples over the surface, fading out over fifteen metres of the bottom's distance.
fn rain_ripples(coordinates: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>, rain: f32, distance: f32) -> vec2<f32> {
  let first: vec4<f32> = textureSampleGrad(ripple_map, texture_sampler, coordinates, dx, dy);
  let second: vec4<f32> = textureSampleGrad(ripple_map, texture_sampler, coordinates * 0.61 + RIPPLE_OFFSETS.xy, dx * 0.61,
    dy * 0.61);
  let third: vec4<f32> = textureSampleGrad(ripple_map, texture_sampler, coordinates * 0.87 + RIPPLE_OFFSETS.zw, dx * 0.87,
    dy * 0.87);
  let ripples: vec2<f32> = ripple_layer(first, RIPPLE_SPEEDS.x * rain) + ripple_layer(second, RIPPLE_SPEEDS.y * rain) +
    ripple_layer(third, RIPPLE_SPEEDS.z * rain);

  return clamp(ripples * max(RIPPLE_REACH - distance, 0.0) * 0.0666, vec2<f32>(-1.0), vec2<f32>(1.0));
}

// Its maps are read with the base coordinates' own gradients, which the parallax's march, ending where the height
// says, must not decide.
fn enhanced_waves(in: WaterVarying, bottom_distance: f32) -> EnhancedWaves {
  let wind: f32 = saturate(water.wind_velocity * 0.001);
  let base: vec2<f32> = in.uv * ENHANCED_TILING;
  let dx: vec2<f32> = dpdx(base);
  let dy: vec2<f32> = dpdy(base);
  var coordinates: vec2<f32> = base + water.waves * vec2<f32>(0.065, 0.445);
  var height: f32 = 0.0;
  let distance: f32 = -(camera.view * vec4<f32>(in.world, 1.0)).z;

  // The parallax, while the waves stand: steps by how closely the eye looks along the surface, one alone past its
  // distance.
  if (water.parallax_height > 0.0) {
    let to_eye: vec3<f32> = normalize(camera.position.xyz - in.world);
    let eye: vec3<f32> = normalize(vec3<f32>(
      dot(to_eye, normalize(in.tangent)),
      dot(to_eye, normalize(in.binormal)),
      dot(to_eye, normalize(in.normal)),
    ));
    let is_near: bool = distance < PARALLAX_DISTANCE;
    let share: f32 = select(1.0, 1.0 / mix(PARALLAX_STEPS, 1.0, abs(eye.z)), is_near);
    let lift: f32 = clamp(water.parallax_height * wind, 0.015, water.parallax_height);
    let marched: vec3<f32> = march_parallax(base, share * eye.xy / eye.z * lift, share);

    coordinates = marched.xy;
    height = marched.z;
  }

  var first: vec3<f32> = textureSampleGrad(wave_map, texture_sampler, coordinates + water.waves * vec2<f32>(0.23, 0.1), dx,
    dy).rgb;
  let apart: f32 = mix(1.0, SECOND_TILING, water.variation);
  var second: vec3<f32> = textureSampleGrad(wave_map, texture_sampler,
    coordinates * apart - water.waves * vec2<f32>(0.21, 0.28), dx * apart, dy * apart).rgb;
  let broad_share: f32 = water.variation * smoothstep(BROAD_NEAR, BROAD_FAR, distance) * BROAD_SHARE;

  if (broad_share > 0.0) {
    let broad: vec3<f32> = textureSampleGrad(wave_map, texture_sampler,
      coordinates * BROAD_TILING + water.waves * vec2<f32>(0.03, 0.07), dx * BROAD_TILING, dy * BROAD_TILING).rgb;

    first = mix(first, broad, broad_share);
    second = mix(second, broad, broad_share);
  }

  let gust: vec2<f32> = textureSampleGrad(wind_map, texture_sampler, coordinates * 0.1 + vec2<f32>(water.gusts_x, water.gusts_y) * 0.1,
    dx * 0.1, dy * 0.1).rg;

  first = vec3<f32>(mix(first.xy, gust, 0.1 * wind), first.z);

  var ripples: vec2<f32> = vec2<f32>(0.0);

  // The rain's ripples, while it rains and they are wanted, at `clamp(rain_params.x * 1.6, 0.65, 1)`.
  if (water.rain > 0.0 && water.ripples > 0.0) {
    ripples = rain_ripples(coordinates * 0.6, dx * 0.6, dy * 0.6, clamp(water.rain * 1.6, 0.65, 1.0), bottom_distance);
  }

  var screen: vec3<f32> = first * 2.0 - 1.0;

  screen = vec3<f32>(screen.xy + vec2<f32>(0.095, 0.088), screen.z);

  var average: vec3<f32> = (first + second) - 1.0 + 0.1 + vec3<f32>(ripples, 0.0);

  average = vec3<f32>(average.xy * 0.5, average.z);

  var out: EnhancedWaves;

  // `clamp(Wave_Int * wind, ...)`: the strongest wind's strength by the wind, never under the calm's.
  let strength: vec2<f32> = clamp(WAVES_WINDY * wind, WAVES_CALM, WAVES_WINDY);

  out.screen = normalize(vec3<f32>(screen.xy * strength * water.refraction, screen.z)).xy + ripples;
  out.normal = normalize(in.tangent * average.x + in.binormal * average.y + normalize(in.normal) * average.z);
  out.height = height;
  out.tilt = screen.xy;

  return out;
}

// The light the enhanced water gathers onto its bottom (`ssfx_water.ps`): two scrolls of its caustics map over the
// bottom's place in the level, the least of the two, sampled before the fragment can be discarded.
fn enhanced_caustics(bottom: vec3<f32>) -> vec3<f32> {
  let place: vec2<f32> = vec2<f32>(bottom.x, -bottom.z);
  let time: f32 = water.flowed;
  let first: vec3<f32> = saturate(textureSample(caustics_map, texture_sampler, place * 0.19 + vec2<f32>(time * 0.1, 0.0)).rgb -
    0.1);
  let second: vec3<f32> = saturate(textureSample(caustics_map, texture_sampler, place * 0.11 + vec2<f32>(-time * 0.07, 0.2))
    .rgb - 0.1);

  return min(first, second);
}

// Screen Space Shaders' `ssfx_water.ps`: what lies under the water, read where its waves move it unless something
// nearer than the water stands there, clouded into the water's colour by the water's depth, its reflection over it by
// the fresnel cubed, the light it gathers on its bottom and the sun's highlight where the sun reaches it, fogged, and
// faded into what lies under it along its shallow edge.
@fragment
fn fs_water_enhanced(in: WaterVarying) -> WaterOutput {
  let size: vec2<f32> = camera.viewport.xy;
  let pixel: vec2<f32> = floor(in.clip.xy);
  // The bottom under the fragment, in the world; far below where only the sky is.
  let stored: f32 = textureLoad(depth_target, vec2<i32>(pixel), 0);
  let waves: EnhancedWaves = enhanced_waves(in, select(scene_distance((pixel + 0.5) / size), FAR_BEHIND, stored <= 0.0));
  let ndc: vec2<f32> = vec2<f32>(in.clip.x / size.x * 2.0 - 1.0, 1.0 - in.clip.y / size.y * 2.0);
  let bottom: vec3<f32> = select(camera_unproject(ndc, stored), in.world - vec3<f32>(0.0, FAR_BEHIND, 0.0), stored <= 0.0);
  let gathered: vec3<f32> = enhanced_caustics(bottom);
  let fragment: WaterFragment = read_water(in);
  let moved: vec2<f32> = clamp(pixel + waves.screen * size, vec2<f32>(0.0), size - 1.0);
  let moved_stored: f32 = textureLoad(depth_target, vec2<i32>(moved), 0);
  let moved_behind: f32 = select(-camera_view_position(moved, moved_stored).z, FAR_BEHIND, moved_stored <= 0.0);
  // `Refraction_Discard`: a read nearer than the water would copy what stands over it into it, and the sky, which the
  // module's position target holds at nothing, is refused as well.
  let refracted: vec2<f32> = select(pixel, moved, moved_stored > 0.0 && moved_behind > -fragment.position.z);
  let screen: vec3<f32> = textureLoad(water_scene, vec2<i32>(refracted), 0).rgb;
  // How deep the water stands over its bottom, straight down, as the module measures its fog and its border.
  let water_depth: f32 = in.world.y - bottom.y;
  // The waves' height its parallax met raises the fog a little.
  let water_fog: f32 = exp(min(water_depth + waves.height * 0.1, 16.0)) - 1.0;
  // `smoothstep(G_SSR_WATER_FOG_MAXDEPTH, -turbidity, fog)`: clear in the shallows, its colour where deep.
  let clear: f32 = saturate((water_fog - ENHANCED_FOG_DEPTH) / (-water.turbidity - ENHANCED_FOG_DEPTH));
  let colour: vec3<f32> = mix(untextured_color(surfaces[in.surface].color), enhanced_colour(in, waves.tilt),
    camera.switches.x) * fragment.light;
  let turbid: vec3<f32> = mix(colour, screen, clear * clear * (3.0 - 2.0 * clear));
  let reflected: vec3<f32> = reflect(fragment.to_point, waves.normal);
  let fresnel: f32 = pow(saturate(dot(reflected, fragment.to_point)), 3.0);
  let reflection: vec3<f32> = enhanced_reflection(waves, reflected, in.world, (pixel + 0.5) / size);
  // Where the lights reached the bottom the sun shines on it, as the module reads its accumulator.
  let sunlit: f32 = saturate(textureLoad(light_target, vec2<i32>(refracted), 0).r * 2000.0);
  let sun: f32 = dot(lighting.sun.rgb, vec3<f32>(0.5));
  // `smoothstep(G_SSR_WATER_FOG_MAXDEPTH + .5, 0, fog)`: the gathered light fades as the water clouds.
  let gathering: f32 = 1.0 - smoothstep(0.0, ENHANCED_FOG_DEPTH + 0.5, water_fog);
  let caustics: vec3<f32> = gathered * sunlit * smoothstep(0.3, 1.0, sun) * water.caustics * gathering *
    saturate(water_fog * 3.0);
  // `L_sun_dir_w`: the way the sunlight travels, in the world.
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let sunlight: vec3<f32> = -(rotation * lighting.to_sun.xyz);
  let flattened: vec3<f32> = normalize(vec3<f32>(waves.normal.x, waves.normal.y * SPECULAR_NORMAL, waves.normal.z));
  let highlight: vec3<f32> = lighting.sun.rgb * pow(abs(dot(normalize(fragment.to_point + sunlight), flattened)), 512.0) *
    saturate(sun) * water.specular * sunlit;
  let seen: f32 = 1.0 - fragment.fog;
  let lit: vec3<f32> = mix(turbid, reflection, saturate(fresnel * water.reflectivity)) + caustics + highlight;
  let fogged: vec3<f32> = mix(lighting.fog_color.rgb, lit, seen);
  let border: f32 = smoothstep(0.0, max(water.soft_border, 1e-4), water_depth + fresnel);
  let shown: vec3<f32> = mix(screen, fogged, border * seen * seen);
  var out: WaterOutput;

  out.color = vec4<f32>(
    select(fragment.base.rgb, shown, lighting.params.y > 0.5),
    saturate(saturate(water_depth - 0.1) * 10.0),
  );
  out.distortion = write_distortion(fragment);

  return out;
}

// `base_tex`: the enhanced water's colour read over the level ten metres a repeat, moved by its waves' tilt, not over
// its coordinates; with its variation, a second read turned and tiled apart mixed in by a noise, so no repeat shows.
fn enhanced_colour(in: WaterVarying, tilt: vec2<f32>) -> vec3<f32> {
  let level: vec2<f32> = vec2<f32>(in.world.x, -in.world.z);
  let slot: u32 = surfaces[in.surface].base;
  let noise: f32 = textureSample(perlin_map, texture_sampler, level * COLOUR_NOISE).r;
  // The noise bends the first read's repeat as well, by up to a repeat's half.
  let bent: vec2<f32> = (noise - 0.5) * COLOUR_BEND * water.variation;
  let first: vec3<f32> = sample_slot(slot, level * 0.1 + tilt + bent).rgb;
  let second: vec3<f32> = sample_slot(slot, COLOUR_TURN * level * (0.1 * COLOUR_TILING) + tilt).rgb;

  return mix(first, second, saturate(noise) * water.variation);
}

// What the enhanced water reflects: its reflection drawn this frame, blurred and clear mixed by a perlin noise over the
// level and greyed a little, read where its waves move it; the sky alone while it draws none.
fn enhanced_reflection(waves: EnhancedWaves, reflected: vec3<f32>, world: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
  if (water.reflected < 0.5) {
    return enhanced_sky(reflected) * water.reflection;
  }

  // The module reads its blurred reflection from the corner of a target the screen's size, where its waves move it
  // twice as far.
  let blurred: vec3<f32> = textureSampleLevel(reflection_blurred, sky_clamp, saturate(uv + waves.screen * 2.0), 0.0).rgb;
  let clear: vec3<f32> = textureSampleLevel(reflection_clear, sky_clamp, saturate(uv + waves.screen), 0.0).rgb;
  let level: vec2<f32> = vec2<f32>(world.x, -world.z) + waves.normal.xy * 100.0;
  let noise: f32 = textureSampleLevel(perlin_map, texture_sampler, level * 0.02, 0.0).r;
  let mixed: vec3<f32> = mix(blurred, clear, saturate(noise * water.blur_noise));

  return mix(vec3<f32>(dot(mixed, LUMINANCE)), mixed, ENHANCED_VIBRANCE);
}

// Two values in nothing to one, from a point: the jitter the reflection reads its history with.
fn hash22(point: vec2<f32>) -> vec2<f32> {
  var p: vec3<f32> = fract(vec3<f32>(point.xyx) * vec3<f32>(0.1031, 0.103, 0.0973));

  p += dot(p, p.yzx + 33.33);

  return fract((p.xx + p.yz) * p.zy);
}

// A view space point's place on the screen, in texture coordinates.
fn view_to_uv(point: vec3<f32>) -> vec2<f32> {
  let clip: vec4<f32> = camera.projection * vec4<f32>(point, 1.0);

  return clip.xy / clip.w * vec2<f32>(0.5, -0.5) + 0.5;
}

// How far along the view the G-buffer's surface lies at a place on the screen; nothing where only the sky is. The
// perspective's depth terms alone give it, as its `w` is the distance: `depth * d = P[3][2] - P[2][2] * d`.
fn scene_distance(uv: vec2<f32>) -> f32 {
  let size: vec2<f32> = camera.viewport.xy;
  let pixel: vec2<f32> = clamp(floor(uv * size), vec2<f32>(0.0), size - 1.0);
  let stored: f32 = textureLoad(depth_target, vec2<i32>(pixel), 0);

  return select(camera.projection[3][2] / (stored + camera.projection[2][2]), 0.0, stored <= 0.0);
}

// The reflection's ray as it marches the screen: where it starts and steps, how long it runs on the screen, and how far
// along the view its ends lie.
struct ReflectionRay {
  start: vec2<f32>,
  step: vec2<f32>,
  length: f32,
  near: f32,
  far: f32,
};

// `SSFX_ray_intersect`: how far the ray, at a place on the screen, lies past the surface there, and that surface's
// distance.
fn ray_behind(ray: ReflectionRay, at: vec2<f32>) -> vec2<f32> {
  let share: f32 = length(at - ray.start) / max(ray.length, 1e-6);
  let along: f32 = ray.near * ray.far / mix(ray.far, ray.near, share);
  let scene: f32 = scene_distance(at);

  return vec2<f32>(along - scene, scene);
}

// `SSFX_ssr_water_ray`: the ray from a point of the water along its reflection, marched over the screen and refined
// where it passes behind a surface; where it hit, then that surface's distance, or nothing where it left the screen.
// A surface nearer than 1.3 m, which the module takes for the actor's weapon, is passed through.
fn march_reflection(start: vec3<f32>, direction: vec3<f32>, noise: f32, uv: vec2<f32>) -> vec3<f32> {
  // A ray turning towards the eye stops short of the near plane rather than wrap behind it.
  let reach: f32 = select(MARCH_REACH, min(MARCH_REACH, (-0.1 - start.z) / direction.z), direction.z > 0.0);
  let end: vec3<f32> = start + direction * max(reach, 0.0);
  let screen_start: vec2<f32> = view_to_uv(start);
  let screen_end: vec2<f32> = view_to_uv(end);
  var ray: ReflectionRay;

  ray.start = screen_start;
  ray.step = (screen_end - screen_start) / f32(MARCH_STEPS);
  ray.length = length(screen_end - screen_start);
  ray.near = -start.z;
  ray.far = -end.z;

  // Squeezed across near the screen's sides, unless the eye looks down.
  let edges: vec2<f32> = 1.0 - smoothstep(vec2<f32>(0.9), vec2<f32>(1.0), vec2<f32>(uv.x, 1.0 - uv.x));
  let looking_down: f32 = saturate(-camera.view[1].z * 3.0);

  ray.step.x *= saturate(edges.x * edges.y + looking_down);

  var at: vec2<f32> = ray.start + ray.step * noise;
  let start_distance: f32 = scene_distance(ray.start);
  var behind: vec3<f32> = vec3<f32>(0.0);

  for (var step: i32 = 1; step <= MARCH_STEPS; step++) {
    if (any(at < vec2<f32>(0.0)) || any(at > vec2<f32>(1.0))) {
      return vec3<f32>(0.0);
    }

    var check: vec2<f32> = ray_behind(ray, at);
    let is_far: bool = check.y > 1.3;

    check.x *= f32(is_far);

    if (check.x > 0.0) {
      if (check.x <= MARCH_THICKNESS || start_distance + 40.0 < check.y) {
        return vec3<f32>(at, check.y);
      }

      let kept: vec2<f32> = at;
      let kept_step: vec2<f32> = ray.step;
      var last_sign: f32 = -1.0;

      for (var refine: i32 = 0; refine < MARCH_REFINES; refine++) {
        if (sign(check.x) != last_sign) {
          ray.step *= -0.5;
          last_sign = sign(check.x);
        }

        at += ray.step;
        check = ray_behind(ray, at);

        if (abs(check.x) <= MARCH_THICKNESS) {
          return vec3<f32>(at, check.y);
        }
      }

      at = kept;
      ray.step = kept_step;
    } else {
      behind = vec3<f32>(at, check.y) * f32(start_distance - 2.0 < check.y && is_far);
    }

    let is_passed: bool = !is_far && check.y > 0.01 && f32(step) > f32(MARCH_STEPS) * 0.4;

    at += ray.step * select(1.0, 3.5, is_passed);
  }

  return behind;
}

// Screen Space Shaders' `ssfx_water_ssr.ps`: the scene the water's flat surface reflects, marched over the screen as it
// stood before the water, faded towards the top of the screen and into the fog, the sky where the ray met nothing; then
// kept over the frames before, read where this one's point stood in the last.
@fragment
fn fs_water_reflection(in: WaterVarying) -> @location(0) vec4<f32> {
  let position: vec3<f32> = (camera.view * vec4<f32>(in.world, 1.0)).xyz;
  let fog: f32 = select(0.0, fog_amount(lighting, position), lighting.params.y > 0.5);

  if (fog >= 1.0 || in.clip.z < textureLoad(nearest_water, vec2<i32>(in.clip.xy), 0)) {
    discard;
  }

  let size: vec2<f32> = camera.viewport.xy;
  let uv: vec2<f32> = (floor(in.clip.xy) + 0.5) / size;
  let normal: vec3<f32> = normalize(in.normal);
  let eye: vec3<f32> = normalize(position);
  let normal_view: vec3<f32> = normalize((camera.view * vec4<f32>(normal, 0.0)).xyz);
  let reflected: vec3<f32> = reflect(eye, normal_view);
  let to_point: vec3<f32> = normalize(in.world - camera.position.xyz);
  let flat_reflected: vec3<f32> = reflect(to_point, normal);
  // Rays towards the eye are not traced: looking down they only mess the reflection. Nor are those the surface would
  // show too little of.
  let shown: f32 = pow(saturate(dot(flat_reflected, to_point)), 3.0) * water.reflectivity;
  let is_away: bool = dot(-eye, reflected) <= -0.3 && shown >= MARCH_LEAST_SHOWN;
  let noise_uv: vec2<f32> = (uv * 1.33 + water.time * 0.02) * vec2<f32>(size.x / size.y, 1.0);
  let noise: f32 = textureSampleLevel(blue_noise, texture_sampler, noise_uv, 0.0).x * 1.5;
  // A branch rather than a select, which would march every ray to throw most away.
  var hit: vec3<f32> = vec3<f32>(0.0);

  if (is_away) {
    hit = march_reflection(position, reflected, noise, uv);
  }
  let sky: vec3<f32> = enhanced_sky(flat_reflected) * water.reflection;
  var reflection: vec3<f32> = sky;

  if (all(hit.xy != vec2<f32>(0.0))) {
    let scene: vec3<f32> = textureLoad(water_scene, vec2<i32>(clamp(floor(hit.xy * size), vec2<f32>(0.0), size - 1.0)), 0)
      .rgb;
    let fogged: f32 = saturate((length(vec3<f32>(position.xy, hit.z)) * lighting.fog.y + lighting.fog.x) * 1.4);

    reflection = mix(sky, scene, saturate(hit.y * 5.0 * f32(is_away) * (1.0 - fogged)));
  }

  // The point the frames before are read at: along this pixel's view as far as what the ray met, or far off at the sky.
  let hit_distance: f32 = select(hit.z, 1e5, hit.z <= 0.0);
  let along: vec3<f32> = eye * (hit_distance / max(-eye.z, 1e-4));
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let world_hit: vec4<f32> = vec4<f32>(camera.position.xyz + rotation * along, 1.0);
  let now: vec4<f32> = camera.motion_current * world_hit;
  let before: vec4<f32> = camera.motion_previous * world_hit;
  let moved: vec2<f32> = (now.xy / now.w - before.xy / before.w) * vec2<f32>(0.5, -0.5);
  let previous: vec2<f32> = uv - moved;
  let is_off: bool = any(previous < vec2<f32>(0.0)) || any(previous > vec2<f32>(1.0));
  let kept: f32 = saturate(REFLECTION_HISTORY - select(0.0, REFLECTION_SKY_HISTORY, hit.z <= 0.0) - f32(is_off)) *
    water.history;
  let jitter: vec2<f32> = (hash22(uv * 100.0 + water.time * 100.0) * 2.0 - 1.0) / size * 0.25;
  let history: vec3<f32> = textureSampleLevel(reflection_history, sky_clamp, previous + jitter, 0.0).rgb;

  return vec4<f32>(mix(reflection, history, kept), 1.0);
}


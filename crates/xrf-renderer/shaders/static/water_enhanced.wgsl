#import "common/water_surface"
#import "common/scene_light"
#import "common/water_enhanced"
#import "common/rain_rings"
#import "common/ray_guides"
#import "generated/static/water_enhanced"

// The enhanced water's waves: its normal map scrolled twice over coordinates its parallax moves into the waves'
// height, a wind layer and the rain's ripples over them, paced and strengthened by the weather's wind.

// The enhanced water's wave strength, across and down, at its calmest and in the strongest wind: how much of its waves'
// normal moves what lies under it, a share of the screen; its refraction strength scales both.
const WAVES_CALM: vec2<f32> = vec2<f32>(0.03, 0.1);
const WAVES_WINDY: vec2<f32> = vec2<f32>(0.1, 0.2);

// The parallax: its most steps, looking along the surface, and the distance over which it falls to a single step,
// from three fifths of it.
const PARALLAX_STEPS: f32 = 16.0;
const PARALLAX_DISTANCE: f32 = 25.0;

// The waves' coordinates a metre of the level spans, before the tiling: read off the level's own place, the engine's
// `x` and `-z`, not the water mesh's UVs, which the levels map piece by piece, so the waves jumped along a line where
// two triangles' UVs did not meet (the engine's and the shader pack's water both read UVs; IXRay's D3D11 water reads
// the place at 0.3).
const WAVE_WORLD_SCALE: f32 = 0.3;

// How many times the enhanced water tiles the base coordinate.
const ENHANCED_TILING: f32 = 1.5;

// The waves' layers: the normal map read four times, each turned (0, 61, 137 and 223 degrees), tiled by powers of the
// golden ratio so no two repeat together, and weighted towards the finest. Each runs along its own turned axis at the
// deep-water pace of its wavelength, `c = sqrt(g / k)`, so the long ones move slower than the short.
const WAVE_TURNS: array<mat2x2<f32>, 4> = array<mat2x2<f32>, 4>(
  mat2x2<f32>(1.0, 0.0, 0.0, 1.0),
  mat2x2<f32>(0.484810, 0.874620, -0.874620, 0.484810),
  mat2x2<f32>(-0.731354, 0.681998, -0.681998, -0.731354),
  mat2x2<f32>(-0.731354, -0.681998, 0.681998, -0.731354),
);
const WAVE_TILES: vec4<f32> = vec4<f32>(1.0, 1.618034, 0.618034, 0.381966);
const WAVE_WEIGHTS: vec4<f32> = vec4<f32>(0.35, 0.25, 0.25, 0.15);
// Map repeats a wave-clock second the layer tiled once runs.
const WAVE_PACE: f32 = 0.3;

// Calm and rippled patches: a noise over the level about 70 m across, drifting with the flow, scales the waves between
// these shares of their strength, by the variation.
const PATCH_SCALE: f32 = 0.015;
const PATCH_DRIFT: vec2<f32> = vec2<f32>(0.0021, 0.0013);
const PATCH_CALM: f32 = 0.4;

// Metres over which the finer layers give way to the coarsest, where their detail would only shimmer.
const BROAD_NEAR: f32 = 5.0;
const BROAD_FAR: f32 = 60.0;

// What share of their slope the waves keep once filtered at a distance, and the sun highlight's sharpness near and far:
// widened as the detail that would sharpen it is filtered out, its peak lowered to keep the light it reflects.
const FAR_SLOPE: f32 = 0.6;
const HIGHLIGHT_NEAR: f32 = 512.0;
const HIGHLIGHT_FAR: f32 = 160.0;


// The rain's ripples: how fast each of its three layers ripples, where the second and third are moved to, and how far
// the ripples reach.
const RIPPLE_SPEEDS: vec3<f32> = vec3<f32>(1.05, 1.31, 1.58);
const RIPPLE_OFFSETS: vec4<f32> = vec4<f32>(0.5, 0.25, 0.31, 0.5);
const RIPPLE_REACH: f32 = 15.0;
// How rough the rain's rings leave the water, by the variance of their tilt their filtering folded away.
const RINGS_ROUGHNESS: f32 = 30.0;

// How strongly the enhanced drops' own rings ripple it, against the map's rings: each ring stands alone where the
// map's layers overlap, so each is drawn strong enough to read in the sky the water mirrors.
const RINGS_STRENGTH: f32 = 1.8;

// The enhanced water's waves: its normal map scrolled twice over coordinates its parallax moves
// into its waves' height, the wind's layer leaning the first and the rain's ripples over both, at a pace and a strength
// the weather's wind sets; how far they move what lies under the water across the screen, the surface's normal in the
// world, the height its parallax met, and the first layer's tilt, which moves where its colour is
// read.
struct EnhancedWaves {
  screen: vec2<f32>,
  normal: vec3<f32>,
  height: f32,
  tilt: vec2<f32>,
  // How far the waves' finer detail has been filtered out with distance, from none to all.
  far: f32,
  // How rough the rain's rings leave it where they are filtered finer than a pixel, from none to all: what of their
  // tilt the filtering folded away, which blurs its reflection as rain roughens water seen far off.
  rain_rough: f32,
};

// The waves' height at coordinates, which its parallax reads: two scrolls of the height map, the higher.
fn wave_height(coordinates: vec2<f32>) -> f32 {
  let at: vec2<f32> = coordinates * 0.35;
  let first: f32 = textureSampleLevel(height_map, texture_sampler, at + enhanced.heights * vec2<f32>(0.065, 0.445), 0.0).r;
  let second: f32 = textureSampleLevel(height_map, texture_sampler, at - enhanced.heights * vec2<f32>(0.105, 0.241), 0.0).b;

  return max(first, second);
}

// Steps along the eye's way across the surface until the waves' height rises over it, then the
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
  let crossing: f32 = left - saturate(last_height - depth + share);
  // A single step onto a height of exactly nought crosses nothing: the step's own end, with no offset.
  let ratio: f32 = select(1.0, left / crossing, abs(crossing) > 1e-6);

  return vec3<f32>(mix(at, at + step, ratio), sampled);
}

// One layer of rain ripples, each texel a ring that spreads and fades on its own clock.
fn ripple_layer(ripple: vec4<f32>, speed: f32) -> vec2<f32> {
  let clock: f32 = fract(ripple.w + water.time * speed);
  let spread: f32 = clamp((clock - 1.0 + ripple.x) * RIPPLE_REACH, 0.0, 4.0);
  let factor: f32 = saturate(0.7 - clock) * ripple.x * sin(spread * 3.141592) * (1.0 - smoothstep(0.0, 4.0, spread));

  return (ripple.yz * 2.0 - 1.0) * factor * enhanced.ripples;
}

// Three layers of ripples over the surface, fading out over fifteen metres of the bottom's distance.
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
  let wind: f32 = saturate(enhanced.wind_velocity * 0.001);
  let base: vec2<f32> = vec2<f32>(in.world.x, -in.world.z) * WAVE_WORLD_SCALE * ENHANCED_TILING;
  // The axes the coordinates run along, laid onto the surface: the world's `x` and `-z`.
  let surface_normal: vec3<f32> = normalize(in.normal);
  let across: vec3<f32> = normalize(vec3<f32>(1.0, 0.0, 0.0) - surface_normal * surface_normal.x);
  let along: vec3<f32> = normalize(vec3<f32>(0.0, 0.0, -1.0) + surface_normal * surface_normal.z);
  let dx: vec2<f32> = dpdx(base);
  let dy: vec2<f32> = dpdy(base);
  var coordinates: vec2<f32> = base + enhanced.waves * vec2<f32>(0.065, 0.445);
  var height: f32 = 0.0;
  let distance: f32 = -(camera.view * vec4<f32>(in.world, 1.0)).z;

  // The parallax, while the waves stand: steps by how closely the eye looks along the surface, one alone past its
  // distance.
  if (enhanced.parallax_height > 0.0) {
    let to_eye: vec3<f32> = -camera_world_ray(in.world);
    let eye: vec3<f32> = normalize(vec3<f32>(
      dot(to_eye, across),
      dot(to_eye, along),
      dot(to_eye, surface_normal),
    ));
    let is_near: bool = distance < PARALLAX_DISTANCE;
    let share: f32 = select(1.0, 1.0 / mix(PARALLAX_STEPS, 1.0, abs(eye.z)), is_near);
    let lift: f32 = clamp(enhanced.parallax_height * wind, 0.015, enhanced.parallax_height);
    // Held off nought, as a surface seen edge-on would offset by no finite amount.
    let facing: f32 = select(-1.0, 1.0, eye.z >= 0.0) * max(abs(eye.z), 1e-3);
    let marched: vec3<f32> = march_parallax(base, share * eye.xy / facing * lift, share);

    coordinates = marched.xy;
    height = marched.z;
  }

  // Each layer's normal turned back from its own axis into the surface's, weighted by its share, the finer shares
  // given to the coarsest with distance.
  let far: f32 = smoothstep(BROAD_NEAR, BROAD_FAR, distance);
  let weights: vec4<f32> = WAVE_WEIGHTS * vec4<f32>(1.0 - far * 0.8, 1.0 - far * 0.9, 1.0, 1.0 + far * 2.0);
  var slope: vec2<f32> = vec2<f32>(0.0);

  for (var layer: u32 = 0u; layer < 4u; layer++) {
    let turn: mat2x2<f32> = WAVE_TURNS[layer];
    let tile: f32 = WAVE_TILES[layer];
    let run: vec2<f32> = vec2<f32>(enhanced.waves * WAVE_PACE / sqrt(tile), 0.0);
    let texel: vec3<f32> = textureSampleGrad(wave_map, texture_sampler, turn * coordinates * tile + run,
      turn * dx * tile, turn * dy * tile).rgb;

    slope += transpose(turn) * (texel.xy * 2.0 - 1.0) * weights[layer];
  }

  // Filtered with distance, the slopes flatten towards their average rather than keep the finest layers' contrast,
  // which only shimmered; the sun's highlight widens to match (`fs_water_enhanced`).
  slope *= mix(1.0, FAR_SLOPE, far) / dot(weights, vec4<f32>(1.0));

  // The patches, calm where the noise is low.
  let level: vec2<f32> = vec2<f32>(in.world.x, -in.world.z);
  let calm_noise: f32 = textureSampleLevel(perlin_map, texture_sampler, level * PATCH_SCALE + PATCH_DRIFT * enhanced.flowed,
    0.0).r;

  slope *= mix(1.0, mix(PATCH_CALM, 1.0, smoothstep(0.3, 0.7, calm_noise)), enhanced.variation);

  let gust: vec2<f32> = textureSampleGrad(wind_map, texture_sampler, coordinates * 0.1 + vec2<f32>(enhanced.gusts_x, enhanced.gusts_y) * 0.1,
    dx * 0.1, dy * 0.1).rg;

  slope = mix(slope, gust * 2.0 - 1.0, 0.1 * wind);

  var ripples: vec2<f32> = vec2<f32>(0.0);
  var rain_rough: f32 = 0.0;

  // The rain's ripples, while it rains and they are wanted, its strength from the rain's density: the enhanced drops'
  // own rings where they fall, the map's otherwise.
  if (enhanced.rain > 0.0 && enhanced.ripples > 0.0) {
    if (enhanced.rings_tile > 0.0) {
      let to_world: vec2<f32> = vec2<f32>(1.0, -1.0) / (WAVE_WORLD_SCALE * ENHANCED_TILING);
      let rings: RainRings = read_rain_rings(rain_rings, texture_sampler,
        vec2<f32>(enhanced.rings_tile, enhanced.rings_reach), camera.position.xz, in.world.xz, dx * to_world,
        dy * to_world);

      let ring_scale: f32 = enhanced.ripples * RINGS_STRENGTH * enhanced.rings_strength * rings.share;

      ripples = clamp(rings.tilt * vec2<f32>(1.0, -1.0) * ring_scale, vec2<f32>(-1.0), vec2<f32>(1.0));
      rain_rough = saturate(rings.variance * ring_scale * ring_scale * RINGS_ROUGHNESS);
    } else {
      ripples = rain_ripples(coordinates * 0.6, dx * 0.6, dy * 0.6, clamp(enhanced.rain * 1.6, 0.65, 1.0),
        bottom_distance);
    }
  }

  let screen: vec3<f32> = vec3<f32>(slope, 1.0);
  let average: vec3<f32> = vec3<f32>(slope + ripples, 1.0);

  var out: EnhancedWaves;

  // The strongest wind's strength by the wind, never under the calm's.
  let strength: vec2<f32> = clamp(WAVES_WINDY * wind, WAVES_CALM, WAVES_WINDY);

  out.screen = normalize(vec3<f32>(screen.xy * strength * enhanced.refraction, screen.z)).xy + ripples;
  out.normal = normalize(across * average.x + along * average.y + surface_normal * average.z);
  out.height = height;
  out.tilt = screen.xy;
  out.far = far;
  out.rain_rough = rain_rough;

  return out;
}

// How much of its waves' height the sun's highlight sees.
const SPECULAR_NORMAL: f32 = 0.2;

// Its colour's second read: turned 37 degrees and tiled 0.382 times as finely, so its repeat never meets the first's,
// mixed in by a perlin noise over the level about 30 m across.
const COLOUR_TURN: mat2x2<f32> = mat2x2<f32>(0.798636, 0.601815, -0.601815, 0.798636);
const COLOUR_TILING: f32 = 0.382;
const COLOUR_NOISE: f32 = 0.035;
const COLOUR_BEND: vec2<f32> = vec2<f32>(1.3, -0.9);

// The water's fog at which it is its colour alone.
const ENHANCED_FOG_DEPTH: f32 = 2.0;

// The share of its colour the shallowest water keeps, and how sharply, per metre of depth, its edge meets the shore.
const SHALLOW_COLOUR: f32 = 0.3;
const SHORE_FADE: f32 = 40.0;

// The luminance the reflection greys towards as its vibrance falls.
const LUMINANCE: vec3<f32> = vec3<f32>(0.3, 0.38, 0.22);

@vertex
fn vs_water(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> WaterVarying {
  return water_vertex(vertex_index, instance_index, water);
}

// The light the enhanced water gathers onto its bottom: two scrolls of its caustics map over the
// bottom's place in the level, the least of the two, sampled before the fragment can be discarded.
fn enhanced_caustics(bottom: vec3<f32>) -> vec3<f32> {
  let place: vec2<f32> = vec2<f32>(bottom.x, -bottom.z);
  let time: f32 = enhanced.flowed;
  let first: vec3<f32> = saturate(textureSample(caustics_map, texture_sampler, place * 0.19 + vec2<f32>(time * 0.1, 0.0)).rgb -
    0.1);
  let second: vec3<f32> = saturate(textureSample(caustics_map, texture_sampler, place * 0.11 + vec2<f32>(-time * 0.07, 0.2))
    .rgb - 0.1);

  return min(first, second);
}

// The enhanced water's surface: what lies under it, read where its waves move it unless something
// nearer than the water stands there, clouded into the water's colour by the water's depth, its reflection over it by
// the fresnel cubed, the light it gathers on its bottom and the sun's highlight where the sun reaches it, fogged, and
// faded into what lies under it along its shallow edge.
// The enhanced water's pixel and what ray reconstruction's guides read of it: its waves' normal, the share of its
// reflection it shows, how rough its reflection reads, and how far its reflected ray went.
struct EnhancedShaded {
  out: WaterOutput,
  normal: vec3<f32>,
  reflected_share: f32,
  roughness: f32,
  hit_distance: f32,
};

// The guides' roughness of the water's reflection, calm and under the heaviest rain's ripples.
const GUIDE_ROUGHNESS_CALM: f32 = 0.05;
const GUIDE_ROUGHNESS_RAIN: f32 = 0.3;

@fragment
fn fs_water_enhanced(in: WaterVarying) -> WaterOutput {
  return shade_enhanced(in).out;
}

// The guides ray reconstruction reads, written where the enhanced water draws, over the G-buffer's, which the water
// draws over after the guides pass.
struct GuidedOutput {
  @location(0) color: vec4<f32>,
  @location(1) distortion: vec4<f32>,
  @location(2) motion: vec2<f32>,
  @location(3) specular_albedo: vec4<f32>,
  @location(4) normals: vec4<f32>,
  @location(5) hit_distance: f32,
  @location(6) bias: f32,
};

@fragment
fn fs_water_enhanced_guided(in: WaterVarying) -> GuidedOutput {
  let shaded: EnhancedShaded = shade_enhanced(in);
  var out: GuidedOutput;

  out.color = shaded.out.color;
  out.distortion = shaded.out.distortion;
  out.motion = shaded.out.motion;
  out.specular_albedo = vec4<f32>(vec3<f32>(shaded.reflected_share), 1.0);
  out.normals = vec4<f32>(shaded.normal, shaded.roughness);
  out.hit_distance = shaded.hit_distance;
  out.bias = RAY_GUIDE_BIAS_WATER;

  return out;
}

// How far the reflected ray went from a point of the water (in view space) to what the reflection kept met, from the
// distance along the view the reflection keeps (`mirrored_distance`); the sky's where it met nothing or reflects none.
fn reflected_ray_length(position: vec3<f32>, kept: f32) -> f32 {
  if (enhanced.reflected < 0.5 || kept <= 0.0 || kept >= SKY_DISTANCE) {
    return RAY_GUIDE_FAR_HIT;
  }

  return clamp((kept / max(-position.z, 1e-4) - 1.0) * length(position), 0.0, RAY_GUIDE_FAR_HIT);
}

fn shade_enhanced(in: WaterVarying) -> EnhancedShaded {
  let size: vec2<f32> = camera.viewport.xy;
  let pixel: vec2<f32> = floor(in.clip.xy);
  // The bottom under the fragment, in the world; far below where only the sky is.
  let stored: f32 = textureLoad(depth_target, vec2<i32>(pixel), 0);
  let waves: EnhancedWaves = enhanced_waves(in, select(scene_distance((pixel + 0.5) / size, depth_target), FAR_BEHIND, stored <= 0.0));
  let ndc: vec2<f32> = vec2<f32>(in.clip.x / size.x * 2.0 - 1.0, 1.0 - in.clip.y / size.y * 2.0);
  let bottom: vec3<f32> = select(camera_unproject(ndc, stored), in.world - vec3<f32>(0.0, FAR_BEHIND, 0.0), stored <= 0.0);
  let gathered: vec3<f32> = enhanced_caustics(bottom);
  let fragment: WaterFragment = read_water(in, water, lighting, depth_target, nearest_water);
  let moved: vec2<f32> = clamp(pixel + waves.screen * size, vec2<f32>(0.0), size - 1.0);
  let moved_stored: f32 = textureLoad(depth_target, vec2<i32>(moved), 0);
  let moved_behind: f32 = select(-camera_view_position(moved, moved_stored).z, FAR_BEHIND, moved_stored <= 0.0);
  // A read nearer than the water would copy what stands over it into it, and the sky is refused as well.
  let refracted: vec2<f32> = select(pixel, moved, moved_stored > 0.0 && moved_behind > -fragment.position.z);
  let screen: vec3<f32> = scene_shown(textureLoad(water_scene, vec2<i32>(refracted), 0).rgb);
  // How deep the water stands over its bottom, straight down.
  let water_depth: f32 = in.world.y - bottom.y;
  // How much water the eye looks through to the bottom, never less than straight down: shallow water seen at a slant
  // clouds as deep water does. The waves' height its parallax met raises the fog a little.
  let path: f32 = max(water_depth, min(distance(bottom, in.world), FAR_BEHIND));
  let water_fog: f32 = exp(min(path + waves.height * 0.1, 16.0)) - 1.0;
  // Clear in the shallows, its colour where deep; even the
  // shallowest keeps a share of its colour, as murky water does.
  let clear: f32 = saturate((water_fog - ENHANCED_FOG_DEPTH) / (-enhanced.turbidity - ENHANCED_FOG_DEPTH)) *
    (1.0 - SHALLOW_COLOUR);
  let colour: vec3<f32> = mix(untextured_color(surfaces[in.surface].color), enhanced_colour(in, waves.tilt),
    camera.switches.x) * fragment.light;
  let turbid: vec3<f32> = mix(colour, screen, clear * clear * (3.0 - 2.0 * clear));
  let reflected: vec3<f32> = reflect(fragment.to_point, waves.normal);
  let fresnel: f32 = pow(saturate(dot(reflected, fragment.to_point)), 3.0);
  let reflection: vec3<f32> = enhanced_reflection(waves, reflected, in.world, (pixel + 0.5) / size);
  // Where the lights reached the bottom the sun shines on it.
  let sunlit: f32 = saturate(textureLoad(light_target, vec2<i32>(refracted), 0).r * 2000.0);
  let sun: f32 = dot(sun_shown(lighting), vec3<f32>(0.5));
  // The gathered light fades as the water clouds.
  let gathering: f32 = 1.0 - smoothstep(0.0, ENHANCED_FOG_DEPTH + 0.5, water_fog);
  let caustics: vec3<f32> = gathered * sunlit * smoothstep(0.3, 1.0, sun) * enhanced.caustics * gathering *
    saturate(water_fog * 3.0);
  // The way the sunlight travels, in the world.
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let sunlight: vec3<f32> = -(rotation * lighting.to_sun.xyz);
  let flattened: vec3<f32> = normalize(vec3<f32>(waves.normal.x, waves.normal.y * SPECULAR_NORMAL, waves.normal.z));
  let sharpness: f32 = mix(HIGHLIGHT_NEAR, HIGHLIGHT_FAR, waves.far);
  let highlight: vec3<f32> = sun_shown(lighting) * pow(abs(dot(normalize(fragment.to_point + sunlight), flattened)), sharpness) *
    (sharpness + 8.0) / (HIGHLIGHT_NEAR + 8.0) * saturate(sun) * enhanced.specular * sunlit;
  let seen: f32 = 1.0 - fragment.fog;
  let lit: vec3<f32> = mix(turbid, reflection, saturate(fresnel * enhanced.reflectivity)) + caustics + highlight;
  let fogged: vec3<f32> = mix(lighting.fog_color.rgb, lit, seen);
  let border: f32 = smoothstep(0.0, max(enhanced.soft_border, 1e-4), path + fresnel);
  let shown: vec3<f32> = mix(screen, fogged, border * seen * seen);
  var shaded: EnhancedShaded;

  shaded.out.color = vec4<f32>(
    scene_from_shown(select(fragment.base.rgb, shown, lighting.params.y > 0.5)),
    saturate(water_depth * SHORE_FADE),
  );
  // No distortion: the water refracts what is behind it above, before the temporal resolve. The distortion target is
  // read after the resolve, at the drawn size against its depth, which flips a blade's border pixels in and out of the
  // move each frame: a dotted outline along every blade over the water, nothing resolving it.
  shaded.out.distortion = vec4<f32>(0.0);
  shaded.out.motion = camera_motion(in.world, in.previous);
  shaded.normal = waves.normal;
  shaded.reflected_share = saturate(fresnel * enhanced.reflectivity) * seen * border * seen * seen;
  shaded.roughness = mix(GUIDE_ROUGHNESS_CALM, GUIDE_ROUGHNESS_RAIN, waves.rain_rough);
  shaded.hit_distance = reflected_ray_length(fragment.position,
    textureSampleLevel(reflection_clear, sky_clamp, saturate((pixel + 0.5) / size + waves.screen), 0.0).a);

  return shaded;
}

// The enhanced water's colour read over the level ten metres a repeat, moved by its waves' tilt, not over
// its coordinates; with its variation, a second read turned and tiled apart mixed in by a noise, so no repeat shows.
fn enhanced_colour(in: WaterVarying, tilt: vec2<f32>) -> vec3<f32> {
  let level: vec2<f32> = vec2<f32>(in.world.x, -in.world.z);
  let slot: u32 = surfaces[in.surface].textures[SLOT_BASE];
  let noise: f32 = textureSample(perlin_map, texture_sampler, level * COLOUR_NOISE).r;
  // The noise bends the first read's repeat as well, by up to a repeat's half.
  let bent: vec2<f32> = (noise - 0.5) * COLOUR_BEND * enhanced.variation;
  let first: vec3<f32> = sample_slot_implicit(slot, level * 0.1 + tilt + bent).rgb;
  let second: vec3<f32> = sample_slot_implicit(slot, COLOUR_TURN * level * (0.1 * COLOUR_TILING) + tilt).rgb;

  return mix(first, second, saturate(noise) * enhanced.variation);
}

// What the enhanced water reflects: its reflection drawn this frame, blurred and clear mixed by a perlin noise over the
// level and greyed a little, read where its waves move it; the sky alone while it draws none.
fn enhanced_reflection(waves: EnhancedWaves, reflected: vec3<f32>, world: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
  if (enhanced.reflected < 0.5) {
    return enhanced_sky(reflected, lighting, sky_cube_0, sky_cube_1, sky_clamp) * water.reflection;
  }

  // The blurred reflection is read where the waves move it twice as far.
  let blurred: vec3<f32> = textureSampleLevel(reflection_blurred, sky_clamp, saturate(uv + waves.screen * 2.0), 0.0).rgb;
  let clear: vec3<f32> = textureSampleLevel(reflection_clear, sky_clamp, saturate(uv + waves.screen), 0.0).rgb;
  let level: vec2<f32> = vec2<f32>(world.x, -world.z) + waves.normal.xy * 100.0;
  let noise: f32 = textureSampleLevel(perlin_map, texture_sampler, level * 0.02, 0.0).r;
  // The rain's roughness takes the clear reflection away for the blurred one.
  let mixed: vec3<f32> = mix(blurred, clear, saturate(noise * enhanced.blur_noise) * (1.0 - waves.rain_rough));

  return mix(vec3<f32>(dot(mixed, LUMINANCE)), mixed, enhanced.vibrance);
}

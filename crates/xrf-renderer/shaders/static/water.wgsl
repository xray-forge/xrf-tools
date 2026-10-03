enable wgpu_binding_array;

#import "common/camera"
#import "common/lighting"
#import "common/sky_box"
#import "static/pulling"

// Water as OpenXRay's `water.vs`, `water.ps` and `waterd.ps` draw it, and Anomaly's programs over them: the wave
// lifting the surface, two scrolling normal layers bending the sky's reflection, the fresnel mixing it with the base by
// the base's alpha, lit per vertex by the hemisphere, the sun and the ambient. Soft water fades by the depth behind it,
// darkens with it towards `water_intensity`, lays foam in the shallows and is fogged. Drawn over the lit frame as it
// shows, which `water.ps` writes without the tonemap, and the distortion it causes into a target of its own.

struct Water {
  // Seconds the water has moved.
  time: f32,
  // `W_POSITION_SHIFT_HEIGHT` and `W_POSITION_SHIFT_SPEED`.
  wave_height: f32,
  wave_speed: f32,
  // What the normal layers' scroll and the sky's reflection are multiplied by.
  ripple: f32,
  reflection: f32,
  // `water_intensity`.
  intensity: f32,
  // One while soft water reads the depth behind it, `r2_soft_water`.
  soft: f32,
  pad: f32,
};

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var texture_sampler: sampler;

@group(3) @binding(0) var<uniform> lighting: Lighting;
@group(3) @binding(1) var<uniform> water: Water;
// The G-buffer's depth, which the water is tested against and fades by.
@group(3) @binding(2) var depth_target: texture_depth_2d;
@group(3) @binding(3) var sky_cube_0: texture_cube<f32>;
@group(3) @binding(4) var sky_cube_1: texture_cube<f32>;
@group(3) @binding(5) var sky_clamp: sampler;

// `watermove`: the wave's direction through the level, in renderer space, where the engine's `z` is negated.
const WAVE_DIRECTION: vec3<f32> = vec3<f32>(0.11, 0.13, -0.07);

// `watermove_tc`: the scroll's direction across the level, over the engine's `x` and `z`.
const SCROLL_DIRECTION: vec2<f32> = vec2<f32>(0.2111, 0.2333);

// `W_DISTORT_BASE_TILE_0` and `_1`, and `W_DISTORT_AMP_0` and `_1` (`shared/waterconfig.h`).
const LAYER_TILES: vec2<f32> = vec2<f32>(1.0, 1.1);
const LAYER_AMPLITUDES: vec2<f32> = vec2<f32>(0.15, 0.55);

// What the depth behind the water reads where nothing was drawn: farther than any water is deep.
const FAR_BEHIND: f32 = 1e6;

struct WaterVarying {
  @builtin(position) clip: vec4<f32>,
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

// Both keyframes' skies along a world direction, as a surface reflecting them reads the cube.
fn sky_cubes(direction: vec3<f32>) -> vec3<f32> {
  let lookup: vec3<f32> = sky_box_direction(direction, lighting.sky_params.x);

  return mix(
    textureSampleLevel(sky_cube_0, sky_clamp, lookup, 0.0).rgb,
    textureSampleLevel(sky_cube_1, sky_clamp, lookup, 0.0).rgb,
    lighting.sky.w,
  );
}

fn sample_slot(slot: u32, uv: vec2<f32>) -> vec4<f32> {
  return textureSample(textures[slot], texture_sampler, uv);
}

@fragment
fn fs_water(in: WaterVarying) -> WaterOutput {
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
  let base: vec4<f32> = vec4<f32>(mix(untextured_color(surface.color), sampled.rgb, is_textured), sampled.a);
  let bent: vec3<f32> = select(vec3<f32>(0.0, 0.0, 1.0), normals, (flags & SURFACE_HAS_WATER_NORMAL) != 0u);
  let foam: vec4<f32> = select(vec4<f32>(0.0), foam_texel, (flags & SURFACE_HAS_FOAM) != 0u);
  let offset_texel: vec2<f32> = select(vec2<f32>(0.5), distorted, (flags & SURFACE_HAS_DISTORTION) != 0u);
  let normal: vec3<f32> = normalize(in.normal);
  let surface_normal: vec3<f32> = normalize(in.tangent * bent.x + in.binormal * bent.y + normal * bent.z);
  let to_point: vec3<f32> = normalize(in.world - camera.position.xyz);
  let position: vec3<f32> = (camera.view * vec4<f32>(in.world, 1.0)).xyz;
  let fog: f32 = select(0.0, fog_amount(lighting, position), lighting.params.y > 0.5);

  // The far plane ends where fog is total, so nothing past it is drawn but the sky.
  if (fog >= 1.0) {
    discard;
  }

  // The true remapping, then the fast one below the top: the cube's lower half is never shown.
  let reflected: vec3<f32> = reflect(to_point, surface_normal);
  let scaled: vec3<f32> = reflected / max(abs(reflected.x), max(abs(reflected.y), abs(reflected.z)));
  let remapped: vec3<f32> = vec3<f32>(scaled.x, select(scaled.y, scaled.y * 2.0 - 1.0, scaled.y < 0.999), scaled.z);
  // `c0`: the vertex's baked light, the hemisphere by its occlusion, the sun by its sun occlusion, and the ambient, as
  // `L_hemi_color`, `L_sun_color` and `L_ambient` bind them raw.
  let normal_view: vec3<f32> = (camera.view * vec4<f32>(normal, 0.0)).xyz;
  let light: vec3<f32> = in.baked.rgb + lighting.environment.rgb * 0.25 * (0.5 + normal.y * 0.5) * in.hemi +
    lighting.sun.rgb * dot(normal_view, lighting.to_sun.xyz) * in.baked.a + lighting.ambient.rgb * 0.5;
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

      color += lighting.sun.rgb * pow(abs(dot(normalize(to_point + sun), surface_normal)), 256.0) * 4.0;
    }

    lit = color * light * 2.0;
    plain_alpha = 0.55 + power * 0.25;
  } else {
    // OpenXRay's `water.ps`: the sky squared and doubled, a share of it by the fresnel, mixed with the base by its
    // alpha.
    let power: f32 = pow(saturate(dot(remapped, to_point)), 9.0);
    let sky: vec3<f32> = sky_cubes(remapped);
    let amount: f32 = (0.15 + power * 0.25) * water.reflection;

    lit = mix(sky * sky * 2.0 * amount, base.rgb, base.a) * light * 2.0;
    plain_alpha = 0.75 + power * 0.25;
  }

  // `NEED_SOFT_WATER` and `USE_SOFT_WATER`: the depth behind the surface, which fades, darkens and foams it.
  let stored: f32 = textureLoad(depth_target, vec2<i32>(in.clip.xy), 0);
  let behind: f32 = select(-camera_view_position(in.clip.xy, stored).z, FAR_BEHIND, stored <= 0.0);
  let depth: f32 = behind + position.z;
  let deepened: vec3<f32> = mix(vec3<f32>(water.intensity * 0.1), lit, plain_alpha);
  let faded: f32 = max(1.0 - exp(depth * -4.0), min(plain_alpha, saturate(depth)));
  let shallow: f32 = saturate(-depth * dot(normal, to_point));
  let is_foamed: bool = (flags & SURFACE_IS_ANOMALY_WATER) == 0u || (flags & SURFACE_IS_FOAMED) != 0u;
  let foamed: f32 = smoothstep(0.025, 0.05, shallow) * (1.0 - smoothstep(0.075, 0.1, shallow)) * foam.a * is_textured *
    f32(is_foamed);
  let soft_color: vec3<f32> = mix(mix(deepened, foam.rgb * water.intensity, foamed), lit, 1.0 - water.soft);
  let soft_alpha: f32 = mix(mix(faded, foam.a, foamed), plain_alpha, 1.0 - water.soft);
  let is_soft: bool = (flags & SURFACE_IS_SOFT_WATER) != 0u;
  let color: vec3<f32> = select(lit, soft_color, is_soft);
  let seen: f32 = 1.0 - fog;
  // Plain `water` is written whole, `blend(false)`; soft water is faded by the fog twice over, as its alpha is.
  let alpha: f32 = select(1.0, soft_alpha * seen * seen, is_soft);
  let finished: vec3<f32> = mix(color, lighting.fog_color.rgb, fog);
  let shown: vec3<f32> = select(base.rgb, finished, lighting.params.y > 0.5);
  // `waterd.ps`: the distortion map at the normal layers' coordinates, gone where the base is opaque, faded by the
  // depth behind soft water, then halved around nothing.
  let opaque: vec2<f32> = mix(offset_texel, vec2<f32>(0.5), base.a);
  let shoal: vec2<f32> = mix(vec2<f32>(0.5), opaque, saturate(depth * 5.0));
  let offset: vec2<f32> = select(opaque, mix(opaque, shoal, water.soft), is_soft);
  var out: WaterOutput;

  out.color = vec4<f32>(shown, alpha);
  out.distortion = vec4<f32>(offset * 0.5 + 0.25, select(0.08, 0.0, is_soft), 0.5);

  return out;
}

enable wgpu_binding_array;

#import "common/camera"
#import "common/lighting"
#import "static/pulling"
#import "generated/structs"

// What every water program shares: the vertex the wave lifts, the fragment's textures, normals, eye, fog, sky and vertex
// light, the depth behind it, and the distortion it writes. The water's own bindings are passed in, since each program
// declares its group 3 from its own passes' parameters.

@group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var texture_sampler: sampler;

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

// The vertex every water program draws, lifted by the wave; each program's `vs_water` calls it with its `water`.
fn water_vertex(vertex_index: u32, instance_index: u32, water: Water) -> WaterVarying {
  let pulled: PulledVertex = pull(vertex_index, instance_index);
  let at: u32 = pulled.word;
  let place: Place = pulled.place;
  let matrix: mat4x4<f32> = place.transform;
  let linear: mat3x3<f32> = mat3x3<f32>(place.transform[0].xyz, place.transform[1].xyz, place.transform[2].xyz);
  let scale: vec3<f32> = vec3<f32>(dot(place.transform[0].xyz, place.transform[0].xyz), dot(place.transform[1].xyz, place.transform[1].xyz),
    dot(place.transform[2].xyz, place.transform[2].xyz));
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
fn scrolled(base: vec2<f32>, world: vec3<f32>, tile: f32, amplitude: f32, water: Water) -> vec2<f32> {
  let angle: f32 = water.time / 10.0 + dot(vec2<f32>(world.x, -world.z), SCROLL_DIRECTION * amplitude);

  return base * tile + vec2<f32>(sin(angle), cos(angle)) * amplitude * water.ripple;
}

fn sample_slot(slot: u32, uv: vec2<f32>) -> vec4<f32> {
  return textureSample(textures[slot], texture_sampler, uv);
}

// What every water takes of a fragment: its surface's textures, its normals, the eye, the fog, the sky it reflects, the
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

// Samples a fragment's surface and works out what every water reads of it, discarding where the fog is total and where a
// nearer fold of the surface covers it.
fn read_water(
  in: WaterVarying,
  water: Water,
  lighting: Lighting,
  depth_target: texture_depth_2d,
  nearest_water: texture_depth_2d,
) -> WaterFragment {
  let surface: Surface = surfaces[in.surface];
  let flags: u32 = surface.flags;
  let first: vec2<f32> = scrolled(in.uv, in.world, LAYER_TILES.x, LAYER_AMPLITUDES.x, water);
  let second: vec2<f32> = scrolled(in.uv, in.world, LAYER_TILES.y, LAYER_AMPLITUDES.y, water);
  // Sampled before any branch, where every derivative is taken alike; a texture the surface binds none of is ignored.
  let sampled: vec4<f32> = sample_slot(surface.textures[SLOT_BASE], in.uv);
  let normals: vec3<f32> = sample_slot(surface.textures[SLOT_WATER_NORMAL], first).xyz + sample_slot(surface.textures[SLOT_WATER_NORMAL], second).xyz - 1.0;
  let foam_texel: vec4<f32> = sample_slot(surface.textures[SLOT_FOAM], in.uv);
  let distorted: vec2<f32> = (sample_slot(surface.textures[SLOT_DISTORTION], first).xy +
    sample_slot(surface.textures[SLOT_DISTORTION], second).xy) * 0.5;
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
fn write_distortion(fragment: WaterFragment, water: Water) -> vec4<f32> {
  let is_soft: bool = (fragment.flags & SURFACE_IS_SOFT_WATER) != 0u;
  let opaque: vec2<f32> = mix(fragment.offset_texel, vec2<f32>(0.5), fragment.base.a);
  let shoal: vec2<f32> = mix(vec2<f32>(0.5), opaque, saturate(fragment.depth * 5.0));
  let offset: vec2<f32> = select(opaque, mix(opaque, shoal, water.soft), is_soft);

  return vec4<f32>(offset * 0.5 + 0.25, select(0.08, 0.0, is_soft), 0.5 * water.distorted);
}

enable wgpu_binding_array;

#import "common/camera"
#import "common/cut_out"
#import "common/octahedral"
#import "static/pulling"

// Static draws into the G-buffer, pulled as `static/pulling` reads them.


@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var texture_sampler: sampler;

// What `def_gloss` writes for a surface without a bump: 2 of 255.
const DEFAULT_GLOSS: f32 = 2.0 / 255.0;

// The sky share of an object under open sky, `ps_r2_dhemi_sky_scale`, which one without an estimate is lit by.
const OPEN_SKY: f32 = 0.08;

struct GBufferVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) normal: vec3<f32>,
  @location(1) tangent: vec3<f32>,
  @location(2) binormal: vec3<f32>,
  @location(3) uv: vec2<f32>,
  @location(4) lightmap_uv: vec2<f32>,
  @location(5) hemi: f32,
  @location(6) @interpolate(flat) surface: u32,
  // The cluster and place drawn, which a pick reads back.
  @location(7) @interpolate(flat) entry: vec2<u32>,
  // A spawned object's sky share, `hemi_value`, which a forward-drawn model is lit by.
  @location(8) @interpolate(flat) sky: f32,
  // A forward-drawn model's light, which its vertex shader lights it by as the engine's does.
  @location(9) light: vec3<f32>,
};

struct GBufferOutput {
  // Albedo, then gloss.
  @location(0) albedo: vec4<f32>,
  // The view space normal, octahedral.
  @location(1) normal: vec2<f32>,
  // Hemisphere, sun, material slice.
  @location(2) material: vec4<f32>,
};

// A vertex placed in the world, swayed by as much of the wind as its rigidity takes: none for a sector's geometry.
fn place_vertex(pulled: PulledVertex, position: vec3<f32>, normal: vec4<f32>, tangent: vec4<f32>, binormal: vec4<f32>,
  rigidity: f32) -> GBufferVarying {
  let place: Place = pulled.place;
  let matrix: mat4x4<f32> = place_matrix(place);
  let world: vec4<f32> = vec4<f32>(swayed((matrix * vec4<f32>(position, 1.0)).xyz, place.m3.y, rigidity), 1.0);
  let linear: mat3x3<f32> = mat3x3<f32>(place.m0.xyz, place.m1.xyz, place.m2.xyz);
  // The inverse transpose of a matrix without shear: each axis divided by its squared length.
  let scale: vec3<f32> = vec3<f32>(dot(place.m0.xyz, place.m0.xyz), dot(place.m1.xyz, place.m1.xyz),
    dot(place.m2.xyz, place.m2.xyz));
  let view: mat3x3<f32> = mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz);
  var out: GBufferVarying;

  out.clip = camera.view_projection * world;
  out.normal = normalize(view * (linear * (unpack_direction(normal) / scale)));
  out.tangent = view * (linear * unpack_direction(tangent));
  out.binormal = view * (linear * unpack_direction(binormal));
  out.hemi = normal.w * place.info.x + place.info.y;
  out.surface = pulled.surface;
  out.entry = pulled.entry;
  out.sky = OPEN_SKY;
  out.light = vec3<f32>(0.0);

  return out;
}

@vertex
fn vs_baked(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> GBufferVarying {
  let pulled: PulledVertex = pull(vertex_index, instance_index);
  let at: u32 = pulled.word;
  let binormal: vec4<f32> = unpack4x8unorm(words[at]);
  let tangent: vec4<f32> = unpack4x8unorm(words[at + 2u]);
  let position: vec3<f32> = vec3<f32>(bitcast<f32>(words[at + 5u]), bitcast<f32>(words[at + 6u]),
    bitcast<f32>(words[at + 7u]));
  var out: GBufferVarying = place_vertex(pulled, position, unpack4x8unorm(words[at + 1u]), tangent, binormal, 0.0);

  // The base coordinate's fraction rides in the tangent's and binormal's fourth bytes.
  out.uv = (unpack_shorts(words[at + 3u]) + vec2<f32>(tangent.w, binormal.w)) / 1024.0;
  out.lightmap_uv = unpack_shorts(words[at + 4u]) / 32768.0;

  return out;
}

@vertex
fn vs_tree(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> GBufferVarying {
  let pulled: PulledVertex = pull(vertex_index, instance_index);
  let at: u32 = pulled.word;
  let position: vec3<f32> = vec3<f32>(bitcast<f32>(words[at + 5u]), bitcast<f32>(words[at + 6u]),
    bitcast<f32>(words[at + 7u]));
  // The coordinate's second pair carries the rigidity, scaled as `consts.x` scales it.
  let rigidity: f32 = unpack_shorts(words[at + 4u]).x / 2048.0;
  var out: GBufferVarying = place_vertex(pulled, position, unpack4x8unorm(words[at + 1u]),
    unpack4x8unorm(words[at + 2u]), unpack4x8unorm(words[at]), rigidity);

  out.uv = unpack_shorts(words[at + 3u]) / 2048.0;
  out.lightmap_uv = vec2<f32>(0.0);

  return out;
}

// A spawned model's vertex: no sway, the base coordinate as two floats, no lightmap, and the hemisphere its place's
// cube gives along its normal where the level's lighting was estimated for it.
@vertex
fn vs_model(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> GBufferVarying {
  return model_vertex(pull(vertex_index, instance_index));
}

// A spawned model's vertex as it stands in its object's place, unswayed.
fn model_position(pulled: PulledVertex) -> vec3<f32> {
  let at: u32 = pulled.word;

  return vec3<f32>(bitcast<f32>(words[at + 5u]), bitcast<f32>(words[at + 6u]), bitcast<f32>(words[at + 7u]));
}

fn model_vertex(pulled: PulledVertex) -> GBufferVarying {
  let at: u32 = pulled.word;
  let position: vec3<f32> = model_position(pulled);
  let normal: vec4<f32> = unpack4x8unorm(words[at + 1u]);
  var out: GBufferVarying = place_vertex(pulled, position, normal, unpack4x8unorm(words[at + 2u]),
    unpack4x8unorm(words[at]), 0.0);
  let place: Place = pulled.place;

  out.uv = vec2<f32>(bitcast<f32>(words[at + 3u]), bitcast<f32>(words[at + 4u]));
  out.lightmap_uv = vec2<f32>(0.0);

  if (place.cube.w != 0u) {
    let linear: mat3x3<f32> = mat3x3<f32>(place.m0.xyz, place.m1.xyz, place.m2.xyz);

    out.hemi = cube_hemi(place.cube, normalize(linear * unpack_direction(normal)));
    out.sky = bitcast<f32>(place.cube.z);
  }

  return out;
}

// The hemisphere a cube gives along a world normal, each axis's face on the side it points to weighed by how far it
// points along it.
fn cube_hemi(cube: vec4<u32>, normal: vec3<f32>) -> f32 {
  let first: vec4<f32> = unpack4x8unorm(cube.x);
  let second: vec4<f32> = unpack4x8unorm(cube.y);
  let positive: vec3<f32> = first.xyz;
  let negative: vec3<f32> = vec3<f32>(first.w, second.x, second.y);

  return saturate(dot(select(positive, negative, normal < vec3<f32>(0.0)), abs(normal)));
}

// The base coordinate's and the lightmap coordinate's derivatives, taken where control flow is uniform: a surface's
// textures are sampled under its own flags, where no implicit derivative may be taken.
struct Footprint {
  uv: vec2<f32>,
  dx: vec2<f32>,
  dy: vec2<f32>,
  lightmap: vec2<f32>,
  lightmap_dx: vec2<f32>,
  lightmap_dy: vec2<f32>,
};

fn take_footprint(in: GBufferVarying) -> Footprint {
  let uv: vec2<f32> = in.uv * surfaces[in.surface].tiling;

  return Footprint(uv, dpdx(uv), dpdy(uv), in.lightmap_uv, dpdx(in.lightmap_uv), dpdy(in.lightmap_uv));
}

fn sample_slot(slot: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec4<f32> {
  return textureSampleGrad(textures[slot], texture_sampler, uv, dx, dy);
}

fn base_texel(in: GBufferVarying, at: Footprint) -> vec4<f32> {
  let surface: Surface = surfaces[in.surface];

  if ((surface.flags & SURFACE_HAS_BASE) == 0u) {
    return vec4<f32>(1.0);
  }

  return sample_slot(surface.base, at.uv, at.dx, at.dy);
}

fn shade(in: GBufferVarying, base: vec4<f32>, at: Footprint) -> GBufferOutput {
  let surface: Surface = surfaces[in.surface];
  let scale: f32 = surface.detail_scale;
  let detail_uv: vec2<f32> = at.uv * scale;
  let detail_dx: vec2<f32> = at.dx * scale;
  let detail_dy: vec2<f32> = at.dy * scale;
  let is_textured: f32 = camera.switches.x;
  let is_bumped: f32 = camera.switches.y;
  var diffuse: vec3<f32> = base.rgb;
  var normal: vec3<f32> = normalize(in.normal);
  var gloss: f32 = DEFAULT_GLOSS;
  var detail: vec4<f32> = vec4<f32>(0.5);

  if ((surface.flags & SURFACE_HAS_DETAIL) != 0u) {
    detail = sample_slot(surface.detail, detail_uv, detail_dx, detail_dy);
    diffuse *= detail.rgb * 2.0;
  }

  if ((surface.flags & SURFACE_HAS_BUMP) != 0u) {
    let bump: vec4<f32> = sample_slot(surface.bump, at.uv, at.dx, at.dy);
    var tangent_normal: vec3<f32> = bump.wzy + sample_slot(surface.bump_companion, at.uv, at.dx, at.dy).xyz - 1.0;
    var bumped_gloss: f32 = bump.x * bump.x;

    if ((surface.flags & SURFACE_HAS_DETAIL_BUMP) != 0u) {
      let detail_bump: vec4<f32> = sample_slot(surface.detail_bump, detail_uv, detail_dx, detail_dy);
      let companion: vec4<f32> = sample_slot(surface.detail_bump_companion, detail_uv, detail_dx, detail_dy);

      tangent_normal += detail_bump.wzy + companion.xyz - 1.0;
      bumped_gloss *= detail_bump.x * 2.0;
    } else if ((surface.flags & SURFACE_HAS_DETAIL) != 0u) {
      bumped_gloss *= detail.a * 2.0;
    }

    let bumped: vec3<f32> = normalize(in.tangent * tangent_normal.x + in.binormal * tangent_normal.y
      + in.normal * tangent_normal.z);
    let weight: f32 = is_bumped * is_textured;

    normal = normalize(mix(normal, bumped, weight));
    gloss = mix(DEFAULT_GLOSS, bumped_gloss, weight);
  }

  var hemi: f32 = in.hemi;
  var sun: f32 = 1.0;

  if ((surface.flags & SURFACE_HAS_HEMI) != 0u) {
    let lightmap: vec4<f32> = sample_slot(surface.hemi, at.lightmap, at.lightmap_dx, at.lightmap_dy);

    hemi = lightmap.a;
    sun = lightmap.g;
  }

  var out: GBufferOutput;

  out.albedo = vec4<f32>(mix(surface.color, diffuse, is_textured), gloss);
  out.normal = octahedral_encode(normal);
  out.material = vec4<f32>(hemi, sun, surface.slice, 0.0);

  return out;
}

@fragment
fn fs_opaque(in: GBufferVarying) -> GBufferOutput {
  let at: Footprint = take_footprint(in);

  return shade(in, base_texel(in, at), at);
}

// Whether a cut-out texel is cut, as `common/cut_out` cuts one.
fn is_cut(in: GBufferVarying, base: vec4<f32>, at: Footprint) -> bool {
  let surface: Surface = surfaces[in.surface];

  return is_alpha_cut(base.a, vec2<f32>(textureDimensions(textures[surface.base])), at.dx, at.dy,
    surface.alpha_reference);
}

@fragment
fn fs_cut_out(in: GBufferVarying) -> GBufferOutput {
  let at: Footprint = take_footprint(in);
  let base: vec4<f32> = base_texel(in, at);

  if (is_cut(in, base, at)) {
    discard;
  }

  return shade(in, base, at);
}

// A pick's texel: what was drawn, by its cluster and place, and its depth's bits.
fn pick_texel(in: GBufferVarying) -> vec4<u32> {
  return vec4<u32>(1u, in.entry.x, in.entry.y, bitcast<u32>(in.clip.z));
}

@fragment
fn fs_pick_opaque(in: GBufferVarying) -> @location(0) vec4<u32> {
  return pick_texel(in);
}

// Cut as the G-buffer cuts it, so a click between leaves picks what is behind them.
@fragment
fn fs_pick_cut_out(in: GBufferVarying) -> @location(0) vec4<u32> {
  let at: Footprint = take_footprint(in);

  if (is_cut(in, base_texel(in, at), at)) {
    discard;
  }

  return pick_texel(in);
}

// A cut-out caster in a shadow's map: depth alone, cut as the G-buffer cuts it, so light falls between its leaves.
@fragment
fn fs_shadow_cut_out(in: GBufferVarying) {
  let at: Footprint = take_footprint(in);

  if (is_cut(in, base_texel(in, at), at)) {
    discard;
  }
}

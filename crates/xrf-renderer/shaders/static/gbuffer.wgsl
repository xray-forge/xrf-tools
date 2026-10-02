enable wgpu_binding_array;

#import "common/camera"
#import "common/octahedral"
#import "static/records"

// Static draws into the G-buffer: every visible cluster of a batch is one instance of `CLUSTER_VERTICES` vertices,
// pulled from its layout's arena through the shared index arena.

@group(1) @binding(0) var<storage, read> clusters: array<Cluster>;
@group(1) @binding(1) var<storage, read> slots: array<Slot>;
@group(1) @binding(2) var<storage, read> places: array<Place>;
@group(1) @binding(3) var<storage, read> surfaces: array<Surface>;
@group(1) @binding(4) var<storage, read> indices: array<u32>;
@group(1) @binding(5) var<storage, read> lists: array<vec2<u32>>;
@group(1) @binding(6) var<storage, read> words: array<u32>;

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var texture_sampler: sampler;

// What `def_gloss` writes for a surface without a bump: 2 of 255.
const DEFAULT_GLOSS: f32 = 2.0 / 255.0;

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
};

struct GBufferOutput {
  // Albedo, then gloss.
  @location(0) albedo: vec4<f32>,
  // The view space normal, octahedral.
  @location(1) normal: vec2<f32>,
  // Hemisphere, sun, material slice.
  @location(2) material: vec4<f32>,
};

// Two shorts from a word, the low half first, sign extended.
fn unpack_shorts(word: u32) -> vec2<f32> {
  return vec2<f32>(f32(bitcast<i32>(word << 16u) >> 16u), f32(bitcast<i32>(word) >> 16u));
}

// A direction packed as a `D3DCOLOR`: blue, green, red bytes as z, y, x.
fn unpack_direction(packed: vec4<f32>) -> vec3<f32> {
  return packed.zyx * 2.0 - 1.0;
}

struct PulledVertex {
  word: u32,
  place: Place,
  surface: u32,
  entry: vec2<u32>,
};

fn pull(vertex_index: u32, instance_index: u32) -> PulledVertex {
  let entry: vec2<u32> = lists[instance_index];
  let cluster: Cluster = clusters[entry.x];
  // A cluster short of 128 triangles collapses its tail onto its last corner.
  let corner: u32 = min(vertex_index, cluster.triangles * 3u - 1u);
  let vertex: u32 = cluster.vertex_start + indices[cluster.first_index + corner];

  return PulledVertex(vertex * 8u, places[entry.y], slots[cluster.slot].surface, entry);
}

fn place_vertex(pulled: PulledVertex, position: vec3<f32>, normal: vec4<f32>, tangent: vec4<f32>, binormal: vec4<f32>)
  -> GBufferVarying {
  let place: Place = pulled.place;
  let matrix: mat4x4<f32> = place_matrix(place);
  let world: vec4<f32> = matrix * vec4<f32>(position, 1.0);
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
  var out: GBufferVarying = place_vertex(pulled, position, unpack4x8unorm(words[at + 1u]), tangent, binormal);

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
  var out: GBufferVarying = place_vertex(pulled, position, unpack4x8unorm(words[at + 1u]),
    unpack4x8unorm(words[at + 2u]), unpack4x8unorm(words[at]));

  // todo: Sway a tree in the wind, by the rigidity its coordinate's second pair carries, once weather drives wind.
  out.uv = unpack_shorts(words[at + 3u]) / 2048.0;
  out.lightmap_uv = vec2<f32>(0.0);

  return out;
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

// Whether a cut-out texel survives: its alpha raised where minification thins it out, cut along a ramp one texel wide
// so the edge does not shimmer.
fn is_cut(in: GBufferVarying, base: vec4<f32>, at: Footprint) -> bool {
  let surface: Surface = surfaces[in.surface];
  let size: vec2<f32> = vec2<f32>(textureDimensions(textures[surface.base]));
  let extent: f32 = max(dot(at.dx * size, at.dx * size), dot(at.dy * size, at.dy * size));
  let alpha: f32 = base.a * (1.0 + 0.25 * max(0.0, 0.5 * log2(max(extent, 1e-8))));
  let width: f32 = max(abs(dpdx(alpha)) + abs(dpdy(alpha)), 1.0 / 255.0);

  return saturate((alpha - surface.alpha_reference) / width + 0.5) <= 0.5;
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

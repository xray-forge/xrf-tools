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

// Whether this pipeline draws a shadow map, whose surfaces casting none are collapsed to nothing.
override IS_SHADOW_DRAW: bool = false;

// The untextured grey a wireframe draws every edge with, lit as a surface is.
const WIRE_COLOR: vec3<f32> = vec3<f32>(0.75);

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
  // Which corner of its triangle the vertex is, one-hot, which a wireframe finds the edges by.
  @location(10) barycentric: vec3<f32>,
  // Where the point stands in the world, and how far it stood from there the frame before, which its motion is read by.
  @location(11) world: vec3<f32>,
  @location(12) moved: vec3<f32>,
};

struct GBufferOutput {
  // Albedo, then gloss.
  @location(0) albedo: vec4<f32>,
  // The view space normal, octahedral.
  @location(1) normal: vec2<f32>,
  // Hemisphere, sun, material slice.
  @location(2) material: vec4<f32>,
  // How far the point moved on the screen since the last frame, as `camera_motion` measures it.
  @location(3) motion: vec2<f32>,
};

// A vertex placed in the world, swayed by as much of the wind as its rigidity takes: none for a sector's geometry.
fn place_vertex(pulled: PulledVertex, position: vec3<f32>, normal: vec4<f32>, tangent: vec4<f32>, binormal: vec4<f32>,
  rigidity: f32) -> GBufferVarying {
  let place: Place = pulled.place;
  let matrix: mat4x4<f32> = place_matrix(place);
  let placed: vec3<f32> = (matrix * vec4<f32>(position, 1.0)).xyz;
  let world: vec4<f32> = vec4<f32>(swayed(placed, place.m3.y, rigidity), 1.0);
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
  out.barycentric = vec3<f32>(0.0);
  out.world = world.xyz;
  out.moved = swayed_before(placed, place.m3.y, rigidity) - world.xyz;

  // A script declaring no shadow element leaves the surface out of every shadow map (`_lua_Compile`'s `E[2]`).
  if (IS_SHADOW_DRAW && (surfaces[pulled.surface].flags & SURFACE_IS_SHADOWLESS) != 0u) {
    out.clip = vec4<f32>(0.0);
  }

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
  out.barycentric = corner_barycentric(vertex_index);

  return out;
}

@vertex
fn vs_tree(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> GBufferVarying {
  let pulled: PulledVertex = pull(vertex_index, instance_index);
  let at: u32 = pulled.word;
  let position: vec3<f32> = vec3<f32>(bitcast<f32>(words[at + 5u]), bitcast<f32>(words[at + 6u]),
    bitcast<f32>(words[at + 7u]));
  // The coordinate's second pair carries the rigidity, scaled as `consts.x` scales it; an object stored as a tree
  // stands still, as `tree_s` draws it.
  let is_still: bool = (surfaces[pulled.surface].flags & SURFACE_IS_STILL) != 0u;
  let rigidity: f32 = select(unpack_shorts(words[at + 4u]).x / 2048.0, 0.0, is_still);
  var out: GBufferVarying = place_vertex(pulled, position, unpack4x8unorm(words[at + 1u]),
    unpack4x8unorm(words[at + 2u]), unpack4x8unorm(words[at]), rigidity);

  out.uv = unpack_shorts(words[at + 3u]) / 2048.0;
  out.lightmap_uv = vec2<f32>(0.0);
  out.barycentric = corner_barycentric(vertex_index);

  return out;
}

// A spawned model's vertex: no sway, the base coordinate as two floats, no lightmap, and the hemisphere its place's
// cube gives along its normal where the level's lighting was estimated for it.
@vertex
fn vs_model(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> GBufferVarying {
  var out: GBufferVarying = model_vertex(pull(vertex_index, instance_index));

  out.barycentric = corner_barycentric(vertex_index);

  return out;
}

// Which corner of its triangle a pulled vertex is: a cluster's vertices run three a triangle.
fn corner_barycentric(vertex_index: u32) -> vec3<f32> {
  let corner: u32 = vertex_index % 3u;

  return vec3<f32>(f32(corner == 0u), f32(corner == 1u), f32(corner == 2u));
}

// Whether a wireframe leaves a fragment out: all but those within a pixel of their triangle's edges. Its derivatives
// are taken before any branch.
fn is_off_wire(in: GBufferVarying) -> bool {
  let width: vec3<f32> = max(fwidth(in.barycentric), vec3<f32>(1e-6));
  let reach: vec3<f32> = in.barycentric / width;

  return camera.modes.x > 0.5 && min(reach.x, min(reach.y, reach.z)) > 1.0;
}

// A surface as a wireframe draws it: its edges in one untextured grey, lit as it is.
fn wired(shaded: GBufferOutput) -> GBufferOutput {
  var out: GBufferOutput = shaded;

  if (camera.modes.x > 0.5) {
    out.albedo = vec4<f32>(WIRE_COLOR, DEFAULT_GLOSS);
  }

  return out;
}

// A spawned model's vertex as it stands in its object's place, unswayed.
fn model_position(pulled: PulledVertex) -> vec3<f32> {
  let at: u32 = pulled.word;

  return vec3<f32>(bitcast<f32>(words[at + 5u]), bitcast<f32>(words[at + 6u]), bitcast<f32>(words[at + 7u]));
}

// A model's vertex as its bones stand it, this frame's or the last's: each linked bone's matrix applied, weighted.
struct SkinnedVertex {
  position: vec3<f32>,
  normal: vec4<f32>,
  tangent: vec4<f32>,
  binormal: vec4<f32>,
};

// One bone's row-major 3x4 applied to a point, or to a direction where `w` is zero.
fn apply_bone(first: u32, point: vec4<f32>) -> vec3<f32> {
  return vec3<f32>(dot(bones[first], point), dot(bones[first + 1u], point), dot(bones[first + 2u], point));
}

// A direction packed as a `D3DCOLOR` turned by the bones, packed again, its fourth byte kept.
fn skin_direction(packed: vec4<f32>, ids: vec4<u32>, weights: vec4<f32>, base: u32) -> vec4<f32> {
  let direction: vec4<f32> = vec4<f32>(unpack_direction(packed), 0.0);
  var turned: vec3<f32> = vec3<f32>(0.0);

  for (var link: u32 = 0u; link < 4u; link++) {
    turned += apply_bone(base + ids[link] * 3u, direction) * weights[link];
  }

  let unit: vec3<f32> = normalize(turned + vec3<f32>(0.0, 0.0, 1e-6));

  return vec4<f32>(unit.zyx * 0.5 + 0.5, packed.w);
}

// A model's vertex hung from its place's bones, `is_previous` reading the last frame's matrices; a rigid model, or a
// vertex hanging from nothing, stands as stored.
fn skin_vertex(pulled: PulledVertex, is_previous: bool) -> SkinnedVertex {
  let at: u32 = pulled.word;
  let stored: SkinnedVertex = SkinnedVertex(model_position(pulled), unpack4x8unorm(words[at + 1u]),
    unpack4x8unorm(words[at + 2u]), unpack4x8unorm(words[at]));
  let skin: vec4<u32> = pulled.place.skin;

  if (skin.w == 0u) {
    return stored;
  }

  let link: u32 = (skin.y + at / 8u - skin.z) * 2u;
  let packed_ids: u32 = skins[link];
  let ids: vec4<u32> = vec4<u32>(packed_ids & 255u, (packed_ids >> 8u) & 255u, (packed_ids >> 16u) & 255u,
    packed_ids >> 24u);
  let raw: vec4<f32> = unpack4x8unorm(skins[link + 1u]);
  let total: f32 = raw.x + raw.y + raw.z + raw.w;

  if (total <= 0.0) {
    return stored;
  }

  let weights: vec4<f32> = raw / total;
  // The last frame's matrices follow this frame's, as many again.
  let base: u32 = skin.x + select(0u, skin.w * 3u, is_previous);
  var position: vec3<f32> = vec3<f32>(0.0);

  for (var index: u32 = 0u; index < 4u; index++) {
    position += apply_bone(base + ids[index] * 3u, vec4<f32>(stored.position, 1.0)) * weights[index];
  }

  return SkinnedVertex(position, skin_direction(stored.normal, ids, weights, base),
    skin_direction(stored.tangent, ids, weights, base), skin_direction(stored.binormal, ids, weights, base));
}

fn model_vertex(pulled: PulledVertex) -> GBufferVarying {
  let skinned: SkinnedVertex = skin_vertex(pulled, false);
  var out: GBufferVarying = place_vertex(pulled, skinned.position, skinned.normal, skinned.tangent,
    skinned.binormal, 0.0);
  let place: Place = pulled.place;
  let at: u32 = pulled.word;
  let normal: vec4<f32> = skinned.normal;

  if (place.skin.w != 0u) {
    // Where the bones stood it the frame before, for its motion.
    let previous: vec3<f32> = (place_matrix(place) * vec4<f32>(skin_vertex(pulled, true).position, 1.0)).xyz;

    out.moved = previous - out.world;
  }

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

// The uv checker an asset viewer draws in place of a surface's textures: eight texels a side, white and dark, each
// repeat of the coordinate.
fn checker_texel(uv: vec2<f32>) -> vec4<f32> {
  let cell: vec2<f32> = floor(uv * camera.modes.y * 8.0);

  return vec4<f32>(vec3<f32>(select(64.0 / 255.0, 1.0, abs((cell.x + cell.y) % 2.0) < 0.5)), 1.0);
}

fn base_texel(in: GBufferVarying, at: Footprint) -> vec4<f32> {
  let surface: Surface = surfaces[in.surface];

  if (camera.modes.y > 0.0) {
    return checker_texel(in.uv);
  }

  if ((surface.flags & SURFACE_HAS_BASE) == 0u) {
    return select(vec4<f32>(1.0), vec4<f32>(camera.plain.rgb, 1.0), camera.plain.w > 0.5);
  }

  return sample_slot(surface.base, at.uv, at.dx, at.dy);
}

// What a terrain lays over its base (`deffer_impl_flat` with `USE_4_DETAIL` and `USE_4_BUMP`): its four details,
// their bumps in tangent space at twice the contrast, and their gloss, each weighed by its mask's channel over the
// mask's sum.
struct TerrainTexel {
  detail: vec3<f32>,
  bump: vec3<f32>,
  gloss: f32,
};

fn terrain_texel(surface: Surface, at: Footprint, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> TerrainTexel {
  let mask: vec4<f32> = sample_slot(surface.terrain_mask, at.uv, at.dx, at.dy);
  let weights: vec4<f32> = mask / max(dot(mask, vec4<f32>(1.0)), 1e-4);
  var out: TerrainTexel = TerrainTexel(vec3<f32>(0.0), vec3<f32>(0.0), 0.0);

  for (var layer: u32 = 0u; layer < 4u; layer++) {
    let weight: f32 = weights[layer];
    // `.wzyx`: the normal in the last three channels, the gloss in the first.
    let bump: vec4<f32> = sample_slot(surface.terrain_bumps[layer], uv, dx, dy).wzyx;

    out.detail += sample_slot(surface.terrain_details[layer], uv, dx, dy).rgb * weight;
    out.bump += (bump.xyz - 0.5) * weight;
    out.gloss += bump.w * weight;
  }

  out.bump.z *= 0.5;

  return out;
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

  let is_terrain: bool = (surface.flags & SURFACE_IS_TERRAIN) != 0u;

  if (is_terrain) {
    let terrain: TerrainTexel = terrain_texel(surface, at, detail_uv, detail_dx, detail_dy);
    let weight: f32 = is_bumped * is_textured;

    diffuse *= terrain.detail * 2.0;
    normal = normalize(mix(normal, normalize(in.tangent * terrain.bump.x + in.binormal * terrain.bump.y +
      in.normal * terrain.bump.z), weight));
    gloss = mix(DEFAULT_GLOSS, terrain.gloss, weight);
  } else if ((surface.flags & SURFACE_HAS_DETAIL) != 0u) {
    detail = sample_slot(surface.detail, detail_uv, detail_dx, detail_dy);
    diffuse *= detail.rgb * 2.0;
  }

  if (!is_terrain && (surface.flags & SURFACE_HAS_BUMP) != 0u) {
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

  // A terrain is lit by its base's alpha, `deffer_impl_flat`'s `Ne.w = D.w`.
  var hemi: f32 = select(in.hemi, base.a, is_terrain);
  var sun: f32 = 1.0;

  if ((surface.flags & SURFACE_HAS_HEMI) != 0u) {
    let lightmap: vec4<f32> = sample_slot(surface.hemi, at.lightmap, at.lightmap_dx, at.lightmap_dy);

    hemi = lightmap.a;
    sun = lightmap.g;
  }

  var out: GBufferOutput;

  out.albedo = vec4<f32>(mix(untextured_color(surface.color), diffuse, is_textured), gloss);
  out.normal = octahedral_encode(normal);
  // Its alpha holds the marks: what is selected, which the present pass outlines, and what is self-lit.
  out.material = vec4<f32>(hemi, sun, surface.slice,
    encode_marks(is_selected(in.entry), (surface.flags & SURFACE_IS_EMISSIVE) != 0u));
  out.motion = camera_motion(in.world, in.world + in.moved);

  return out;
}

@fragment
fn fs_opaque(in: GBufferVarying) -> GBufferOutput {
  let at: Footprint = take_footprint(in);

  if (is_off_wire(in)) {
    discard;
  }

  return wired(shade(in, base_texel(in, at), at));
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
  let is_off: bool = is_off_wire(in);

  // A wireframe draws a cut-out surface's every edge, whatever its alpha; so does a view drawing every surface solid.
  if (is_off || (camera.modes.x < 0.5 && camera.modes.z < 0.5 && is_cut(in, base, at))) {
    discard;
  }

  return wired(shade(in, base, at));
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

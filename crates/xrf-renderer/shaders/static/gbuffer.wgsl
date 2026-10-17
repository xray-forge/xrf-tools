enable wgpu_binding_array;

#import "common/camera"
#import "common/relief"
#import "common/cut_out"
#import "common/octahedral"
#import "common/gloss"
#import "common/debug_colours"
#import "common/luminance"
#import "static/pulling"

// Static draws into the G-buffer, pulled as `static/pulling` reads them.

@group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var texture_sampler: sampler;
// Trilinear whatever the preset's anisotropy, for the bumps and the hemisphere's lightmap: low-frequency maps whose
// anisotropic samples cost more than they show.
@group(1) @binding(2) var trilinear_sampler: sampler;

// Whether this pipeline draws a shadow map, whose surfaces casting none are collapsed to nothing.
override IS_SHADOW_DRAW: bool = false;
// Whether the frame may inspect its surfaces (a wireframe, the asset checker, alpha hidden, the parallax, bumps or
// heights drawn for their debug views); a frame doing none of them drops their code, which the frame's registers
// carry otherwise.
override IS_INSPECTED: bool = true;

// The untextured grey a wireframe draws every edge with, lit as a surface is.
const WIRE_COLOR: vec3<f32> = vec3<f32>(0.75);

// The sky share of an object under open sky, `ps_r2_dhemi_sky_scale`, which one without an estimate is lit by.
const OPEN_SKY: f32 = 0.08;

struct GBufferVarying {
  // Invariant, so a cut-out's depth-only draw and its shading draw meet the same depth exactly.
  @builtin(position) @invariant clip: vec4<f32>,
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
  // Whether a tree's leaves are lit as foliage, the wetness, which bends their normal up harder, and whether these are
  // leaves at all.
  @location(13) @interpolate(flat) flora: vec3<f32>,
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
  // How far the surface sinks below its relief's top where the enhanced parallax marches it, and how far below the top
  // water there must stand for its reflection to be cut by the relief, from one (`ViewTargets::RELIEF`).
  @location(4) relief: vec2<f32>,
};

// A vertex placed in the world, swayed by as much of the wind as its rigidity takes, or by the enhanced motion as its
// `foliage` kind and texture `v` say: none for a sector's geometry.
fn place_vertex(pulled: PulledVertex, position: vec3<f32>, normal: vec4<f32>, tangent: vec4<f32>, binormal: vec4<f32>,
  rigidity: f32, foliage: vec2<f32>) -> GBufferVarying {
  let place: Place = pulled.place;
  let matrix: mat4x4<f32> = place.transform;
  let placed: vec3<f32> = (matrix * vec4<f32>(position, 1.0)).xyz;
  let world: vec4<f32> = vec4<f32>(swayed(placed, place.transform[3].y, rigidity, foliage), 1.0);
  let linear: mat3x3<f32> = mat3x3<f32>(place.transform[0].xyz, place.transform[1].xyz, place.transform[2].xyz);
  // The inverse transpose of a matrix without shear: each axis divided by its squared length.
  let scale: vec3<f32> = vec3<f32>(dot(place.transform[0].xyz, place.transform[0].xyz), dot(place.transform[1].xyz, place.transform[1].xyz),
    dot(place.transform[2].xyz, place.transform[2].xyz));
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
  out.flora = vec3<f32>(0.0);
  out.moved = swayed_before(placed, place.transform[3].y, rigidity, foliage) - world.xyz;

  // A script declaring no shadow element leaves the surface out of every shadow map (`_lua_Compile`'s `E[2]`).
  if (IS_SHADOW_DRAW && (surfaces[pulled.surface].flags & SURFACE_IS_SHADOWLESS) != 0u) {
    out.clip = vec4<f32>(0.0);
  }

  return out;
}

@vertex
fn vs_baked(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> GBufferVarying {
  return baked_vertex(pull(vertex_index, instance_index), vertex_index);
}

// A sector's or an instanced mesh's vertex, its base and lightmap coordinates as the baked layout packs them.
fn baked_vertex(pulled: PulledVertex, vertex_index: u32) -> GBufferVarying {
  let at: u32 = pulled.word;
  let binormal: vec4<f32> = unpack4x8unorm(words[at + VERTEX_BINORMAL]);
  let tangent: vec4<f32> = unpack4x8unorm(words[at + VERTEX_TANGENT]);
  let position: vec3<f32> = static_position(at);
  var out: GBufferVarying = place_vertex(pulled, position, unpack4x8unorm(words[at + VERTEX_NORMAL]), tangent, binormal,
    0.0, vec2<f32>(FOLIAGE_NONE, 0.0));

  out.uv = baked_uv(words[at + VERTEX_UV], tangent, binormal);
  out.lightmap_uv = baked_lightmap_uv(words[at + VERTEX_EXTRA]);
  out.barycentric = corner_barycentric(vertex_index);

  return out;
}

@vertex
fn vs_tree(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> GBufferVarying {
  let pulled: PulledVertex = pull(vertex_index, instance_index);
  let at: u32 = pulled.word;
  let position: vec3<f32> = static_position(at);
  // The coordinate's second pair carries the rigidity, scaled as `consts.x` scales it; an object stored as a tree
  // stands still, as `tree_s` draws it.
  let is_still: bool = (surfaces[pulled.surface].flags & SURFACE_IS_STILL) != 0u;
  let rigidity: f32 = select(unpack_shorts(words[at + VERTEX_EXTRA]).x / 2048.0, 0.0, is_still);
  let uv: vec2<f32> = unpack_shorts(words[at + VERTEX_UV]) / 2048.0;
  // The enhanced motion swings a trunk and tosses a cut-out branch or leaf, as `tree_branch` draws those.
  let is_branch: bool = (surfaces[pulled.surface].flags & SURFACE_IS_CUT_OUT) != 0u;
  let kind: f32 = tree_foliage_kind(is_still, is_branch);
  var out: GBufferVarying = place_vertex(pulled, position, unpack4x8unorm(words[at + VERTEX_NORMAL]),
    unpack4x8unorm(words[at + VERTEX_TANGENT]), unpack4x8unorm(words[at + VERTEX_BINORMAL]), rigidity,
    vec2<f32>(kind, uv.y));

  // Under the enhanced motion a crown's leaves are lit as a sphere about its foot rather than by their cards, and never
  // darker than three tenths of the sky.
  if (kind == FOLIAGE_BRANCH && is_foliage_enhanced(wind, vec2<f32>(kind, uv.y))) {
    let place: Place = pulled.place;
    let linear: mat3x3<f32> = mat3x3<f32>(place.transform[0].xyz, place.transform[1].xyz, place.transform[2].xyz);
    let view: mat3x3<f32> = mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz);
    let n: vec3<f32> = normalize(position + vec3<f32>(0.0, 1e-4, 0.0));
    let axis: vec3<f32> = select(vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 1.0, 0.0), abs(n.z) > 0.99);
    let t: vec3<f32> = normalize(cross(n, axis));

    out.normal = normalize(view * (linear * n));
    out.tangent = view * (linear * t);
    out.binormal = view * (linear * normalize(cross(n, t)));
    out.hemi = clamp(out.hemi, 0.3, 1.0);
    out.flora.z = 1.0;

    if (wind.foliage_flora.w > 0.5) {
      out.flora = vec3<f32>(1.0, wind.foliage_flora.z, 1.0);
    }
  }

  out.uv = uv;
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

  return IS_INSPECTED && camera.modes.x > 0.5 && min(reach.x, min(reach.y, reach.z)) > 1.0;
}

// A surface as a wireframe draws it: its edges in one untextured grey, lit as it is.
fn wired(shaded: GBufferOutput) -> GBufferOutput {
  var out: GBufferOutput = shaded;

  if (IS_INSPECTED && camera.modes.x > 0.5) {
    out.albedo = vec4<f32>(WIRE_COLOR, DEFAULT_GLOSS);
  }

  return out;
}

// A vertex's position as its arena holds it, at its first word.
fn static_position(at: u32) -> vec3<f32> {
  return vertex_position(words[at + VERTEX_POSITION], words[at + VERTEX_POSITION + 1u], words[at + VERTEX_POSITION + 2u]);
}

// A spawned model's vertex as it stands in its object's place, unswayed.
fn model_position(pulled: PulledVertex) -> vec3<f32> {
  return static_position(pulled.word);
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
  let stored: SkinnedVertex = SkinnedVertex(model_position(pulled), unpack4x8unorm(words[at + VERTEX_NORMAL]),
    unpack4x8unorm(words[at + VERTEX_TANGENT]), unpack4x8unorm(words[at + VERTEX_BINORMAL]));
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
    skinned.binormal, 0.0, vec2<f32>(FOLIAGE_NONE, 0.0));
  let place: Place = pulled.place;
  let at: u32 = pulled.word;
  let normal: vec4<f32> = skinned.normal;

  if (place.skin.w != 0u) {
    // Where the bones stood it the frame before, for its motion.
    let previous: vec3<f32> = (place.transform * vec4<f32>(skin_vertex(pulled, true).position, 1.0)).xyz;

    out.moved = previous - out.world;
  }

  out.uv = vec2<f32>(bitcast<f32>(words[at + VERTEX_UV]), bitcast<f32>(words[at + VERTEX_EXTRA]));
  out.lightmap_uv = vec2<f32>(0.0);

  if (place.cube.w != 0u) {
    let linear: mat3x3<f32> = mat3x3<f32>(place.transform[0].xyz, place.transform[1].xyz, place.transform[2].xyz);

    out.hemi = cube_hemi(place.cube, normalize(linear * unpack_direction(normal)));
    out.sky = bitcast<f32>(place.cube.z);
  }

  return out;
}

// The base coordinate's and the lightmap coordinate's derivatives, taken where control flow is uniform: a surface's
// textures are sampled under its own flags, where no implicit derivative may be taken.
struct Footprint {
  uv: vec2<f32>,
  // The base coordinate before any parallax moved it, which the detail is read at, as the engines leave it.
  origin: vec2<f32>,
  // Where the enhanced parallax marched a steep surface: how far its low ground darkens, how much of the sun its
  // relief keeps from its hollows, and how far the point sinks below its relief's top; none, all and none, and
  // unmarked, where it did not.
  cavity: f32,
  relief: f32,
  sink: f32,
  clearance: f32,
  is_relief: bool,
  dx: vec2<f32>,
  dy: vec2<f32>,
  lightmap: vec2<f32>,
  lightmap_dx: vec2<f32>,
  lightmap_dy: vec2<f32>,
};

fn take_footprint(in: GBufferVarying) -> Footprint {
  let uv: vec2<f32> = in.uv * surfaces[in.surface].tiling;
  let scale: f32 = texture_footprint_scale();

  return Footprint(uv, uv, 0.0, 1.0, 0.0, 1.0, false, dpdx(uv) * scale, dpdy(uv) * scale, in.lightmap_uv,
    dpdx(in.lightmap_uv), dpdy(in.lightmap_uv));
}

// The engines' steep parallax (`UpdateTC` under `USE_STEEPPARALLAX`): within `STEEP_FADE.y` metres, the base coordinate
// marched along the eye's lean in tangent space over the height in the bump pair's second's alpha, `STEEP_STEPS` from
// grazing to head on, settled between its last two steps by their heights, and faded out over `STEEP_FADE`.
const STEEP_FADE: vec2<f32> = vec2<f32>(8.0, 12.0);
const STEEP_STEPS: vec2<f32> = vec2<f32>(25.0, 5.0);
// `fParallaxOffset * 1.2`: the coordinate the march reaches at most.
const STEEP_REACH: f32 = -0.0156;

// How far a marched terrain's coordinate moved, its layers weighed as they blend, out of the most its march reaches,
// which the parallax view shows; set where the enhanced parallax marched the invocation's terrain.
var<private> terrain_moved: vec2<f32> = vec2<f32>(0.0);
var<private> is_terrain_marched: bool = false;

// Where an enhanced march stands a surface the engines leave flat under its relief's top: at the middle, so a height
// that barely varies, about a half throughout, leaves the coordinate where it was.
const BUMPED_PLANE: f32 = 0.5;

// Whether the enhanced parallax marches a surface the engines leave flat (`SURFACE_IS_RELIEF_BUMPED`).
fn is_relief_bumped(flags: u32) -> bool {
  return has_surfaces_bit(SURFACES_STEEP) && has_surfaces_bit(SURFACES_ENHANCED) &&
    (flags & SURFACE_IS_RELIEF_BUMPED) != 0u;
}

fn steep_footprint(in: GBufferVarying, at: Footprint) -> Footprint {
  let surface: Surface = surfaces[in.surface];
  let position: vec3<f32> = (camera.view * vec4<f32>(in.world, 1.0)).xyz;
  let distance: f32 = -position.z;

  if (is_relief_bumped(surface.flags)) {
    return surface_relief(in, at, position, BUMPED_PLANE);
  }

  if (!has_surfaces_bit(SURFACES_STEEP) || (surface.flags & SURFACE_IS_STEEP) == 0u) {
    return at;
  }

  if (has_surfaces_bit(SURFACES_ENHANCED)) {
    return surface_relief(in, at, position, 0.0);
  }

  if (distance >= STEEP_FADE.y) {
    return at;
  }

  let eye: vec3<f32> = normalize(vec3<f32>(dot(normalize(in.tangent), -position), dot(normalize(in.binormal), -position),
    dot(normalize(in.normal), -position)));
  let steps: f32 = mix(STEEP_STEPS.x, STEEP_STEPS.y, eye.z);
  let step_size: f32 = 1.0 / steps;
  let reach: vec2<f32> = eye.xy * STEEP_REACH;
  let per_step: vec2<f32> = reach * step_size;
  let slot: u32 = surface.textures[SLOT_BUMP_COMPANION];
  var offset: vec2<f32> = at.uv;
  var height: f32 = 0.0;
  var bound: f32 = 1.0;

  for (var step: f32 = 0.0; step < steps; step += 1.0) {
    if (height < bound) {
      offset += per_step;
      height = textureSampleLevel(textures[slot], texture_sampler, offset, 0.0).a;
      bound -= step_size;
    }
  }

  // Between the step the march stopped at and the one before it, where the heights cross the bound.
  offset -= per_step;

  let previous: f32 = textureSampleGrad(textures[slot], texture_sampler, offset, at.dx, at.dy).a;
  let before: f32 = bound + step_size - previous;
  let after: f32 = bound - height;
  let amount: f32 = select(bound, (bound * before - (bound + step_size) * after) / (before - after),
    abs(before - after) > 1e-6);
  var out: Footprint = at;

  out.uv = at.uv + reach * ((1.0 - amount) * smoothstep(STEEP_FADE.y, STEEP_FADE.x, distance));

  return out;
}

// The enhanced parallax on a steep surface, in place of the engines', or on a bumped one they leave flat: its bump
// pair's second's alpha marched as a terrain's layer is, the surface drawn standing `plane` under the relief's top,
// within `parallax_mix.w` metres and `parallax_mix.z` of its coordinate deep, its hollows darkened and shadowed towards
// the sun.
fn surface_relief(in: GBufferVarying, at: Footprint, position: vec3<f32>, plane: f32) -> Footprint {
  let range: f32 = camera.parallax_mix.w;
  let distance: f32 = -position.z;

  if (distance >= range) {
    return at;
  }

  let tangent: vec3<f32> = normalize(in.tangent);
  let binormal: vec3<f32> = normalize(in.binormal);
  let normal: vec3<f32> = normalize(in.normal);
  let eye: vec3<f32> = normalize(vec3<f32>(dot(tangent, -position), dot(binormal, -position), dot(normal, -position)));
  let sun: vec3<f32> = camera.parallax_light.xyz;
  let light: vec3<f32> = vec3<f32>(dot(tangent, sun), dot(binormal, sun), dot(normal, sun));
  let fade: f32 = relief_fade(range, distance);
  let depth: f32 = camera.parallax_mix.z * fade;
  let reach: vec2<f32> = -eye.xy / (abs(eye.z) + RELIEF_GRAZING) * depth;
  let steps: f32 = mix(camera.parallax.x, RELIEF_STEPS_LEAST, saturate(abs(eye.z)));
  let slot: u32 = surfaces[in.surface].textures[SLOT_BUMP_COMPANION];
  let lod: f32 = relief_lod(slot, at.dx, at.dy);
  let hit: vec3<f32> = relief_march(slot, at.uv, lod, 1.0, reach, steps, 3u, plane);
  let height: f32 = 1.0 - hit.z;
  var out: Footprint = at;

  out.uv = hit.xy;
  out.cavity = (1.0 - height * height) * RELIEF_CAVITY * camera.parallax_mix.y * fade;
  out.sink = hit.z;
  out.clearance = relief_clearance(slot, hit.xy, lod, 1.0, reach, 3u);
  out.is_relief = true;

  if (light.z > 0.0 && camera.parallax_light.w > 0.0) {
    let to_sun: vec2<f32> = light.xy / (light.z + RELIEF_GRAZING) * depth;

    out.relief = mix(1.0, relief_shadow(slot, hit, lod, 1.0, to_sun, 3u), camera.parallax_light.w);
  }

  return out;
}

// What a surfaces debug view draws in place of a surface's albedo, the shading otherwise as it was. The parallax view:
// how far the coordinate moved, across in red and down in green about a half, out to the march's reach, over full blue,
// a terrain's by its layers as they blend; a half in both where the surface takes parallax and stood past its reach;
// every other surface a dim grey of its own brightness. The bumps view: a bumped surface's normal in its own tangent
// space as a normal map colours it, lavender where flat, a terrain's by its four details' bumps; a surface without a
// bump red by its brightness. The heights view: a terrain's details' heights weighed by its mask, and a steep surface's
// height, grey from black low to white high; every other surface a dim blue of its own brightness. The sampled mip's
// view: the mip its base texture is read at by its footprint, `mip_colour`'s, shaded by its own brightness; a surface
// without a base texture grey.
fn surfaces_shown(in: GBufferVarying, at: Footprint, shaded: GBufferOutput) -> GBufferOutput {
  let is_parallax: bool = IS_INSPECTED && has_surfaces_bit(SURFACES_PARALLAX_SHOWN);
  let is_heights: bool = IS_INSPECTED && has_surfaces_bit(SURFACES_HEIGHTS_SHOWN);
  let is_mip: bool = IS_INSPECTED && has_surfaces_bit(SURFACES_MIP_SHOWN);

  if (!is_parallax && !is_heights && !is_mip && !(IS_INSPECTED && has_surfaces_bit(SURFACES_BUMPS_SHOWN))) {
    return shaded;
  }

  var out: GBufferOutput = shaded;
  let flags: u32 = surfaces[in.surface].flags;
  let brightness: f32 = luminance(shaded.albedo.rgb);

  if (is_mip) {
    var shown: vec3<f32> = vec3<f32>(0.25 + 0.25 * brightness);

    if ((flags & SURFACE_HAS_BASE) != 0u) {
      let size: vec2<f32> = vec2<f32>(textureDimensions(textures[surfaces[in.surface].textures[SLOT_BASE]], 0));
      let texels: f32 = max(length(at.dx * size), length(at.dy * size));

      shown = mip_colour(log2(max(texels, 1e-6))) * (0.6 + 0.4 * brightness);
    }

    out.albedo = vec4<f32>(shown, shaded.albedo.a);

    return out;
  }

  if (is_parallax) {
    let is_steep: bool = (has_surfaces_bit(SURFACES_STEEP) && (flags & SURFACE_IS_STEEP) != 0u) ||
      is_relief_bumped(flags);
    // The most a march reaches: the engines' fixed one, or an enhanced surface's depth over the grazing bias.
    let reach: f32 = select(abs(STEEP_REACH), max(camera.parallax_mix.z / RELIEF_GRAZING, 1e-4),
      has_surfaces_bit(SURFACES_ENHANCED));
    let moved: vec2<f32> = select((at.uv - at.origin) / reach, terrain_moved, is_terrain_marched);

    out.albedo = vec4<f32>(select(vec3<f32>(0.08 + 0.25 * brightness),
      vec3<f32>(clamp(moved, vec2<f32>(-1.0), vec2<f32>(1.0)) * 0.5 + 0.5, 1.0), is_steep || is_terrain_marched),
      shaded.albedo.a);

    return out;
  }

  if (is_heights) {
    out.albedo = vec4<f32>(vec3<f32>(0.04, 0.06, 0.14) * (0.5 + brightness), shaded.albedo.a);

    if ((flags & SURFACE_IS_TERRAIN) != 0u) {
      let surface: Surface = surfaces[in.surface];
      let mask: vec4<f32> = sample_slot(surface.terrain.mask, at.uv, at.dx, at.dy);
      let weights: vec4<f32> = mask / max(dot(mask, vec4<f32>(1.0)), 1e-4);
      let scale: f32 = surface.detail_scale;
      var height: f32 = 0.0;

      for (var layer: u32 = 0u; layer < 4u; layer++) {
        height += sample_slot(surface.terrain.heights[layer], at.origin * scale, at.dx * scale, at.dy * scale).r *
          weights[layer];
      }

      out.albedo = vec4<f32>(vec3<f32>(height), shaded.albedo.a);
    } else if ((flags & (SURFACE_IS_STEEP | SURFACE_IS_RELIEF_BUMPED)) != 0u) {
      let height: f32 = sample_slot(surfaces[in.surface].textures[SLOT_BUMP_COMPANION], at.uv, at.dx, at.dy).a;

      out.albedo = vec4<f32>(vec3<f32>(height), shaded.albedo.a);
    }

    return out;
  }

  let is_bumped: bool = (flags & (SURFACE_HAS_BUMP | SURFACE_IS_TERRAIN)) != 0u;
  let normal: vec3<f32> = octahedral_decode(shaded.normal);
  let tangent: vec3<f32> = vec3<f32>(dot(normal, normalize(in.tangent)), dot(normal, normalize(in.binormal)),
    dot(normal, normalize(in.normal)));
  let missing: vec3<f32> = vec3<f32>(0.3 + 0.5 * brightness, 0.04, 0.04);

  out.albedo = vec4<f32>(select(missing, tangent * 0.5 + 0.5, is_bumped), shaded.albedo.a);

  return out;
}

// A slot's texture at `uv`, at the level its footprint's derivatives `dx` and `dy` pick.
fn sample_slot(slot: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec4<f32> {
  return textureSampleGrad(textures[slot], texture_sampler, uv, dx, dy);
}

// The same, trilinear: a bump or a lightmap.
fn sample_slot_trilinear(slot: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec4<f32> {
  return textureSampleGrad(textures[slot], trilinear_sampler, uv, dx, dy);
}

// The uv checker an asset viewer draws in place of a surface's textures: eight texels a side, white and dark, each
// repeat of the coordinate.
fn checker_texel(uv: vec2<f32>) -> vec4<f32> {
  let cell: vec2<f32> = floor(uv * camera.modes.y * 8.0);

  return vec4<f32>(vec3<f32>(select(64.0 / 255.0, 1.0, abs((cell.x + cell.y) % 2.0) < 0.5)), 1.0);
}

fn base_texel(in: GBufferVarying, at: Footprint) -> vec4<f32> {
  let surface: Surface = surfaces[in.surface];

  if (IS_INSPECTED && camera.modes.y > 0.0) {
    return checker_texel(in.uv);
  }

  if ((surface.flags & SURFACE_HAS_BASE) == 0u) {
    return select(vec4<f32>(1.0), vec4<f32>(camera.plain.rgb, 1.0), camera.plain.w > 0.5);
  }

  return sample_slot(surface.textures[SLOT_BASE], at.uv, at.dx, at.dy);
}

// What a terrain lays over its base (`deffer_impl_flat` with `USE_4_DETAIL` and `USE_4_BUMP`): its four details,
// their bumps in tangent space at twice the contrast, and their gloss, each weighed by its mask's channel over the
// mask's sum; and what the enhanced parallax darkens its low ground by and keeps of the sun from its hollows, none and
// all without it.
struct TerrainTexel {
  detail: vec3<f32>,
  bump: vec3<f32>,
  gloss: f32,
  cavity: f32,
  relief: f32,
  // How far its blended layers sink below the relief's top where they march, nought where they do not; and the
  // shallowest water whose reflection clears the relief (`relief_clearance`), one where nothing marches.
  sink: f32,
  clearance: f32,
};

// The enhanced parallax, a terrain's and a steep surface's: the steps a march takes looking straight down, at least;
// what is added to the eye's rise over the surface, so a grazing look reaches a bounded way along it rather than
// without end; the halvings a refined march settles by; how far the lowest ground darkens at most; and the steps a
// shadow ray takes towards the sun and how sharply one passing under the relief is shadowed. Then the band of height
// within which a terrain's layers blending by height share a texel.
const RELIEF_STEPS_LEAST: f32 = 3.0;
const RELIEF_GRAZING: f32 = 0.41;
const RELIEF_REFINES: u32 = 3u;
const RELIEF_CAVITY: f32 = 0.125;
const RELIEF_SHADOW_STEPS: u32 = 8u;
const RELIEF_SHADOW_SHARPNESS: f32 = 8.0;
const TERRAIN_BLEND_BAND: f32 = 0.25;

// Where a terrain's drawn surface stands in its relief, a share of its depth: at its heights' mean, which their solve
// sets at a half, so its bumps rise above the ground as far as its hollows sink below it and the ground meets a wall's
// foot where it is drawn, rather than all of it sunk under the top and the wall seen standing over it.
const TERRAIN_PLANE: f32 = 0.5;

// How deep a relief lies under a coordinate, from nought at its top: its height's depth, the height in `channel` of
// its texture, by its weight, so a terrain's layer weighing little lies nearly flat.
fn relief_depth(slot: u32, uv: vec2<f32>, lod: f32, weight: f32, channel: u32) -> f32 {
  return (1.0 - textureSampleLevel(textures[slot], texture_sampler, uv, lod)[channel]) * weight;
}

// The level a height is read at for a footprint, as trilinear sampling would choose it: a march reads at a distance
// what the eye sees there, never the full texture.
fn relief_lod(slot: u32, dx: vec2<f32>, dy: vec2<f32>) -> f32 {
  let size: vec2<f32> = vec2<f32>(textureDimensions(textures[slot]));
  let extent: f32 = max(dot(dx * size, dx * size), dot(dy * size, dy * size));

  return max(0.5 * log2(max(extent, 1e-8)), 0.0);
}

// Where a view ray into a relief meets it, the surface drawn standing at depth `plane` under its top: marched from
// where the ray enters the top, `plane` before `uv` along `reach`, the coordinate it moves by down to the deepest, in
// `steps`; settled between its last two steps as straight lines, then by halving where refined. The coordinate, and
// the depth.
fn relief_march(
  slot: u32,
  uv: vec2<f32>,
  lod: f32,
  weight: f32,
  reach: vec2<f32>,
  steps: f32,
  channel: u32,
  plane: f32,
) -> vec3<f32> {
  let count: u32 = u32(ceil(steps));
  let step: f32 = 1.0 / f32(count);
  let entry: vec2<f32> = uv - reach * plane;
  var depth: f32 = 0.0;
  var surface: f32 = relief_depth(slot, entry, lod, weight, channel);
  var before_depth: f32 = 0.0;
  var before_surface: f32 = surface;

  for (var index: u32 = 0u; index < count; index++) {
    if (surface <= depth) {
      break;
    }

    before_depth = depth;
    before_surface = surface;
    depth += step;
    surface = relief_depth(slot, entry + reach * depth, lod, weight, channel);
  }

  let above: f32 = before_surface - before_depth;
  let below: f32 = surface - depth;
  var hit: f32 = select(depth, mix(before_depth, depth, above / (above - below)), above - below > 1e-6);

  if (camera.parallax.w > 0.5) {
    var low: f32 = before_depth;
    var high: f32 = depth;

    for (var index: u32 = 0u; index < RELIEF_REFINES; index++) {
      let middle: f32 = (low + high) * 0.5;

      if (relief_depth(slot, entry + reach * middle, lod, weight, channel) <= middle) {
        high = middle;
      } else {
        low = middle;
      }
    }

    hit = (low + high) * 0.5;
  }

  return vec3<f32>(entry + reach * hit, hit);
}

// How shallow water over a relief's point at `uv` must stand for its reflection, leaving the water's level towards
// `to`, the coordinate it moves by a unit of depth risen, to pass over the relief rather than meet a hollow's far side:
// the least, over a ray's steps rising a unit of depth, of the relief's depth there and the rise. The reflected ray
// on a flat relief leaves along the view's own march, mirrored up, so `to` is the march's `reach`. Water above it
// mirrors the sky; below it, the hollow's side.
fn relief_clearance(slot: u32, uv: vec2<f32>, lod: f32, weight: f32, to: vec2<f32>, channel: u32) -> f32 {
  var clearance: f32 = 1.0;

  for (var index: u32 = 1u; index <= RELIEF_SHADOW_STEPS; index++) {
    let rise: f32 = f32(index) / f32(RELIEF_SHADOW_STEPS);

    clearance = min(clearance, relief_depth(slot, uv + to * rise, lod, weight, channel) + rise);
  }

  return clearance;
}

// How much of the sun reaches a relief's point at `hit`, its coordinate and depth: a ray rising from it towards the sun
// along `to_sun`, the coordinate it moves by a unit of depth, shadowed softly where it passes under the relief and the
// more the nearer the point.
fn relief_shadow(slot: u32, hit: vec3<f32>, lod: f32, weight: f32, to_sun: vec2<f32>, channel: u32) -> f32 {
  var shadow: f32 = 0.0;

  for (var index: u32 = 1u; index <= RELIEF_SHADOW_STEPS; index++) {
    let along: f32 = f32(index) / f32(RELIEF_SHADOW_STEPS);
    let ray: f32 = hit.z * (1.0 - along);
    let surface: f32 = relief_depth(slot, hit.xy + to_sun * (hit.z * along), lod, weight, channel);

    shadow = max(shadow, (ray - surface) * RELIEF_SHADOW_SHARPNESS * (1.0 - along));
  }

  return 1.0 - saturate(shadow);
}

fn terrain_texel(
  in: GBufferVarying,
  surface: Surface,
  at: Footprint,
  uv: vec2<f32>,
  dx: vec2<f32>,
  dy: vec2<f32>,
) -> TerrainTexel {
  let mask: vec4<f32> = sample_slot(surface.terrain.mask, at.uv, at.dx, at.dy);
  let weights: vec4<f32> = mask / max(dot(mask, vec4<f32>(1.0)), 1e-4);
  var out: TerrainTexel = TerrainTexel(vec3<f32>(0.0), vec3<f32>(0.0), 0.0, 0.0, 1.0, 0.0, 1.0);
  let position: vec3<f32> = (camera.view * vec4<f32>(in.world, 1.0)).xyz;
  let range: f32 = camera.parallax.y;
  let is_marched: bool = has_surfaces_bit(SURFACES_ENHANCED) && -position.z < range;
  var reach: vec2<f32> = vec2<f32>(0.0);
  var to_sun: vec2<f32> = vec2<f32>(0.0);
  var steps: f32 = 0.0;
  var is_lit: bool = false;
  var fade: f32 = 0.0;

  // The eye and the sun in the surface's tangent space, the depth faded out over the last of the range.
  if (is_marched) {
    let tangent: vec3<f32> = normalize(in.tangent);
    let binormal: vec3<f32> = normalize(in.binormal);
    let normal: vec3<f32> = normalize(in.normal);
    let eye: vec3<f32> = normalize(vec3<f32>(dot(tangent, -position), dot(binormal, -position), dot(normal, -position)));
    let sun: vec3<f32> = camera.parallax_light.xyz;
    let light: vec3<f32> = vec3<f32>(dot(tangent, sun), dot(binormal, sun), dot(normal, sun));
    fade = relief_fade(range, -position.z);

    let depth: f32 = camera.parallax.z * fade;

    reach = -eye.xy / (abs(eye.z) + RELIEF_GRAZING) * depth;
    steps = mix(camera.parallax.x, RELIEF_STEPS_LEAST, saturate(abs(eye.z)));
    is_lit = light.z > 0.0 && camera.parallax_light.w > 0.0;
    to_sun = light.xy / (light.z + RELIEF_GRAZING) * depth;
  }

  // Each layer's coordinate where its march met it and its height there; the plane where none marches. The dominant
  // layer's coordinate, depth and lod are kept apart as the march finds them, so nothing reads the array by a layer
  // known only per pixel.
  var coords: array<vec2<f32>, 4> = array<vec2<f32>, 4>(uv, uv, uv, uv);
  var heights: vec4<f32> = vec4<f32>(0.5);
  var dominant: u32 = 0u;
  var dominant_coord: vec2<f32> = uv;
  var dominant_depth: f32 = 0.0;
  var dominant_lod: f32 = 0.0;

  for (var layer: u32 = 0u; layer < 4u; layer++) {
    let weight: f32 = weights[layer];

    if (weight <= TERRAIN_LAYER_LEAST) {
      continue;
    }

    if (weight > weights[dominant]) {
      dominant = layer;
    }

    if (is_marched) {
      let slot: u32 = surface.terrain.heights[layer];
      let lod: f32 = relief_lod(slot, dx, dy);
      let hit: vec3<f32> = relief_march(slot, uv, lod, weight, reach, steps, 0u, TERRAIN_PLANE * weight);

      coords[layer] = hit.xy;
      heights[layer] = textureSampleLevel(textures[slot], texture_sampler, hit.xy, lod).r;

      if (layer == dominant) {
        dominant_coord = hit.xy;
        dominant_depth = hit.z;
        dominant_lod = lod;
      }
    }
  }

  // Layers blended by height: within a band of the one standing highest, weight and height together, each shares by how
  // far it stands into the band; the mask alone at none.
  let present: vec4<f32> = step(vec4<f32>(TERRAIN_LAYER_LEAST + 1e-6), weights);
  var blended: vec4<f32> = weights;

  if (is_marched && camera.parallax_mix.x * fade > 0.0) {
    let keys: vec4<f32> = (weights + heights * camera.parallax_mix.x) * present - (1.0 - present);
    let top: f32 = max(max(keys.x, keys.y), max(keys.z, keys.w));
    let shares: vec4<f32> = max(keys - (top - TERRAIN_BLEND_BAND), vec4<f32>(0.0)) * present;

    blended = mix(weights, shares / max(dot(shares, vec4<f32>(1.0)), 1e-4), camera.parallax_mix.x * fade);
  }

  for (var layer: u32 = 0u; layer < 4u; layer++) {
    let weight: f32 = blended[layer];

    if (weight <= TERRAIN_LAYER_LEAST || present[layer] < 0.5) {
      continue;
    }

    // `.wzyx`: the normal in the last three channels, the gloss in the first.
    let bump: vec4<f32> = sample_slot(surface.terrain.bumps[layer], coords[layer], dx, dy).wzyx;

    out.detail += sample_slot(surface.terrain.details[layer], coords[layer], dx, dy).rgb * weight;
    out.bump += (bump.xyz - 0.5) * weight;
    out.gloss += bump.w * weight;
    out.cavity += (1.0 - heights[layer] * heights[layer]) * weight;
    out.sink += (1.0 - heights[layer]) * weight;
    terrain_moved += (coords[layer] - uv) * weight;
  }

  if (is_marched) {
    terrain_moved /= max(camera.parallax.z / RELIEF_GRAZING, 1e-4);
    is_terrain_marched = true;
  }

  out.bump.z *= 0.5;
  out.cavity = select(0.0, out.cavity * RELIEF_CAVITY * camera.parallax_mix.y * fade, is_marched);
  out.sink = select(0.0, out.sink, is_marched);

  if (is_marched) {
    out.clearance = relief_clearance(surface.terrain.heights[dominant], dominant_coord, dominant_lod,
      weights[dominant], reach, 0u);
  }

  if (is_lit) {
    let slot: u32 = surface.terrain.heights[dominant];
    let shadow: f32 = relief_shadow(slot, vec3<f32>(dominant_coord, dominant_depth), dominant_lod,
      weights[dominant], to_sun, 0u);

    out.relief = mix(1.0, shadow, camera.parallax_light.w);
  }

  return out;
}

fn shade(in: GBufferVarying, base: vec4<f32>, at: Footprint) -> GBufferOutput {
  let surface: Surface = surfaces[in.surface];
  let scale: f32 = surface.detail_scale;
  let detail_uv: vec2<f32> = at.origin * scale;
  let detail_dx: vec2<f32> = at.dx * scale;
  let detail_dy: vec2<f32> = at.dy * scale;
  let is_textured: f32 = camera.switches.x;
  let is_bumped: f32 = camera.switches.y;
  var diffuse: vec3<f32> = base.rgb;
  var normal: vec3<f32> = normalize(in.normal);
  var gloss: f32 = DEFAULT_GLOSS;
  var detail: vec4<f32> = vec4<f32>(0.5);

  let is_terrain: bool = (surface.flags & SURFACE_IS_TERRAIN) != 0u;
  var relief: f32 = at.relief;
  var sink: f32 = at.sink;
  var clearance: f32 = at.clearance;

  diffuse *= 1.0 - at.cavity;

  if (is_terrain) {
    let terrain: TerrainTexel = terrain_texel(in, surface, at, detail_uv, detail_dx, detail_dy);
    let weight: f32 = is_bumped * is_textured;

    diffuse *= terrain.detail * 2.0 * (1.0 - terrain.cavity);
    relief = terrain.relief;
    sink = terrain.sink;
    clearance = terrain.clearance;
    normal = normalize(mix(normal, normalize(in.tangent * terrain.bump.x + in.binormal * terrain.bump.y +
      in.normal * terrain.bump.z), weight));
    gloss = mix(DEFAULT_GLOSS, terrain.gloss, weight);
  } else if ((surface.flags & SURFACE_HAS_DETAIL) != 0u) {
    detail = sample_slot(surface.textures[SLOT_DETAIL], detail_uv, detail_dx, detail_dy);
    diffuse *= detail.rgb * 2.0;
  }

  // A leaf lit as foliage reads its bump's tilt alone, over a normal standing up.
  var leaf_tilt: vec2<f32> = vec2<f32>(0.0);

  if (!is_terrain && (surface.flags & SURFACE_HAS_BUMP) != 0u) {
    let bump: vec4<f32> = sample_slot_trilinear(surface.textures[SLOT_BUMP], at.uv, at.dx, at.dy);
    var tangent_normal: vec3<f32> = bump.wzy +
      sample_slot_trilinear(surface.textures[SLOT_BUMP_COMPANION], at.uv, at.dx, at.dy).xyz - 1.0;

    leaf_tilt = tangent_normal.xy * is_bumped * is_textured;
    var bumped_gloss: f32 = bump.x * bump.x;

    if ((surface.flags & SURFACE_HAS_DETAIL_BUMP) != 0u) {
      let detail_bump: vec4<f32> = sample_slot_trilinear(surface.textures[SLOT_DETAIL_BUMP], detail_uv, detail_dx,
        detail_dy);
      let companion: vec4<f32> = sample_slot_trilinear(surface.textures[SLOT_DETAIL_BUMP_COMPANION], detail_uv,
        detail_dx, detail_dy);

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

  // Foliage: a leaf's normal stands up, tilted by its bump, harder when wet, so the sun lights a crown from above and
  // through it from behind.
  let is_flora: bool = in.flora.x > 0.5;

  if (is_flora) {
    let tilt: vec2<f32> = leaf_tilt * max(5.0 * in.flora.y, 3.0);

    normal = normalize((camera.view * vec4<f32>(tilt.x, 1.0, tilt.y, 0.0)).xyz);
  }

  // A terrain is lit by its base's alpha, `deffer_impl_flat`'s `Ne.w = D.w`.
  var hemi: f32 = select(in.hemi, base.a, is_terrain);
  var sun: f32 = 1.0;

  if ((surface.flags & SURFACE_HAS_HEMI) != 0u) {
    let lightmap: vec4<f32> = sample_slot_trilinear(surface.textures[SLOT_HEMI], at.lightmap, at.lightmap_dx,
      at.lightmap_dy);

    hemi = lightmap.a;
    sun = lightmap.g;
  }

  // A terrain's and a marched steep surface's sun term is what the enhanced parallax keeps of the sun, which the sun
  // pass takes, the baked sun unread as the deferred engines leave it.
  if (is_terrain || at.is_relief) {
    sun = relief;
  }

  var out: GBufferOutput;

  out.albedo = vec4<f32>(mix(untextured_color(surface.color), diffuse, is_textured), gloss);
  out.normal = octahedral_encode(normal);
  // Its alpha holds the marks: what is selected, which the present pass outlines, what is self-lit, foliage and
  // terrain.
  out.material = vec4<f32>(hemi, sun, surface.slice,
    encode_marks(is_selected(in.entry), (surface.flags & SURFACE_IS_EMISSIVE) != 0u, is_flora, is_terrain, at.is_relief,
      in.flora.z > 0.5, is_moved(in.moved)));
  out.motion = camera_motion(in.world, in.world + in.moved);
  // The clearance stored from one down, so a target cleared to none reads as clear.
  out.relief = vec2<f32>(sink, 1.0 - clearance);

  return out;
}

@fragment
fn fs_opaque(in: GBufferVarying) -> GBufferOutput {
  let at: Footprint = steep_footprint(in, take_footprint(in));

  if (is_off_wire(in)) {
    discard;
  }

  return wired(surfaces_shown(in, at, shade(in, base_texel(in, at), at)));
}

// Whether a cut-out texel is cut, as `common/cut_out` cuts one.
fn is_cut(in: GBufferVarying, base: vec4<f32>, at: Footprint) -> bool {
  let surface: Surface = surfaces[in.surface];

  return is_alpha_cut(base.a, vec2<f32>(textureDimensions(textures[surface.textures[SLOT_BASE]])), at.dx, at.dy,
    surface.alpha_reference);
}

// A cut-out surface's depth alone, cut as `fs_cut_out` cuts it, ahead of the G-buffer's draw: the cut-out draw then
// shades only where its depth is equal, once a pixel however many cut-out layers overlap there.
@fragment
fn fs_depth_cut_out(in: GBufferVarying) {
  let at: Footprint = steep_footprint(in, take_footprint(in));
  let base: vec4<f32> = base_texel(in, at);

  if (is_off_wire(in) || ((!IS_INSPECTED || (camera.modes.x < 0.5 && camera.modes.z < 0.5)) && is_cut(in, base, at))) {
    discard;
  }
}

@fragment
fn fs_cut_out(in: GBufferVarying) -> GBufferOutput {
  let at: Footprint = steep_footprint(in, take_footprint(in));
  let base: vec4<f32> = base_texel(in, at);
  let is_off: bool = is_off_wire(in);

  // A wireframe draws a cut-out surface's every edge, whatever its alpha; so does a view drawing every surface solid.
  if (is_off || ((!IS_INSPECTED || (camera.modes.x < 0.5 && camera.modes.z < 0.5)) && is_cut(in, base, at))) {
    discard;
  }

  return wired(surfaces_shown(in, at, shade(in, base, at)));
}

// A pick's texel: what was drawn, by its cluster and place, and its depth's bits.
// What the overdraw's debug view adds into the light target for each surface drawn over a pixel.
const OVERDRAW_LAYER: vec4<f32> = vec4<f32>(1.0, 0.0, 0.0, 0.0);

@fragment
fn fs_overdraw(in: GBufferVarying) -> @location(0) vec4<f32> {
  return OVERDRAW_LAYER;
}

// A cut-out counts where its alpha keeps it, as it is drawn.
@fragment
fn fs_overdraw_cut_out(in: GBufferVarying) -> @location(0) vec4<f32> {
  let at: Footprint = steep_footprint(in, take_footprint(in));

  if (is_cut(in, base_texel(in, at), at)) {
    discard;
  }

  return OVERDRAW_LAYER;
}

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

// The share of a blended composited texel past which a click meets it: any that shows at all, glass however clear (a
// gauge's is a fiftieth), and not the empty texels about a blended fence's wires.
const PICK_SHOWN: f32 = 0.5 / 255.0;

// A composited surface in a pick, the nearest thing a click meets as glass is: one blended over what is behind it,
// where its texel shows and its own reference passes. Light added or multiplied in, a glow or a halo, stands for
// nothing a click could meet, and is passed by.
@fragment
fn fs_pick_composited(in: GBufferVarying) -> @location(0) vec4<u32> {
  let at: Footprint = take_footprint(in);
  let surface: Surface = surfaces[in.surface];
  let base: vec4<f32> = base_texel(in, at);

  if ((surface.flags & (SURFACE_IS_ADDED | SURFACE_IS_MULTIPLIED)) != 0u ||
    base.a <= max(surface.alpha_reference, PICK_SHOWN)) {
    discard;
  }

  return pick_texel(in);
}

// A cut-out caster in a shadow's map: depth alone, cut as the G-buffer cuts it, so light falls between its leaves. Its
// base is read at the mip its footprint's longer side picks, without the anisotropic taps, which a depth's cut does not
// need; the cut raises the alpha by the mip as it does for any read.
@fragment
fn fs_shadow_cut_out(in: GBufferVarying) {
  let at: Footprint = take_footprint(in);
  let surface: Surface = surfaces[in.surface];
  var base: vec4<f32> = vec4<f32>(1.0);

  if ((surface.flags & SURFACE_HAS_BASE) != 0u) {
    let slot: u32 = surface.textures[SLOT_BASE];
    let size: vec2<f32> = vec2<f32>(textureDimensions(textures[slot]));
    let extent: f32 = max(dot(at.dx * size, at.dx * size), dot(at.dy * size, at.dy * size));

    base = textureSampleLevel(textures[slot], texture_sampler, at.uv, 0.5 * log2(max(extent, 1e-8)));
  }

  if (is_cut(in, base, at)) {
    discard;
  }
}

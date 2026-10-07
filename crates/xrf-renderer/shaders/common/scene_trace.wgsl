enable wgpu_binding_array;

#import "static/records"
#import "static/vertex"
#import "generated/common/scene_trace"

// Rays traced by hardware ray query against the level's static geometry (`StaticTrace`, bound by `TraceParameters`
// at group 3): each top-level instance names an entry of the instance table, its first record and its place; each
// geometry a record naming its layout's arena, where its triangles' indices and vertices start, and its surface; and
// the texture cache's bindless textures at group 4, which a hit is shaded from and a cut-out geometry's holes are
// tested in. A device without ray queries composes `common/scene_trace_none` in its place, whose trace meets nothing.

// What a traced ray met: its distance along the ray, below none for nothing; its record, triangle and barycentrics;
// the place its instance stands at; the point it met, in renderer space; and what turns its geometry's normals into
// renderer space, the transpose of its instance's inverse.
struct SceneHit {
  distance: f32,
  record: u32,
  primitive: u32,
  barycentrics: vec2<f32>,
  place: u32,
  position: vec3<f32>,
  to_world: mat3x3<f32>,
}

// A hit's surface as a reflection shades it, in renderer space: where it is, its normal, its albedo, its baked
// hemisphere and baked sun share, and its surface's slice of the material table.
struct SceneSurface {
  position: vec3<f32>,
  normal: vec3<f32>,
  albedo: vec3<f32>,
  hemi: f32,
  sun: f32,
  slice: f32,
}

// How wide a ray's cone is where it starts, in metres, and how much wider each metre along it: what the level of
// detail a hit's textures are read at follows.
struct SceneCone {
  width: f32,
  spread: f32,
}

// One triangle's corners as its layout's arena holds them, in its geometry's own space.
struct SceneTriangle {
  positions: array<vec3<f32>, 3>,
  uvs: array<vec2<f32>, 3>,
  lightmaps: array<vec2<f32>, 3>,
  normals: array<vec4<f32>, 3>,
}

@group(4) @binding(0) var trace_textures: binding_array<texture_2d<f32>>;
@group(4) @binding(1) var trace_sampler: sampler;

const IS_SCENE_TRACED: bool = true;
// A record's flags, as `StaticTraceRecord` sets them: a cut-out geometry's, and the arena a tree's or a spawned model's
// vertices are in, the baked one where neither.
const TRACE_RECORD_CUT_OUT: u32 = 1u;
const TRACE_RECORD_TREE: u32 = 2u;
const TRACE_RECORD_MODEL: u32 = 4u;

// The nearest of the level's static geometry along a ray in renderer space between `near` and `far` metres, a
// cut-out section's holes letting it through.
fn trace_scene_ray(origin: vec3<f32>, direction: vec3<f32>, near: f32, far: f32, cone: SceneCone) -> SceneHit {
  return traverse_solid(RayDesc(RAY_FLAG_NONE, 0xffu, near, far, origin, direction), cone);
}

// Whether anything solid of the level's static geometry stands along a ray within `far` metres.
fn trace_scene_blocked(origin: vec3<f32>, direction: vec3<f32>, far: f32, cone: SceneCone) -> bool {
  return traverse_solid(RayDesc(RAY_FLAG_TERMINATE_ON_FIRST_HIT, 0xffu, 0.0, far, origin, direction), cone).distance >=
    0.0;
}

// Traces a ray, taking each cut-out triangle it passes where its texel there is solid, an opaque section taken by the
// hardware without asking; what it met, its distance below none for nothing.
fn traverse_solid(ray: RayDesc, cone: SceneCone) -> SceneHit {
  var query: ray_query;

  rayQueryInitialize(&query, trace_scene, ray);

  while (rayQueryProceed(&query)) {
    let candidate: RayIntersection = rayQueryGetCandidateIntersection(&query);
    let record: u32 = trace_instances[candidate.instance_custom_data].first_record + candidate.geometry_index;

    if (candidate.kind == RAY_QUERY_INTERSECTION_TRIANGLE &&
      is_scene_solid(record, candidate.primitive_index, candidate.barycentrics, cone.width + cone.spread * candidate.t)) {
      rayQueryConfirmIntersection(&query);
    }
  }

  let hit: RayIntersection = rayQueryGetCommittedIntersection(&query);

  if (hit.kind == RAY_QUERY_INTERSECTION_NONE) {
    return SceneHit(-1.0, 0u, 0u, vec2<f32>(0.0), 0u, vec3<f32>(0.0), mat3x3<f32>());
  }

  let instance: TraceInstance = trace_instances[hit.instance_custom_data];
  let inverse: mat4x3<f32> = hit.world_to_object;

  return SceneHit(hit.t, instance.first_record + hit.geometry_index, hit.primitive_index, hit.barycentrics,
    instance.place, ray.origin + ray.dir * hit.t, transpose(mat3x3<f32>(inverse[0], inverse[1], inverse[2])));
}

// Whether a cut-out section is solid at a point of a triangle: its base texture's alpha at the level of detail the
// cone's `width` there reads, raised as minification thins it as the G-buffer's cut raises it (`is_alpha_cut`), over
// its surface's reference.
fn is_scene_solid(index: u32, primitive: u32, barycentrics: vec2<f32>, width: f32) -> bool {
  let record: TraceRecord = trace_records[index];

  if ((record.flags & TRACE_RECORD_CUT_OUT) == 0u) {
    return true;
  }

  let surface: Surface = trace_surfaces[record.surface];
  let triangle: SceneTriangle = scene_triangle(record, primitive);
  let weights: vec3<f32> = barycentric_weights(barycentrics);
  let slot: u32 = surface.textures[SLOT_BASE];
  let lod: f32 = base_lod(surface, triangle, slot, width);
  let alpha: f32 = textureSampleLevel(trace_textures[slot], trace_sampler, base_uv(surface, triangle, weights), lod).a;

  return alpha * (1.0 + 0.25 * lod) > surface.alpha_reference;
}

// A hit's surface: its corners read from its layout's arena, interpolated, its normal turned into renderer space and
// its hemisphere scaled as its place's; its base texture read at the level of detail the ray's cone, `width` wide at
// the hit, covers.
fn scene_surface(hit: SceneHit, width: f32) -> SceneSurface {
  let record: TraceRecord = trace_records[hit.record];
  let surface: Surface = trace_surfaces[record.surface];
  let triangle: SceneTriangle = scene_triangle(record, hit.primitive);
  let weights: vec3<f32> = barycentric_weights(hit.barycentrics);
  let slot: u32 = surface.textures[SLOT_BASE];
  let base: vec4<f32> = textureSampleLevel(trace_textures[slot], trace_sampler, base_uv(surface, triangle, weights),
    base_lod(surface, triangle, slot, width));
  let normal: vec4<f32> = triangle.normals[0] * weights.x + triangle.normals[1] * weights.y +
    triangle.normals[2] * weights.z;
  let place: vec4<f32> = trace_places[hit.place].info;
  var hemi: f32 = normal.w * place.x + place.y;
  var sun: f32 = 1.0;

  // A terrain is lit by its base's alpha, as the G-buffer lights it; a lightmapped surface by its lightmap.
  if ((surface.flags & SURFACE_IS_TERRAIN) != 0u) {
    hemi = base.a;
  }

  if ((surface.flags & SURFACE_HAS_HEMI) != 0u) {
    let lightmap_uv: vec2<f32> = triangle.lightmaps[0] * weights.x + triangle.lightmaps[1] * weights.y +
      triangle.lightmaps[2] * weights.z;
    let lightmap: vec4<f32> = textureSampleLevel(trace_textures[surface.textures[SLOT_HEMI]], trace_sampler,
      lightmap_uv, 0.0);

    hemi = lightmap.a;
    sun = lightmap.g;
  }

  return SceneSurface(hit.position, normalize(hit.to_world * normal.xyz), base.rgb, hemi, sun, surface.slice);
}

// A triangle's corners, read from its layout's arena through its record: a sector's or an instanced mesh's base
// coordinate as two shorts with their fraction, and its lightmap's; a tree's as two shorts over 2048, and none; a
// spawned model's as two floats, and none.
fn scene_triangle(record: TraceRecord, primitive: u32) -> SceneTriangle {
  var triangle: SceneTriangle;

  for (var corner: u32 = 0u; corner < 3u; corner++) {
    let vertex: u32 = trace_indices[record.first_index + primitive * 3u + corner] + record.vertex_start;
    let at: u32 = vertex * VERTEX_WORDS;
    let binormal: vec4<f32> = unpack4x8unorm(trace_word(record.flags, at + VERTEX_BINORMAL));
    let normal: vec4<f32> = unpack4x8unorm(trace_word(record.flags, at + VERTEX_NORMAL));
    let tangent: vec4<f32> = unpack4x8unorm(trace_word(record.flags, at + VERTEX_TANGENT));
    let uv: u32 = trace_word(record.flags, at + VERTEX_UV);
    let extra: u32 = trace_word(record.flags, at + VERTEX_EXTRA);

    triangle.positions[corner] = vertex_position(trace_word(record.flags, at + VERTEX_POSITION),
      trace_word(record.flags, at + VERTEX_POSITION + 1u), trace_word(record.flags, at + VERTEX_POSITION + 2u));
    triangle.normals[corner] = vec4<f32>(unpack_direction(normal), normal.w);

    if ((record.flags & TRACE_RECORD_TREE) != 0u) {
      triangle.uvs[corner] = unpack_shorts(uv) / 2048.0;
    } else if ((record.flags & TRACE_RECORD_MODEL) != 0u) {
      triangle.uvs[corner] = vec2<f32>(bitcast<f32>(uv), bitcast<f32>(extra));
    } else {
      triangle.uvs[corner] = baked_uv(uv, tangent, binormal);
      triangle.lightmaps[corner] = baked_lightmap_uv(extra);
    }
  }

  return triangle;
}

// A word of a record's layout's arena.
fn trace_word(flags: u32, at: u32) -> u32 {
  if ((flags & TRACE_RECORD_TREE) != 0u) {
    return trace_tree_words[at];
  }

  if ((flags & TRACE_RECORD_MODEL) != 0u) {
    return trace_model_words[at];
  }

  return trace_baked_words[at];
}

fn barycentric_weights(barycentrics: vec2<f32>) -> vec3<f32> {
  return vec3<f32>(1.0 - barycentrics.x - barycentrics.y, barycentrics);
}

// A point's base coordinate, tiled as its surface draws it.
fn base_uv(surface: Surface, triangle: SceneTriangle, weights: vec3<f32>) -> vec2<f32> {
  return (triangle.uvs[0] * weights.x + triangle.uvs[1] * weights.y + triangle.uvs[2] * weights.z) * surface.tiling;
}

// The level of detail a base texture is read at under a cone `width` wide: texels a metre of the triangle spans, from
// its area in each.
fn base_lod(surface: Surface, triangle: SceneTriangle, slot: u32, width: f32) -> f32 {
  let size: vec2<f32> = vec2<f32>(textureDimensions(trace_textures[slot]));
  let world_area: f32 = length(cross(triangle.positions[1] - triangle.positions[0],
    triangle.positions[2] - triangle.positions[0]));
  let uv_area: f32 = abs(trace_cross(triangle.uvs[1] - triangle.uvs[0], triangle.uvs[2] - triangle.uvs[0])) *
    surface.tiling * surface.tiling;
  let density: f32 = sqrt(uv_area / max(world_area, 1e-8)) * max(size.x, size.y);

  return log2(max(width * density, 1.0));
}

fn trace_cross(a: vec2<f32>, b: vec2<f32>) -> f32 {
  return a.x * b.y - a.y * b.x;
}

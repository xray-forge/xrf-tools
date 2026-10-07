enable wgpu_binding_array;

#import "static/records"
#import "static/vertex"
#import "generated/common/scene_trace"

// Rays traced by hardware ray query against the level's static geometry (`StaticTrace`, bound by `TraceParameters`
// at group 3): a sector's deferred baked sections, each geometry a record naming where its triangles' indices and
// vertices start and its surface; and the texture cache's bindless textures at group 4, which a hit is shaded from.
// A device without ray queries composes `common/scene_trace_none` in its place, whose trace meets nothing.

// What a traced ray met: its distance along the ray, below none for nothing; its record, triangle and barycentrics.
struct SceneHit {
  distance: f32,
  record: u32,
  primitive: u32,
  barycentrics: vec2<f32>,
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

@group(4) @binding(0) var trace_textures: binding_array<texture_2d<f32>>;
@group(4) @binding(1) var trace_sampler: sampler;

const IS_SCENE_TRACED: bool = true;

// The nearest of the level's static geometry along a ray in renderer space between `near` and `far` metres; a
// cut-out section stands whole.
fn trace_scene_ray(origin: vec3<f32>, direction: vec3<f32>, near: f32, far: f32) -> SceneHit {
  var query: ray_query;

  rayQueryInitialize(&query, trace_scene, RayDesc(RAY_FLAG_FORCE_OPAQUE, 0xffu, near, far, origin, direction));
  rayQueryProceed(&query);

  let hit: RayIntersection = rayQueryGetCommittedIntersection(&query);

  if (hit.kind == RAY_QUERY_INTERSECTION_NONE) {
    return SceneHit(-1.0, 0u, 0u, vec2<f32>(0.0));
  }

  return SceneHit(hit.t, hit.instance_custom_data + hit.geometry_index, hit.primitive_index, hit.barycentrics);
}

// Whether anything of the level's static geometry stands along a ray within `far` metres.
fn trace_scene_blocked(origin: vec3<f32>, direction: vec3<f32>, far: f32) -> bool {
  var query: ray_query;

  rayQueryInitialize(&query, trace_scene,
    RayDesc(RAY_FLAG_FORCE_OPAQUE | RAY_FLAG_TERMINATE_ON_FIRST_HIT, 0xffu, 0.0, far, origin, direction));
  rayQueryProceed(&query);

  return rayQueryGetCommittedIntersection(&query).kind != RAY_QUERY_INTERSECTION_NONE;
}

// A hit's surface: its corners read from the baked arena, interpolated; its base texture sampled at a level of detail
// widening with `spread`, the width the ray's cone has grown to at the hit.
fn scene_surface(hit: SceneHit, spread: f32) -> SceneSurface {
  let record: TraceRecord = trace_records[hit.record];
  let surface: Surface = trace_surfaces[record.surface];
  let weights: vec3<f32> = vec3<f32>(1.0 - hit.barycentrics.x - hit.barycentrics.y, hit.barycentrics);
  var positions: array<vec3<f32>, 3>;
  var uvs: array<vec2<f32>, 3>;
  var lightmaps: array<vec2<f32>, 3>;
  var normal: vec3<f32> = vec3<f32>(0.0);
  var hemi: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 3u; corner++) {
    let vertex: u32 = trace_indices[record.first_index + hit.primitive * 3u + corner] + record.vertex_start;
    let at: u32 = vertex * VERTEX_WORDS;
    let binormal: vec4<f32> = unpack4x8unorm(trace_words[at + VERTEX_BINORMAL]);
    let packed_normal: vec4<f32> = unpack4x8unorm(trace_words[at + VERTEX_NORMAL]);
    let tangent: vec4<f32> = unpack4x8unorm(trace_words[at + VERTEX_TANGENT]);

    positions[corner] = vertex_position(trace_words[at + VERTEX_POSITION], trace_words[at + VERTEX_POSITION + 1u],
      trace_words[at + VERTEX_POSITION + 2u]);
    uvs[corner] = baked_uv(trace_words[at + VERTEX_UV], tangent, binormal);
    lightmaps[corner] = baked_lightmap_uv(trace_words[at + VERTEX_EXTRA]);
    normal += unpack_direction(packed_normal) * weights[corner];
    hemi += packed_normal.w * weights[corner];
  }

  let uv: vec2<f32> = (uvs[0] * weights.x + uvs[1] * weights.y + uvs[2] * weights.z) * surface.tiling;
  let lightmap_uv: vec2<f32> = lightmaps[0] * weights.x + lightmaps[1] * weights.y + lightmaps[2] * weights.z;
  // Texels a metre of the triangle spans of its base texture, from its area in each, which the cone's width reads
  // a level of detail by.
  let base_slot: u32 = surface.textures[SLOT_BASE];
  let size: vec2<f32> = vec2<f32>(textureDimensions(trace_textures[base_slot]));
  let world_area: f32 = length(cross(positions[1] - positions[0], positions[2] - positions[0]));
  let uv_area: f32 = abs(trace_cross(uvs[1] - uvs[0], uvs[2] - uvs[0])) * surface.tiling * surface.tiling;
  let density: f32 = sqrt(uv_area / max(world_area, 1e-8)) * max(size.x, size.y);
  let lod: f32 = log2(max(spread * density, 1.0));
  let base: vec4<f32> = textureSampleLevel(trace_textures[base_slot], trace_sampler, uv, lod);
  var sun: f32 = 1.0;
  let is_terrain: bool = (surface.flags & SURFACE_IS_TERRAIN) != 0u;

  // A terrain is lit by its base's alpha, as the G-buffer lights it; a lightmapped surface by its lightmap.
  if (is_terrain) {
    hemi = base.a;
  }

  if ((surface.flags & SURFACE_HAS_HEMI) != 0u) {
    let lightmap: vec4<f32> = textureSampleLevel(trace_textures[surface.textures[SLOT_HEMI]], trace_sampler,
      lightmap_uv, 0.0);

    hemi = lightmap.a;
    sun = lightmap.g;
  }

  let position: vec3<f32> = positions[0] * weights.x + positions[1] * weights.y + positions[2] * weights.z;

  return SceneSurface(position, normalize(normal), base.rgb, hemi, sun, surface.slice);
}

fn trace_cross(a: vec2<f32>, b: vec2<f32>) -> f32 {
  return a.x * b.y - a.y * b.x;
}

enable wgpu_binding_array;

#import "common/camera"
#import "common/hmodel"
#import "common/local_lights"
#import "grass/records"
#import "static/records"
#import "static/vertex"
#import "generated/common/scene_trace"

// Rays traced by hardware ray query against the level's static geometry (`StaticTrace`, bound by `TraceParameters`
// at group 3): each top-level instance names an entry of the instance table, its first record and its place; each
// geometry a record naming its layout's arena, where its triangles' indices and vertices start, and its surface; and
// the texture cache's bindless textures at group 4, which a hit is shaded from and a cut-out geometry's holes are
// tested in; and the view's local lights with the shadow atlas, which light a hit as the lights pass lights a pixel. A device without ray queries composes `common/scene_trace_none` in its place, whose trace meets nothing.

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
// hemisphere and baked sun share, and its surface's slice of the material table; whether it is a swaying tree's leaf,
// and the normal the enhanced foliage lights such a leaf by, the sphere about the tree's foot its point stands on.
struct SceneSurface {
  position: vec3<f32>,
  normal: vec3<f32>,
  albedo: vec3<f32>,
  hemi: f32,
  sun: f32,
  slice: f32,
  is_leaf: bool,
  crown: vec3<f32>,
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
// Metres a ray to the sun from a traced hit starts off it, and reaches at most.
const SCENE_SUN_BIAS: f32 = 0.05;
const SCENE_SUN_REACH: f32 = 1000.0;
// A record's flags, as `StaticTraceRecord` sets them: a cut-out geometry's, and the arena a tree's or a spawned model's
// vertices are in, the baked one where neither.
const TRACE_RECORD_CUT_OUT: u32 = 1u;
const TRACE_RECORD_TREE: u32 = 2u;
const TRACE_RECORD_MODEL: u32 = 4u;
// No place, which a ray passes none of.
const SCENE_NO_PLACE: u32 = 0xffffffffu;

// The nearest of the level's static geometry along a ray in renderer space between `near` and `far` metres, a
// cut-out section's holes letting it through.
fn trace_scene_ray(origin: vec3<f32>, direction: vec3<f32>, near: f32, far: f32, cone: SceneCone) -> SceneHit {
  return traverse_solid(RayDesc(RAY_FLAG_NONE, 0xffu, near, far, origin, direction), cone, SCENE_NO_PLACE);
}

// Whether anything solid of the level's static geometry stands along a ray within `far` metres, the cut-out geometry
// at `passed` (a place, or `SCENE_NO_PLACE`) aside.
fn trace_scene_blocked(origin: vec3<f32>, direction: vec3<f32>, far: f32, cone: SceneCone, passed: u32) -> bool {
  return traverse_solid(RayDesc(RAY_FLAG_TERMINATE_ON_FIRST_HIT, 0xffu, 0.0, far, origin, direction), cone, passed)
    .distance >= 0.0;
}

// Traces a ray, taking each cut-out triangle it passes where its texel there is solid, but those at the place `passed`,
// an opaque section taken by the hardware without asking; what it met, its distance below none for nothing.
fn traverse_solid(ray: RayDesc, cone: SceneCone, passed: u32) -> SceneHit {
  var query: ray_query;

  rayQueryInitialize(&query, trace_scene, ray);

  while (rayQueryProceed(&query)) {
    let candidate: RayIntersection = rayQueryGetCandidateIntersection(&query);
    let instance: TraceInstance = trace_instances[candidate.instance_custom_data];
    let record: u32 = instance.first_record + candidate.geometry_index;

    if (candidate.kind == RAY_QUERY_INTERSECTION_TRIANGLE && instance.place != passed &&
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

  // What the test reads alone, each candidate a leaf a ray passes may be: the surface's base slot, tiling and reference,
  // and the triangle's outline.
  let slot: u32 = trace_surfaces[record.surface].textures[SLOT_BASE];
  let tiling: f32 = trace_surfaces[record.surface].tiling;
  let triangle: SceneTriangle = scene_outline(record, primitive);
  let lod: f32 = texture_lod(tiling, triangle, slot, width, 1.0);
  let uv: vec2<f32> = base_uv(tiling, triangle, barycentric_weights(barycentrics));
  let alpha: f32 = textureSampleLevel(trace_textures[slot], trace_sampler, uv, lod).a;

  return alpha * (1.0 + 0.25 * lod) > trace_surfaces[record.surface].alpha_reference;
}

// A hit's surface: its corners read from its layout's arena, interpolated, its normal turned into renderer space and
// its hemisphere scaled as its place's, or a spawned model's taken from its place's cube where the level's lighting
// was estimated for it, as the G-buffer takes it; its albedo as the G-buffer lays it, each texture read at the level of
// detail the ray's cone, `width` wide at the hit, covers.
fn scene_surface(hit: SceneHit, width: f32) -> SceneSurface {
  let record: TraceRecord = trace_records[hit.record];
  let surface: Surface = trace_surfaces[record.surface];
  let triangle: SceneTriangle = scene_triangle(record, hit.primitive);
  let weights: vec3<f32> = barycentric_weights(hit.barycentrics);
  let slot: u32 = surface.textures[SLOT_BASE];
  let uv: vec2<f32> = base_uv(surface.tiling, triangle, weights);
  let base: vec4<f32> = textureSampleLevel(trace_textures[slot], trace_sampler, uv,
    texture_lod(surface.tiling, triangle, slot, width, 1.0));
  let normal: vec4<f32> = triangle.normals[0] * weights.x + triangle.normals[1] * weights.y +
    triangle.normals[2] * weights.z;
  let place: Place = trace_places[hit.place];
  let world_normal: vec3<f32> = normalize(hit.to_world * normal.xyz);
  var hemi: f32 = normal.w * place.info.x + place.info.y;
  var sun: f32 = 1.0;

  if ((record.flags & TRACE_RECORD_MODEL) != 0u && place.cube.w != 0u) {
    hemi = cube_hemi(place.cube, world_normal);
  }

  var albedo: vec3<f32> = base.rgb * scene_detail(surface, triangle, uv, width);

  // A terrain is lit by its base's alpha, as the G-buffer lights it, and covered by the grass planted on it; a
  // lightmapped surface by its lightmap.
  if ((surface.flags & SURFACE_IS_TERRAIN) != 0u) {
    let grass: SceneGrass = scene_grass(hit.position, world_normal);

    hemi = mix(base.a, grass.hemi, grass.cover);
    albedo = mix(albedo, grass.albedo, grass.cover);
  }

  if ((surface.flags & SURFACE_HAS_HEMI) != 0u) {
    let lightmap_uv: vec2<f32> = triangle.lightmaps[0] * weights.x + triangle.lightmaps[1] * weights.y +
      triangle.lightmaps[2] * weights.z;
    let lightmap: vec4<f32> = textureSampleLevel(trace_textures[surface.textures[SLOT_HEMI]], trace_sampler,
      lightmap_uv, 0.0);

    hemi = lightmap.a;
    sun = lightmap.g;
  }

  // A leaf's point in its tree's own space, about its foot, as `vs_tree` lays the crown's sphere.
  let local: vec3<f32> = triangle.positions[0] * weights.x + triangle.positions[1] * weights.y +
    triangle.positions[2] * weights.z;
  let is_leaf: bool = (record.flags & (TRACE_RECORD_TREE | TRACE_RECORD_CUT_OUT)) ==
    (TRACE_RECORD_TREE | TRACE_RECORD_CUT_OUT) && (surface.flags & SURFACE_IS_STILL) == 0u;
  let crown: vec3<f32> = normalize(hit.to_world * normalize(local + vec3<f32>(0.0, 1e-4, 0.0)));

  return SceneSurface(hit.position, world_normal, albedo, hemi, sun, surface.slice, is_leaf, crown);
}

// What a surface's details multiply its base by, as the G-buffer lays them: a terrain's four by its mask, a surface's
// one, each doubled; one for a surface without. A terrain's layers blend by its mask alone, not sharpened by their
// heights as the G-buffer's are.
fn scene_detail(surface: Surface, triangle: SceneTriangle, uv: vec2<f32>, width: f32) -> vec3<f32> {
  let scale: f32 = surface.detail_scale;

  if ((surface.flags & SURFACE_IS_TERRAIN) != 0u) {
    let mask_slot: u32 = surface.terrain.mask;
    let mask: vec4<f32> = textureSampleLevel(trace_textures[mask_slot], trace_sampler, uv,
      texture_lod(surface.tiling, triangle, mask_slot, width, 1.0));
    let weights: vec4<f32> = mask / max(dot(mask, vec4<f32>(1.0)), 1e-4);
    var detail: vec3<f32> = vec3<f32>(0.0);

    for (var layer: u32 = 0u; layer < 4u; layer++) {
      if (weights[layer] > TERRAIN_LAYER_LEAST) {
        let slot: u32 = surface.terrain.details[layer];

        detail += textureSampleLevel(trace_textures[slot], trace_sampler, uv * scale,
          texture_lod(surface.tiling, triangle, slot, width, scale)).rgb * weights[layer];
      }
    }

    return detail * 2.0;
  }

  if ((surface.flags & SURFACE_HAS_DETAIL) != 0u) {
    let slot: u32 = surface.textures[SLOT_DETAIL];

    return textureSampleLevel(trace_textures[slot], trace_sampler, uv * scale,
      texture_lod(surface.tiling, triangle, slot, width, scale)).rgb * 2.0;
  }

  return vec3<f32>(1.0);
}

// A traced hit lit as combine lights the G-buffer, unfogged: `hmodel` over its base and its baked hemisphere, the sun
// as `plight_infinity` lights it where a ray to the sun is not blocked, and the view's lamps. `toward` is the ray's
// direction in renderer space and `cone` its cone; `state` the frame's lighting, `lut` the material table and the
// cubes the hemisphere's environment. The ray to the sun reads the holes it passes as coarsely as the hit was read.
fn shade_scene_hit(
  hit: SceneHit,
  toward: vec3<f32>,
  cone: SceneCone,
  state: Lighting,
  lut: texture_3d<f32>,
  lut_sampler: sampler,
  environment_0: texture_cube<f32>,
  environment_1: texture_cube<f32>,
  cube_sampler: sampler,
) -> vec3<f32> {
  let width: f32 = cone.width + cone.spread * hit.distance;
  let met: SceneSurface = scene_surface(hit, width);
  // Under the enhanced foliage a leaf is lit as the frame lights it: by its crown's sphere rather than its card, never
  // darker than three tenths of the sky, and passing the sun through from behind (`frame/sun.wgsl`). Lit by its card,
  // a crown the water mirrors from below turned every leaf from the sun and reflected black.
  let is_crown: bool = met.is_leaf && state.flora.x > 0.5;
  let facing: vec3<f32> = select(select(met.normal, -met.normal, dot(met.normal, toward) > 0.0), met.crown, is_crown);
  let lifted: vec3<f32> = met.position + facing * SCENE_SUN_BIAS;
  let to_sun: vec3<f32> = normalize((transpose(camera.view) * vec4<f32>(state.to_sun.xyz, 0.0)).xyz);
  let half_way: vec3<f32> = normalize(to_sun - toward);
  let lit: vec4<f32> = textureSampleLevel(lut, lut_sampler,
    vec3<f32>(dot(to_sun, facing), dot(half_way, facing), met.slice), 0.0);
  // A tree's own leaves do not shade it: its sun ray starts among them, and passing through the whole crown it blacked
  // the far side out, which the frame's foliage, lit through its leaves, never shows.
  let own: u32 = select(SCENE_NO_PLACE, hit.place, (trace_records[hit.record].flags & TRACE_RECORD_TREE) != 0u);
  var sun: vec4<f32> = vec4<f32>(0.0);

  let through: vec3<f32> = select(vec3<f32>(0.0), saturate(dot(-toward, -(to_sun + facing))) * state.flora.y *
    mix(vec3<f32>(1.0), state.sun.rgb, state.flora.z), is_crown);

  if ((lit.x > 0.0 || any(through > vec3<f32>(0.0))) &&
    !trace_scene_blocked(lifted, to_sun, SCENE_SUN_REACH, SceneCone(width, 0.0), own)) {
    sun = vec4<f32>(state.sun.rgb * lit.x + through, state.sun.w * lit.y);
  }

  let lamps: vec4<f32> = scene_lamps(lifted, facing, toward, met.slice, lut, lut_sampler);
  let occlusion: f32 = mix(1.0, select(met.hemi, clamp(met.hemi, 0.3, 1.0), is_crown), camera.switches.z);

  return hmodel(state, lut, lut_sampler, environment_0, environment_1, cube_sampler, vec4<f32>(met.albedo, 0.0),
    sun + lamps, facing, toward, met.slice, occlusion, vec3<f32>(1.0));
}

// What the view's local lights give a point a ray met, its `normal` facing the ray, as the lights pass gives a pixel
// (`frame/lights.wgsl`): each light's falloff, a spot's cone and projector, a shadowed light's map; diffuse in colour,
// specular in alpha, the material table `lut` read for its surface's `slice`. The point stands off the view's
// clusters, so every light of the view's list is tried by its reach; a light culled from the view lights no hit.
fn scene_lamps(
  position: vec3<f32>,
  normal: vec3<f32>,
  toward: vec3<f32>,
  slice: f32,
  lut: texture_3d<f32>,
  lut_sampler: sampler,
) -> vec4<f32> {
  let view_position: vec3<f32> = (camera.view * vec4<f32>(position, 1.0)).xyz;
  let view_normal: vec3<f32> = normalize((camera.view * vec4<f32>(normal, 0.0)).xyz);
  let to_eye: vec3<f32> = -normalize((camera.view * vec4<f32>(toward, 0.0)).xyz);
  let offset: vec3<f32> = view_position + view_normal * VIRTUAL_OFFSET;
  let is_soft: bool = trace_light_view.shadow_filter == 1u;
  var total: vec4<f32> = vec4<f32>(0.0);

  for (var index: u32 = 0u; index < trace_light_view.count; index++) {
    // Its position and reach alone first: most of the view's lights reach no hit, and a record is ten vectors.
    let reach: vec4<f32> = trace_lights[index].position;
    let apart: f32 = max(length(view_position - reach.xyz) - VIRTUAL_OFFSET, 0.0);

    if (apart * apart * reach.w >= 1.0) {
      continue;
    }

    let record: LightRecord = trace_lights[index];
    let is_spot: bool = record.axis.w > -1.0;
    let is_shadowed: bool = record.shadow.z > 0.0;
    let to_point: vec3<f32> = select(view_position, offset, is_spot || is_shadowed) - record.position.xyz;
    let falloff: f32 = light_falloff(record, to_point);

    if (falloff <= 0.0 || is_outside_cone(record, to_point)) {
      continue;
    }

    let to_light: vec3<f32> = normalize(-to_point);
    let half_way: vec3<f32> = normalize(to_light + to_eye);
    let material: vec4<f32> = textureSampleLevel(lut, lut_sampler,
      vec3<f32>(dot(to_light, view_normal), dot(half_way, view_normal), slice), 0.0);
    var light: vec4<f32> = vec4<f32>(material.xxx, material.y) * falloff;

    if (is_spot && record.up.w >= 0.0) {
      let slot: u32 = u32(record.up.w);
      let uv: vec2<f32> = projector_uv(record, to_point, vec2<f32>(textureDimensions(trace_textures[slot])));

      light *= textureSampleLevel(trace_textures[slot], trace_sampler, uv, 0.0);
    }

    if (is_shadowed) {
      light *= light_shadow(trace_shadow_atlas, record, to_point, view_normal, is_spot, is_soft);
    }

    total += record.color * light;
  }

  return total;
}

// How much of a point of terrain the level's grass covers as the view plants it, the colour it covers it in and the
// hemisphere its slot was baked with. Each of the slot's four models is planted at a candidate as often as its density
// there says (`InterpolateAndDither` without the dither), one model a candidate where several pass, and none on a slope
// past the planting's limit; each tuft covers the disc its vertices spread over at its mean planted size, by the share
// of its texture its blades fill, the discs overlapping at random (`1 - e^-sum`); the slot shrinks to nothing towards
// the planting's fade, as its tufts do. None past a slot's heights, where the point stands off the ground it was
// planted on. Each model's colour and blade share are its texture's solid texels' (`grass/colours`).
struct SceneGrass {
  cover: f32,
  albedo: vec3<f32>,
  hemi: f32,
}

// Metres a point may stand off its slot's heights and still be the ground the slot's tufts stand on.
const SCENE_GRASS_REACH: f32 = 0.5;

fn scene_grass(position: vec3<f32>, normal: vec3<f32>) -> SceneGrass {
  let none: SceneGrass = SceneGrass(0.0, vec3<f32>(0.0), 0.0);
  let fade: f32 = saturate(1.0 - distance(position, camera.position.xyz) / max(trace_grass.fade, 1e-3));

  if (trace_grass.model_count == 0u || fade <= 0.0) {
    return none;
  }

  // The engine's space: renderer space with `z` negated.
  let engine: vec2<f32> = vec2<f32>(position.x, -position.z) / GRASS_SLOT_METERS;
  let slot: vec2<i32> = vec2<i32>(floor(engine));
  let cell: vec2<i32> = slot + trace_grass.grid.zw;

  if (any(cell < vec2<i32>(0)) || any(cell >= trace_grass.grid.xy)) {
    return none;
  }

  let record: u32 = trace_grass_grid[u32(cell.y * trace_grass.grid.x + cell.x)];

  if (record == 0u) {
    return none;
  }

  let at: u32 = (record - 1u) * GRASS_SLOT_WORDS;
  let w0: u32 = trace_grass_slots[at];
  let w1: u32 = trace_grass_slots[at + 1u];
  let w2: u32 = trace_grass_slots[at + 2u];
  let w3: u32 = trace_grass_slots[at + 3u];
  let base: f32 = f32(w0 & 0xfffu) * 0.2 - 200.0;
  let top: f32 = base + f32((w0 >> 12u) & 0xffu) * 0.1;

  if (position.y < base - SCENE_GRASS_REACH || position.y > top + SCENE_GRASS_REACH) {
    return none;
  }

  let ids: array<u32, 4> = array<u32, 4>((w0 >> 20u) & 0x3fu, (w0 >> 26u) & 0x3fu, w1 & 0x3fu, (w1 >> 6u) & 0x3fu);
  let palettes: array<u32, 4> = array<u32, 4>(w2 & 0xffffu, w2 >> 16u, w3 & 0xffffu, w3 >> 16u);
  let within: vec2<f32> = engine - vec2<f32>(slot);
  let candidates: f32 = f32((trace_grass.steps + 1) * (trace_grass.steps + 1));
  var densities: vec4<f32> = vec4<f32>(0.0);
  var missed: f32 = 1.0;

  for (var object: u32 = 0u; object < 4u; object++) {
    if (ids[object] != GRASS_EMPTY_ID && ids[object] < trace_grass.model_count) {
      let corners: vec4<f32> = vec4<f32>(f32(palettes[object] & 0xfu), f32((palettes[object] >> 4u) & 0xfu),
        f32((palettes[object] >> 8u) & 0xfu), f32((palettes[object] >> 12u) & 0xfu)) / 15.0;

      densities[object] = mix(mix(corners.x, corners.y, within.x), mix(corners.z, corners.w, within.x), within.y);
      missed *= 1.0 - densities[object];
    }
  }

  // The share of the candidates the slope leaves a tuft on: the planting draws each one's limit down by up to
  // `SLOPE_THINNING` of it, so the grass thins up a bank and stands on none of it past the limit. Without it a steep
  // bank's sparse tufts covered it as its slot's density says and greened its reflection.
  let slope: f32 = acos(clamp(abs(normal.y), -1.0, 1.0));
  let limit: f32 = trace_grass.slope_limit;
  let standing: f32 = select(saturate((limit - slope) / max(limit * SLOPE_THINNING, 1e-4)), 1.0, limit >= SLOPE_NONE);
  let planted: f32 = (1.0 - missed) * standing;
  let total: f32 = dot(densities, vec4<f32>(1.0));

  if (planted <= 0.0) {
    return none;
  }

  var cover: f32 = 0.0;
  var albedo: vec3<f32> = vec3<f32>(0.0);

  for (var object: u32 = 0u; object < 4u; object++) {
    if (densities[object] > 0.0) {
      let model: GrassModel = trace_grass_models[ids[object]];
      let radius: f32 = model.spread * (model.shape.x * 0.5 + model.shape.y * 0.9) * 0.5 * trace_grass.height;
      let footprint: f32 = candidates * 3.14159265 * radius * radius / (GRASS_SLOT_METERS * GRASS_SLOT_METERS);
      let colour: vec4<f32> = trace_grass_colours[ids[object]];
      let share: f32 = planted * densities[object] / total * footprint * colour.a;

      cover += share;
      albedo += colour.rgb * share;
    }
  }

  let hemi: f32 = f32((w1 >> 16u) & 0xfu) / 15.0;

  return SceneGrass((1.0 - exp(-cover)) * fade * fade, albedo / max(cover, 1e-4), hemi);
}

// A triangle's corners, read from its layout's arena through its record: its outline, then each corner's normal and,
// for a sector's or an instanced mesh's, its lightmap's coordinate as two shorts over 32768.
fn scene_triangle(record: TraceRecord, primitive: u32) -> SceneTriangle {
  var triangle: SceneTriangle = scene_outline(record, primitive);

  for (var corner: u32 = 0u; corner < 3u; corner++) {
    let at: u32 = scene_vertex(record, primitive, corner) * VERTEX_WORDS;
    let normal: vec4<f32> = unpack4x8unorm(trace_word(record.flags, at + VERTEX_NORMAL));

    triangle.normals[corner] = vec4<f32>(unpack_direction(normal), normal.w);

    if ((record.flags & (TRACE_RECORD_TREE | TRACE_RECORD_MODEL)) == 0u) {
      triangle.lightmaps[corner] = baked_lightmap_uv(trace_word(record.flags, at + VERTEX_EXTRA));
    }
  }

  return triangle;
}

// A triangle's outline alone, what a cut-out's test reads: its corners' positions and base coordinates, a sector's or
// an instanced mesh's as two shorts with their fraction, a tree's as two shorts over 2048, a spawned model's as two
// floats.
fn scene_outline(record: TraceRecord, primitive: u32) -> SceneTriangle {
  var triangle: SceneTriangle;

  for (var corner: u32 = 0u; corner < 3u; corner++) {
    let at: u32 = scene_vertex(record, primitive, corner) * VERTEX_WORDS;
    let uv: u32 = trace_word(record.flags, at + VERTEX_UV);

    triangle.positions[corner] = vertex_position(trace_word(record.flags, at + VERTEX_POSITION),
      trace_word(record.flags, at + VERTEX_POSITION + 1u), trace_word(record.flags, at + VERTEX_POSITION + 2u));

    if ((record.flags & TRACE_RECORD_TREE) != 0u) {
      triangle.uvs[corner] = unpack_shorts(uv) / 2048.0;
    } else if ((record.flags & TRACE_RECORD_MODEL) != 0u) {
      triangle.uvs[corner] = vec2<f32>(bitcast<f32>(uv), bitcast<f32>(trace_word(record.flags, at + VERTEX_EXTRA)));
    } else {
      triangle.uvs[corner] = baked_uv(uv, unpack4x8unorm(trace_word(record.flags, at + VERTEX_TANGENT)),
        unpack4x8unorm(trace_word(record.flags, at + VERTEX_BINORMAL)));
    }
  }

  return triangle;
}

// A triangle's corner's vertex in its layout's arena.
fn scene_vertex(record: TraceRecord, primitive: u32, corner: u32) -> u32 {
  return trace_indices[record.first_index + primitive * 3u + corner] + record.vertex_start;
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

// A point's base coordinate, tiled by its surface's `tiling` as the surface draws it.
fn base_uv(tiling: f32, triangle: SceneTriangle, weights: vec3<f32>) -> vec2<f32> {
  return (triangle.uvs[0] * weights.x + triangle.uvs[1] * weights.y + triangle.uvs[2] * weights.z) * tiling;
}

// The level of detail a texture is read at under a cone `width` wide, its coordinate the base's times `scale`: texels a
// metre of the triangle spans, from its area in each.
fn texture_lod(tiling: f32, triangle: SceneTriangle, slot: u32, width: f32, scale: f32) -> f32 {
  let size: vec2<f32> = vec2<f32>(textureDimensions(trace_textures[slot]));
  let world_area: f32 = length(cross(triangle.positions[1] - triangle.positions[0],
    triangle.positions[2] - triangle.positions[0]));
  let uv_area: f32 = abs(trace_cross(triangle.uvs[1] - triangle.uvs[0], triangle.uvs[2] - triangle.uvs[0])) *
    tiling * tiling * scale * scale;
  let density: f32 = sqrt(uv_area / max(world_area, 1e-8)) * max(size.x, size.y);

  return log2(max(width * density, 1.0));
}

fn trace_cross(a: vec2<f32>, b: vec2<f32>) -> f32 {
  return a.x * b.y - a.y * b.x;
}

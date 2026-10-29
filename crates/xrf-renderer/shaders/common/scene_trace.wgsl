enable wgpu_binding_array;

#import "common/camera"
#import "common/cut_out"
#import "common/scene_light"
#import "common/hmodel"
#import "common/local_lights"
#import "common/sun_shadow"
#import "common/axis_frame"
#import "grass/records"
#import "static/records"
#import "static/vertex"
#import "generated/common/scene_trace"

// Rays traced by hardware ray query against the level's static geometry (`StaticTrace`, bound by `TraceParameters`
// at group 3): each top-level instance names an entry of the instance table, its first record and its place; each
// geometry a record naming its layout's arena, where its triangles' indices and vertices start, and its surface; and
// the texture cache's bindless textures at group 4, which a hit is shaded from and a cut-out geometry's holes are
// tested in; and the view's local lights with the shadow atlas, which light a hit as the lights pass lights a pixel. A
// hit is lit from what the frame already holds where it can be: the sun's cascades shadow a hit they cover, as they
// shadow the frame's own surfaces, and only a hit past them traces its ray to the sun. A device without ray queries
// composes `common/scene_trace_none` in its place, whose trace meets nothing.

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
// The grass cards' top-level instance's custom index (`GrassCards::INSTANCE`), past any of the scene's; the scene's
// instances' mask (`StaticTrace::MASK`), which a ray meeting no grass asks for, and every instance's.
const SCENE_GRASS_CARDS: u32 = 0xffffffu;
const SCENE_MASK_SCENE: u32 = 0x01u;
const SCENE_MASK_ALL: u32 = 0xffu;

// Radians a ray's cone widens by past which its cut-out cards are met whole: a reflection that rough blurs any leaf's
// outline away.
const SCENE_OPAQUE_SPREAD: f32 = 0.1;

// The nearest of the level's static geometry along a ray in renderer space between `near` and `far` metres, a
// cut-out section's holes letting a narrow cone through.
fn trace_scene_ray(origin: vec3<f32>, direction: vec3<f32>, near: f32, far: f32, cone: SceneCone) -> SceneHit {
  // A wide cone's hit is blurred over its lobe past any leaf's outline: its cut-out cards are taken whole, by the
  // hardware, without asking each one's alpha.
  let flags: u32 = select(RAY_FLAG_NONE, RAY_FLAG_FORCE_OPAQUE, cone.spread > SCENE_OPAQUE_SPREAD);

  return traverse_solid(RayDesc(flags, SCENE_MASK_SCENE, near, far, origin, direction), cone, SCENE_NO_PLACE);
}

// Whether anything solid of the level's static geometry or the grass's cards the view stands near its eye stands along
// a ray within `far` metres, the cut-out geometry at `passed` (a place, or `SCENE_NO_PLACE`) aside.
fn trace_scene_blocked(origin: vec3<f32>, direction: vec3<f32>, far: f32, cone: SceneCone, passed: u32) -> bool {
  let ray: RayDesc = RayDesc(RAY_FLAG_TERMINATE_ON_FIRST_HIT, SCENE_MASK_ALL, 0.0, far, origin, direction);

  return traverse_solid(ray, cone, passed).distance >= 0.0;
}

// Traces a ray, taking each cut-out triangle it passes where its texel there is solid, but those at the place `passed`,
// an opaque section taken by the hardware without asking; what it met, its distance below none for nothing.
fn traverse_solid(ray: RayDesc, cone: SceneCone, passed: u32) -> SceneHit {
  var query: ray_query;

  rayQueryInitialize(&query, trace_scene, ray);

  while (rayQueryProceed(&query)) {
    let candidate: RayIntersection = rayQueryGetCandidateIntersection(&query);
    let width: f32 = cone.width + cone.spread * candidate.t;

    if (candidate.kind != RAY_QUERY_INTERSECTION_TRIANGLE) {
      continue;
    }

    if (candidate.instance_custom_data == SCENE_GRASS_CARDS) {
      if (is_grass_card_solid(candidate.primitive_index, candidate.barycentrics, width)) {
        rayQueryConfirmIntersection(&query);
      }

      continue;
    }

    let instance: TraceInstance = trace_instances[candidate.instance_custom_data];

    if (instance.place != passed &&
      is_scene_solid(instance.first_record + candidate.geometry_index, candidate.primitive_index,
        candidate.barycentrics, width, normals_to_world(candidate.world_to_object))) {
      rayQueryConfirmIntersection(&query);
    }
  }

  let hit: RayIntersection = rayQueryGetCommittedIntersection(&query);

  if (hit.kind == RAY_QUERY_INTERSECTION_NONE) {
    return SceneHit(-1.0, 0u, 0u, vec2<f32>(0.0), 0u, vec3<f32>(0.0), mat3x3<f32>());
  }

  // A card met, by a ray asking for every instance, answers its distance alone: such a ray asks only whether it met.
  if (hit.instance_custom_data == SCENE_GRASS_CARDS) {
    return SceneHit(hit.t, 0u, 0u, hit.barycentrics, SCENE_NO_PLACE, ray.origin + ray.dir * hit.t, mat3x3<f32>());
  }

  let instance: TraceInstance = trace_instances[hit.instance_custom_data];

  return SceneHit(hit.t, instance.first_record + hit.geometry_index, hit.primitive_index, hit.barycentrics,
    instance.place, ray.origin + ray.dir * hit.t, normals_to_world(hit.world_to_object));
}

// What turns an instance's geometry's normals into renderer space, from what takes renderer space into its geometry:
// the transpose of its inverse.
fn normals_to_world(world_to_object: mat4x3<f32>) -> mat3x3<f32> {
  return transpose(mat3x3<f32>(world_to_object[0], world_to_object[1], world_to_object[2]));
}

// Whether a cut-out section is solid at a point of a triangle: its base texture's alpha at the level of detail the
// cone's `width` there reads, raised as minification thins it as the G-buffer's cut raises it (`is_alpha_cut`), over
// its surface's reference; `to_world` turns its instance's normals into renderer space.
fn is_scene_solid(index: u32, primitive: u32, barycentrics: vec2<f32>, width: f32, to_world: mat3x3<f32>) -> bool {
  let record: TraceRecord = trace_records[index];

  if ((record.flags & TRACE_RECORD_CUT_OUT) == 0u) {
    return true;
  }

  // What the test reads alone, each candidate a leaf a ray passes may be: the surface's base slot, tiling and
  // reference, and the triangle's outline.
  let slot: u32 = trace_surfaces[record.surface].textures[SLOT_BASE];
  let tiling: f32 = trace_surfaces[record.surface].tiling;
  let triangle: SceneTriangle = scene_outline(record, primitive);
  let lod: f32 = texture_lod(tiling, triangle, slot, width, 1.0, scene_area(triangle, to_world));
  let uv: vec2<f32> = base_uv(tiling, triangle, barycentric_weights(barycentrics));
  let alpha: f32 = textureSampleLevel(trace_textures[slot], trace_sampler, uv, lod).a;

  return !is_alpha_cut_at(alpha, lod, trace_surfaces[record.surface].alpha_reference);
}

// Whether a grass card is solid at a point of its triangle `primitive`: its model's texture's alpha at the level of
// detail the cone's `width` there reads, raised as minification thins it, over the model's reference, as
// `is_scene_solid` tests a cut-out section's.
fn is_grass_card_solid(primitive: u32, barycentrics: vec2<f32>, width: f32) -> bool {
  let at: u32 = primitive * 4u;
  let head: u32 = trace_grass_cards[at];
  let model: GrassModel = trace_grass_models[head & 0xffffu];
  let density: f32 = unpack2x16float(head >> 16u).x;
  let weights: vec3<f32> = barycentric_weights(barycentrics);
  let uv: vec2<f32> = unpack2x16float(trace_grass_cards[at + 1u]) * weights.x +
    unpack2x16float(trace_grass_cards[at + 2u]) * weights.y + unpack2x16float(trace_grass_cards[at + 3u]) * weights.z;
  let size: vec2<f32> = vec2<f32>(textureDimensions(trace_textures[model.texture]));
  let lod: f32 = log2(max(width * density * max(size.x, size.y), 1.0));
  let alpha: f32 = textureSampleLevel(trace_textures[model.texture], trace_sampler, uv, lod).a;

  return !is_alpha_cut_at(alpha, lod, model.alpha_reference);
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
  let area: f32 = scene_area(triangle, hit.to_world);
  let base: vec4<f32> = textureSampleLevel(trace_textures[slot], trace_sampler, uv,
    texture_lod(surface.tiling, triangle, slot, width, 1.0, area));
  let normal: vec4<f32> = triangle.normals[0] * weights.x + triangle.normals[1] * weights.y +
    triangle.normals[2] * weights.z;
  let place: Place = trace_places[hit.place];
  let world_normal: vec3<f32> = normalize(hit.to_world * normal.xyz);
  var hemi: f32 = normal.w * place.info.x + place.info.y;
  var sun: f32 = 1.0;

  if ((record.flags & TRACE_RECORD_MODEL) != 0u && place.cube.w != 0u) {
    hemi = cube_hemi(place.cube, world_normal);
  }

  var albedo: vec3<f32> = base.rgb * scene_detail(surface, triangle, uv, width, area);

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

  return SceneSurface(hit.position, world_normal, authored_colour(albedo), hemi, sun, surface.slice, is_leaf, crown);
}

// What a surface's details multiply its base by, as the G-buffer lays them: a terrain's four by its mask, a surface's
// one, each doubled; one for a surface without. A terrain's layers blend by its mask alone, not sharpened by their
// heights as the G-buffer's are. `area` is the triangle's in renderer space.
fn scene_detail(surface: Surface, triangle: SceneTriangle, uv: vec2<f32>, width: f32, area: f32) -> vec3<f32> {
  let scale: f32 = surface.detail_scale;

  if ((surface.flags & SURFACE_IS_TERRAIN) != 0u) {
    let mask_slot: u32 = surface.terrain.mask;
    let mask: vec4<f32> = textureSampleLevel(trace_textures[mask_slot], trace_sampler, uv,
      texture_lod(surface.tiling, triangle, mask_slot, width, 1.0, area));
    let weights: vec4<f32> = mask / max(dot(mask, vec4<f32>(1.0)), 1e-4);
    var detail: vec3<f32> = vec3<f32>(0.0);

    for (var layer: u32 = 0u; layer < 4u; layer++) {
      if (weights[layer] > TERRAIN_LAYER_LEAST) {
        let slot: u32 = surface.terrain.details[layer];

        detail += textureSampleLevel(trace_textures[slot], trace_sampler, uv * scale,
          texture_lod(surface.tiling, triangle, slot, width, scale, area)).rgb * weights[layer];
      }
    }

    return detail * 2.0;
  }

  if ((surface.flags & SURFACE_HAS_DETAIL) != 0u) {
    let slot: u32 = surface.textures[SLOT_DETAIL];

    return textureSampleLevel(trace_textures[slot], trace_sampler, uv * scale,
      texture_lod(surface.tiling, triangle, slot, width, scale, area)).rgb * 2.0;
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
  // darker than three tenths of the sky, and passing the sun through from behind (`frame/sun.wgsl`), so a crown the
  // water mirrors from below is not turned from the sun.
  let is_crown: bool = met.is_leaf && state.flora.x > 0.5;
  let facing: vec3<f32> = select(select(met.normal, -met.normal, dot(met.normal, toward) > 0.0), met.crown, is_crown);
  let lifted: vec3<f32> = met.position + facing * SCENE_SUN_BIAS;
  let to_sun: vec3<f32> = normalize((transpose(camera.view) * vec4<f32>(state.to_sun.xyz, 0.0)).xyz);
  let lit: vec4<f32> = light_terms(lut, lut_sampler, to_sun, facing, toward, met.slice);
  // A tree's own leaves do not shade it: its sun ray starts among them, and the frame lights foliage through its
  // leaves.
  let own: u32 = select(SCENE_NO_PLACE, hit.place, (trace_records[hit.record].flags & TRACE_RECORD_TREE) != 0u);
  var sun: vec4<f32> = vec4<f32>(0.0);

  let through: vec3<f32> = select(vec3<f32>(0.0), flora_through(state, -toward, to_sun, facing), is_crown);

  if (lit.x > 0.0 || any(through > vec3<f32>(0.0))) {
    sun = vec4<f32>(state.sun.rgb * lit.x + through, state.sun.w * lit.y) *
      scene_sun_lit(met.position, facing, to_sun, SceneCone(width, 0.0), own);
  }

  let lamps: vec4<f32> = scene_lamps(lifted, facing, toward, met.slice, lut, lut_sampler);
  let occlusion: f32 = mix(1.0, select(met.hemi, clamp(met.hemi, 0.3, 1.0), is_crown), camera.switches.z);
  let visible: f32 = scene_occlusion(lifted, facing, width, own);

  return hmodel(state, lut, lut_sampler, environment_0, environment_1, cube_sampler, vec4<f32>(met.albedo, 0.0),
    sun + lamps, facing, toward, met.slice, occlusion, vec3<f32>(visible));
}

// How much of the sun reaches a traced point: the cascades' share where one covers it, sampled along `normal` as the
// frame's surfaces are; past them, whether a ray towards the sun meets nothing solid, the cut-out geometry at `passed`
// aside.
fn scene_sun_lit(position: vec3<f32>, normal: vec3<f32>, to_sun: vec3<f32>, cone: SceneCone, passed: u32) -> f32 {
  if (is_sun_mapped(trace_sun_shadows, position)) {
    return sun_shadow(trace_sun_maps, trace_sun_shadows, position, normal, dot(normal, to_sun));
  }

  return f32(!trace_scene_blocked(position + normal * SCENE_SUN_BIAS, to_sun, SCENE_SUN_REACH, cone, passed));
}

// Metres the occlusion's rays reach from a traced hit, and how many it casts: opposite each other about a turn its
// place picks, the reflections' filtering evening out what two rays leave coarse.
const SCENE_OCCLUSION_REACH: f32 = 1.5;
const SCENE_OCCLUSION_RAYS: u32 = 2u;

// How much of the light about it reaches a traced hit, which its hemisphere and ambient are lit by as the frame's
// occlusion lets them: the share of a few short rays from it, half way between its normal and the ground, turned by its
// place, that meet nothing solid within `SCENE_OCCLUSION_REACH`, so the trace reads as dark as the screen's own
// reflection of the same bank, which the frame's occlusion darkens.
fn scene_occlusion(position: vec3<f32>, normal: vec3<f32>, width: f32, passed: u32) -> f32 {
  let frame: AxisFrame = axis_frame(normal);
  let across: vec3<f32> = frame.across;
  let along: vec3<f32> = frame.along;
  let turn: f32 = fract(sin(dot(position, vec3<f32>(12.9898, 78.233, 37.719))) * 43758.547) * 6.2831853;
  var open: f32 = 0.0;

  for (var ray: u32 = 0u; ray < SCENE_OCCLUSION_RAYS; ray++) {
    let angle: f32 = turn + f32(ray) * 6.2831853 / f32(SCENE_OCCLUSION_RAYS);
    let direction: vec3<f32> = normalize(normal + across * cos(angle) + along * sin(angle));

    // Solid geometry alone: testing a cut-out card's alpha at each one the ray passes costs a third of the occlusion,
    // for leaves within a metre and a half that the hit's own hemisphere darkens already.
    let probe: RayDesc = RayDesc(RAY_FLAG_TERMINATE_ON_FIRST_HIT | RAY_FLAG_CULL_NO_OPAQUE, 0xffu, 0.0,
      SCENE_OCCLUSION_REACH, position, direction);

    open += f32(traverse_solid(probe, SceneCone(width, 0.0), passed).distance < 0.0);
  }

  return open / f32(SCENE_OCCLUSION_RAYS);
}

// The grass a ray passed through, lit as the frame lights its tufts: its blades facing the ray across and leaning up as
// the frame's flora normal leans them, the sun through a ray towards it from where the ray first met grass and passed
// through from behind under the enhanced foliage, and the hemisphere its slots were baked with.
fn shade_scene_grass(
  grass: SceneGrassPassed,
  origin: vec3<f32>,
  toward: vec3<f32>,
  state: Lighting,
  lut: texture_3d<f32>,
  lut_sampler: sampler,
  environment_0: texture_cube<f32>,
  environment_1: texture_cube<f32>,
  cube_sampler: sampler,
) -> vec3<f32> {
  let up: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0);
  let across: vec3<f32> = -vec3<f32>(toward.x, 0.0, toward.z) / max(length(toward.xz), 1e-4);
  let facing: vec3<f32> = normalize(across + up);
  let point: vec3<f32> = origin + toward * grass.entry;
  let to_sun: vec3<f32> = normalize((transpose(camera.view) * vec4<f32>(state.to_sun.xyz, 0.0)).xyz);
  let lit: vec4<f32> = light_terms(lut, lut_sampler, to_sun, facing, toward, GRASS_MATERIAL_SLICE);
  let through: vec3<f32> = select(vec3<f32>(0.0), flora_through(state, -toward, to_sun, facing), state.flora.x > 0.5);
  var sun: vec4<f32> = vec4<f32>(0.0);

  if (lit.x > 0.0 || any(through > vec3<f32>(0.0))) {
    sun = vec4<f32>(state.sun.rgb * lit.x + through, state.sun.w * lit.y) *
      scene_sun_lit(point, up, to_sun, SceneCone(0.0, 0.0), SCENE_NO_PLACE);
  }

  let occlusion: f32 = mix(1.0, grass.hemi, camera.switches.z);

  return hmodel(state, lut, lut_sampler, environment_0, environment_1, cube_sampler, vec4<f32>(authored_colour(grass.albedo), 0.0), sun,
    facing, toward, GRASS_MATERIAL_SLICE, occlusion, vec3<f32>(1.0));
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
    let material: vec4<f32> = light_terms(lut, lut_sampler, to_light, view_normal, -to_eye, slice);
    var light: vec4<f32> = vec4<f32>(material.xxx, material.y) *
      light_share(record, to_point, is_falloff_inverse_square(), 0.0);

    if (is_spot && record.up.w >= 0.0) {
      let slot: u32 = u32(record.up.w);
      let uv: vec2<f32> = projector_uv(record, to_point, vec2<f32>(textureDimensions(trace_textures[slot])));

      let projected: vec4<f32> = textureSampleLevel(trace_textures[slot], trace_sampler, uv, 0.0);

      light *= vec4<f32>(authored_colour(projected.rgb), projected.a);
    }

    if (is_shadowed) {
      light *= light_shadow(trace_shadow_atlas, trace_shadow_sampler, record, to_point, view_normal, is_spot, is_soft);
    }

    total += record.color * light;
  }

  return total;
}

// The level's grass about a slot as the view plants it: its ground's lowest and highest heights; how much of the ground
// its tufts cover from above, summed before they overlap; how much of a metre of a ray crossing them sideways they
// stop, likewise; their mean height; the colour they are of and the hemisphere the slot was baked with; whether any
// stand. Each of the slot's four models is planted at a candidate as often as its density there says
// (`InterpolateAndDither` without the dither), one model a candidate where several pass, as many of them as `standing`
// leaves; each tuft is the disc its vertices spread over at its mean planted size, as tall as its model at that size,
// by the share of its texture its blades fill (`grass/colours`): its disc seen from above, its width by its height
// sideways.
struct SceneGrassSlot {
  base: f32,
  top: f32,
  cover: f32,
  extinction: f32,
  height: f32,
  albedo: vec3<f32>,
  hemi: f32,
  is_planted: bool,
}

// The slot `slot` (in the engine's space, slots across) at `within` it, nought to one each way, its tufts thinned to
// the share `standing` of them.
fn scene_grass_slot(slot: vec2<i32>, within: vec2<f32>, standing: f32) -> SceneGrassSlot {
  var out: SceneGrassSlot;
  let cell: vec2<i32> = slot + trace_grass.grid.zw;

  if (trace_grass.model_count == 0u || any(cell < vec2<i32>(0)) || any(cell >= trace_grass.grid.xy)) {
    return out;
  }

  let record: u32 = trace_grass_grid[u32(cell.y * trace_grass.grid.x + cell.x)];

  if (record == 0u) {
    return out;
  }

  let at: u32 = (record - 1u) * GRASS_SLOT_WORDS;
  let w0: u32 = trace_grass_slots[at];
  let w1: u32 = trace_grass_slots[at + 1u];
  let w2: u32 = trace_grass_slots[at + 2u];
  let w3: u32 = trace_grass_slots[at + 3u];
  let ids: array<u32, 4> = array<u32, 4>((w0 >> 20u) & 0x3fu, (w0 >> 26u) & 0x3fu, w1 & 0x3fu, (w1 >> 6u) & 0x3fu);
  let palettes: array<u32, 4> = array<u32, 4>(w2 & 0xffffu, w2 >> 16u, w3 & 0xffffu, w3 >> 16u);
  var densities: vec4<f32> = vec4<f32>(0.0);
  var missed: f32 = 1.0;

  out.base = f32(w0 & 0xfffu) * 0.2 - 200.0;
  out.top = out.base + f32((w0 >> 12u) & 0xffu) * 0.1;
  out.hemi = f32((w1 >> 16u) & 0xfu) / 15.0;

  for (var object: u32 = 0u; object < 4u; object++) {
    if (ids[object] != GRASS_EMPTY_ID && ids[object] < trace_grass.model_count) {
      let corners: vec4<f32> = vec4<f32>(f32(palettes[object] & 0xfu), f32((palettes[object] >> 4u) & 0xfu),
        f32((palettes[object] >> 8u) & 0xfu), f32((palettes[object] >> 12u) & 0xfu)) / 15.0;

      densities[object] = mix(mix(corners.x, corners.y, within.x), mix(corners.z, corners.w, within.x), within.y);
      missed *= 1.0 - densities[object];
    }
  }

  let planted: f32 = (1.0 - missed) * standing;
  let total: f32 = dot(densities, vec4<f32>(1.0));

  if (planted <= 0.0) {
    return out;
  }

  // Tufts a square metre: the slot's candidates over its area, as many planted as its densities say.
  let tufts: f32 = planted * f32((trace_grass.steps + 1) * (trace_grass.steps + 1)) /
    (GRASS_SLOT_METERS * GRASS_SLOT_METERS);
  var counted: f32 = 0.0;

  for (var object: u32 = 0u; object < 4u; object++) {
    if (densities[object] > 0.0) {
      let model: GrassModel = trace_grass_models[ids[object]];
      let colour: vec4<f32> = trace_grass_colours[ids[object]];
      let size: f32 = (model.shape.x * 0.5 + model.shape.y * 0.9) * 0.5 * trace_grass.height;
      let radius: f32 = model.spread * size;
      let count: f32 = tufts * densities[object] / total;
      let blades: f32 = count * colour.a;

      out.cover += blades * 3.14159265 * radius * radius;
      out.extinction += blades * 2.0 * radius;
      out.albedo += colour.rgb * blades * radius;
      out.height += count * model.shape.w * size;
      counted += count;
    }
  }

  out.albedo /= max(out.extinction * 0.5, 1e-6);
  out.height /= max(counted, 1e-6);
  out.is_planted = true;

  return out;
}

// How much of a point of terrain the level's grass covers as the view plants it, the colour it covers it in and the
// hemisphere its slot was baked with: its slot's tufts, overlapping at random (`1 - e^-cover`), none on a slope past
// the planting's limit, the slot shrinking to nothing towards the planting's fade as its tufts do. None past a slot's
// heights, where the point stands off the ground it was planted on.
struct SceneGrass {
  cover: f32,
  albedo: vec3<f32>,
  hemi: f32,
}

// Metres a point may stand off its slot's heights and still be the ground the slot's tufts stand on.
const SCENE_GRASS_REACH: f32 = 0.5;

fn scene_grass(position: vec3<f32>, normal: vec3<f32>) -> SceneGrass {
  let none: SceneGrass = SceneGrass(0.0, vec3<f32>(0.0), 0.0);
  let fade: f32 = scene_grass_fade(position);

  if (fade <= 0.0) {
    return none;
  }

  // The share of the candidates the slope leaves a tuft on: the planting draws each one's limit down by up to
  // `SLOPE_THINNING` of it, so the grass thins up a bank and stands on none of it past the limit.
  let slope: f32 = acos(clamp(abs(normal.y), -1.0, 1.0));
  let limit: f32 = trace_grass.slope_limit;
  let standing: f32 = select(saturate((limit - slope) / max(limit * SLOPE_THINNING, 1e-4)), 1.0, limit >= SLOPE_NONE);
  // The engine's space: renderer space with `z` negated.
  let engine: vec2<f32> = vec2<f32>(position.x, -position.z) / GRASS_SLOT_METERS;
  let slot: vec2<i32> = vec2<i32>(floor(engine));
  let grass: SceneGrassSlot = scene_grass_slot(slot, engine - vec2<f32>(slot), standing);

  if (!grass.is_planted || position.y < grass.base - SCENE_GRASS_REACH ||
    position.y > grass.top + SCENE_GRASS_REACH) {
    return none;
  }

  return SceneGrass((1.0 - exp(-grass.cover)) * fade * fade, grass.albedo, grass.hemi);
}

// How far the planting reaches at a point: one at the eye, nought at its fade.
fn scene_grass_fade(position: vec3<f32>) -> f32 {
  return saturate(1.0 - distance(position, camera.position.xyz) / max(trace_grass.fade, 1e-3));
}

// The level's grass a ray passes through before what it met: how much of what lies behind the grass hides, the colour
// it hides it with and the hemisphere its slots were baked with, and how far along the ray it first meets grass.
struct SceneGrassPassed {
  opacity: f32,
  albedo: vec3<f32>,
  hemi: f32,
  entry: f32,
}

// Metres a ray is walked through the grass at most, and a step of the walk; and the metres before what the ray met
// that it is not, where the grass the hit stands in covers it instead (`scene_grass`).
const SCENE_GRASS_WALK: f32 = 32.0;
const SCENE_GRASS_STEP: f32 = 0.5;
const SCENE_GRASS_SHORT: f32 = 2.0;
// Metres over its start a ray rising is walked to: the grass a reflection passes stands on the shores about the water.
const SCENE_GRASS_RISE: f32 = 3.0;

// The grass a ray from `origin` along `toward` passes through before what it met `length` metres on (or its reach,
// `is_met` false), each slot's tufts a layer as tall as they stand over its ground's middle height, stopping a ray
// crossing it as their sideways extinction says. Only a slot whose ground spans less than its tufts stand is walked,
// where that middle is the ground: on a bank the layer would float over one side and sink into the other, and the
// bank's own grass covers it where the ray meets it. A reflection's ray leaves the view's grass cards out, which stand
// only near the eye, so tall reeds stood in a reflection as the sky behind them.
fn scene_grass_passed(origin: vec3<f32>, toward: vec3<f32>, length: f32, is_met: bool) -> SceneGrassPassed {
  var out: SceneGrassPassed = SceneGrassPassed(0.0, vec3<f32>(0.0), 0.0, -1.0);

  if (trace_grass.model_count == 0u) {
    return out;
  }

  let risen: f32 = select(SCENE_GRASS_WALK, SCENE_GRASS_RISE / toward.y, toward.y > 0.0);
  let reach: f32 = min(min(select(length, length - SCENE_GRASS_SHORT, is_met), SCENE_GRASS_WALK), risen);
  var through: f32 = 1.0;
  var last: vec2<i32> = vec2<i32>(-2147483647);
  var grass: SceneGrassSlot;

  for (var along: f32 = SCENE_GRASS_STEP * 0.5; along < reach; along += SCENE_GRASS_STEP) {
    let point: vec3<f32> = origin + toward * along;
    let fade: f32 = scene_grass_fade(point);

    if (fade <= 0.0) {
      continue;
    }

    let engine: vec2<f32> = vec2<f32>(point.x, -point.z) / GRASS_SLOT_METERS;
    let slot: vec2<i32> = vec2<i32>(floor(engine));

    if (any(slot != last)) {
      last = slot;
      grass = scene_grass_slot(slot, vec2<f32>(0.5), 1.0);
    }

    let above: f32 = point.y - (grass.base + grass.top) * 0.5;

    if (!grass.is_planted || grass.top - grass.base > grass.height || above < 0.0 || above > grass.height) {
      continue;
    }

    let stopped: f32 = 1.0 - exp(-grass.extinction * fade * fade * SCENE_GRASS_STEP);

    out.albedo += grass.albedo * through * stopped;
    out.hemi += grass.hemi * through * stopped;
    out.entry = select(out.entry, along, out.entry < 0.0);
    through *= 1.0 - stopped;

    if (through < 0.02) {
      break;
    }
  }

  out.opacity = 1.0 - through;
  out.albedo /= max(out.opacity, 1e-4);
  out.hemi /= max(out.opacity, 1e-4);

  return out;
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
// metre of the triangle spans, from its area in each, `area` its renderer space's.
fn texture_lod(tiling: f32, triangle: SceneTriangle, slot: u32, width: f32, scale: f32, area: f32) -> f32 {
  let size: vec2<f32> = vec2<f32>(textureDimensions(trace_textures[slot]));
  let uv_area: f32 = abs(trace_cross(triangle.uvs[1] - triangle.uvs[0], triangle.uvs[2] - triangle.uvs[0])) *
    tiling * tiling * scale * scale;
  let density: f32 = sqrt(uv_area / max(area, 1e-8)) * max(size.x, size.y);

  return log2(max(width * density, 1.0));
}

// A triangle's area in renderer space, its geometry's corners carried by its instance: the cross of its edges, turned
// as a normal by `to_world` (the transpose of the instance's inverse), scaled by the instance's determinant, which is
// that matrix's inverse.
fn scene_area(triangle: SceneTriangle, to_world: mat3x3<f32>) -> f32 {
  let across: vec3<f32> = cross(triangle.positions[1] - triangle.positions[0], triangle.positions[2] -
    triangle.positions[0]);

  return length(to_world * across) / max(abs(determinant(to_world)), 1e-12);
}

fn trace_cross(a: vec2<f32>, b: vec2<f32>) -> f32 {
  return a.x * b.y - a.y * b.x;
}

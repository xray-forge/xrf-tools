#import "common/compute_grid"
#import "common/camera"
#import "static/records"

// Decides which clusters a frame draws: each visible one is appended to its batch's run of the list, which the
// batch's indirect draw then draws as instances.
//
// Occlusion is culled in two phases. The early phase keeps what last frame's depth pyramid, seen through last frame's
// view, does not hide, and sets what it hides aside as candidates; once the kept clusters are drawn and this frame's
// pyramid is reduced from their depth, the late phase tests the candidates again and appends those it now sees after
// the early ones, for a second draw.

// Whether the view culled is a shadow's: its casters drawn as trees whatever their impostor says, at the band and
// discard the camera's distance picks, and never occluded.
override IS_SHADOW: bool = false;

// Whether the shadow is a light's face, kept while nothing it casts from changes: its trees cast at their finest band,
// and nothing too small for the camera is dropped, so no move of the camera is drawn into it.
override IS_FINEST: bool = false;

struct CullParams {
  cluster_count: u32,
  row_count: u32,
  batch_count: u32,
  impostor_count: u32,
  // Squared screen area thresholds, as the engine's `ssa` compares them.
  glod_start: f32,
  glod_end: f32,
  discard_below: f32,
  candidate_capacity: u32,
  is_occluding: u32,
  // Screen areas an impostor draws below, and its trees above.
  lod_a: f32,
  lod_b: f32,
  is_impostors: u32,
  // A bit a visibility group, from the lowest, set where the view hides it.
  hidden_groups: u32,
  pad1: u32,
  pad2: u32,
  pad3: u32,
  // xyz: the camera every view's levels of detail are measured from.
  lod_origin: vec4<f32>,
};

// The view the pyramid was reduced through, which the early phase tests against.
struct Occlusion {
  view: mat4x4<f32>,
  projection: mat4x4<f32>,
  // The pyramid's first level's size.
  size: vec2<f32>,
  levels: u32,
  // Whether the pyramid holds an earlier frame's depth drawn through this view.
  has_history: u32,
};

@group(1) @binding(0) var<storage, read> clusters: array<Cluster>;
@group(1) @binding(1) var<storage, read> spheres: array<vec4<f32>>;
@group(1) @binding(2) var<storage, read> slots: array<Slot>;
@group(1) @binding(3) var<storage, read> places: array<Place>;
@group(1) @binding(4) var<storage, read> rows: array<Row>;
@group(1) @binding(5) var<storage, read> regions: array<Region>;
@group(1) @binding(6) var<storage, read_write> lists: array<vec2<u32>>;
@group(1) @binding(7) var<storage, read_write> args: array<atomic<u32>>;
@group(1) @binding(8) var<uniform> params: CullParams;
@group(1) @binding(9) var<storage, read_write> candidates: array<vec2<u32>>;
// Each batch's late draw arguments, then the late phase's dispatch and the candidates' count.
@group(1) @binding(10) var<storage, read_write> late: array<atomic<u32>>;
@group(1) @binding(11) var pyramid: texture_2d<f32>;
@group(1) @binding(12) var<uniform> occlusion: Occlusion;
@group(1) @binding(13) var<storage, read> impostors: array<Impostor>;
// Each impostor's best facet, the next best, its fade and blend bytes, and what its level of detail draws.
@group(1) @binding(14) var<storage, read_write> terms: array<vec4<u32>>;
@group(1) @binding(15) var<storage, read_write> impostor_list: array<u32>;

// `EPS_S`, the least a fade's range is taken as.
const RANGE_EPSILON: f32 = 1e-6;

fn is_in_frustum(sphere: vec4<f32>) -> bool {
  if (sphere.w < 0.0) {
    return false;
  }

  for (var plane: u32 = 0u; plane < 6u; plane++) {
    let p: vec4<f32> = camera.planes[plane];

    if (dot(p.xyz, sphere.xyz) + p.w < -sphere.w) {
      return false;
    }
  }

  return true;
}

// `r / d²`, the engine's screen area of a sphere from the camera, without the screen it is compared against.
fn screen_area(sphere: vec4<f32>) -> f32 {
  let offset: vec3<f32> = sphere.xyz - params.lod_origin.xyz;

  return sphere.w / (dot(offset, offset) + 1e-7);
}

// The four words after the batches' draw arguments: clusters and triangles drawn, then occluded, for the readouts; the
// impostors' draw arguments follow them, their instance count the sixth word.
fn stats_at(word: u32) -> u32 {
  return params.batch_count * 4u + word;
}

// The word of the late buffer after the batches' late draw arguments: the dispatch's three, then the candidates'.
fn late_at(word: u32) -> u32 {
  return params.batch_count * 4u + word;
}

fn put(batch: u32, index: u32, cluster: u32, place: u32) {
  let region: Region = regions[batch];

  if (index < region.capacity) {
    lists[region.base + index] = vec2<u32>(cluster, place);
    atomicAdd(&args[stats_at(0u)], 1u);
    atomicAdd(&args[stats_at(1u)], clusters[cluster].triangles);
  }
}

fn append(batch: u32, cluster: u32, place: u32) {
  put(batch, atomicAdd(&args[batch * 4u + 1u], 1u), cluster, place);
}

fn set_aside(cluster: u32, place: u32) {
  let index: u32 = atomicAdd(&late[late_at(3u)], 1u);

  if (index < params.candidate_capacity) {
    candidates[index] = vec2<u32>(cluster, place);
  }
}

// Whether a sphere lies wholly behind the depth the pyramid holds, seen through a view: its screen rectangle is read at
// the level where it spans at most two texels a side, and its nearest point compared with the farthest depth there.
fn is_occluded(sphere: vec4<f32>, view: mat4x4<f32>, projection: mat4x4<f32>) -> bool {
  let center: vec3<f32> = (view * vec4<f32>(sphere.xyz, 1.0)).xyz;
  let ahead: f32 = -center.z;
  let radius: f32 = sphere.w;

  // A sphere reaching the eye covers the screen.
  if (ahead <= radius) {
    return false;
  }

  // Its tangent lines in each axis, as the projection of a sphere bounds it.
  let cx: vec2<f32> = vec2<f32>(center.x, ahead);
  let vx: vec2<f32> = vec2<f32>(sqrt(dot(cx, cx) - radius * radius), radius);
  let min_x: vec2<f32> = mat2x2<f32>(vx.x, vx.y, -vx.y, vx.x) * cx;
  let max_x: vec2<f32> = mat2x2<f32>(vx.x, -vx.y, vx.y, vx.x) * cx;
  let cy: vec2<f32> = vec2<f32>(center.y, ahead);
  let vy: vec2<f32> = vec2<f32>(sqrt(dot(cy, cy) - radius * radius), radius);
  let min_y: vec2<f32> = mat2x2<f32>(vy.x, vy.y, -vy.y, vy.x) * cy;
  let max_y: vec2<f32> = mat2x2<f32>(vy.x, -vy.y, vy.y, vy.x) * cy;
  let ndc: vec4<f32> = vec4<f32>(
    min_x.x / min_x.y * projection[0][0],
    min_y.x / min_y.y * projection[1][1],
    max_x.x / max_x.y * projection[0][0],
    max_y.x / max_y.y * projection[1][1]
  );
  let low: vec2<f32> = clamp(vec2<f32>(ndc.x, -ndc.w) * 0.5 + 0.5, vec2<f32>(0.0), vec2<f32>(1.0));
  let high: vec2<f32> = clamp(vec2<f32>(ndc.z, -ndc.y) * 0.5 + 0.5, vec2<f32>(0.0), vec2<f32>(1.0));

  if (any(high <= low)) {
    return false;
  }

  let span: vec2<f32> = (high - low) * occlusion.size;
  let level: u32 = min(u32(max(ceil(log2(max(span.x, span.y))), 0.0)), occlusion.levels - 1u);
  let size: vec2<u32> = textureDimensions(pyramid, level);
  let first: vec2<u32> = min(vec2<u32>(low * vec2<f32>(size)), size - 1u);
  let last: vec2<u32> = min(vec2<u32>(high * vec2<f32>(size)), size - 1u);
  let farthest: f32 = min(
    min(textureLoad(pyramid, first, level).x, textureLoad(pyramid, vec2<u32>(last.x, first.y), level).x),
    min(textureLoad(pyramid, vec2<u32>(first.x, last.y), level).x, textureLoad(pyramid, last, level).x)
  );
  let nearest: vec4<f32> = projection * vec4<f32>(0.0, 0.0, center.z + radius, 1.0);

  return nearest.z / nearest.w < farthest;
}

// Whether the early phase sets a cluster aside: hidden by last frame's depth.
fn is_hidden_early(sphere: vec4<f32>) -> bool {
  return params.is_occluding != 0u && occlusion.has_history != 0u &&
    is_occluded(sphere, occlusion.view, occlusion.projection);
}

// Appends a cluster the frustum keeps, or sets it aside for the late phase where last frame's depth hid it.
fn keep(batch: u32, cluster: u32, place: u32, sphere: vec4<f32>) {
  // Water, composited surfaces and wall marks cast no shadow.
  if (IS_SHADOW && batch % CLASS_COUNT >= WATER_CLASS) {
    return;
  }

  if (!IS_SHADOW && is_hidden_early(sphere)) {
    set_aside(cluster, place);
  } else {
    append(batch, cluster, place);
  }
}

// A cluster's sphere where its place stands it: moved by the place's matrix, grown by its largest scale.
fn placed_sphere(sphere: vec4<f32>, place: Place) -> vec4<f32> {
  let center: vec4<f32> = place_matrix(place) * vec4<f32>(sphere.xyz, 1.0);

  return vec4<f32>(center.xyz, sphere.w * place.info.w);
}

@compute @workgroup_size(64)
fn cull_singles(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let index: u32 = compute_index(id, groups, 64u);

  if (index >= params.cluster_count) {
    return;
  }

  let cluster: Cluster = clusters[index];
  let slot: Slot = slots[cluster.slot];

  if (cluster.triangles == 0u || slot.kind != SLOT_SINGLE) {
    return;
  }

  let sphere: vec4<f32> = placed_sphere(spheres[index], places[slot.place]);

  if (is_in_frustum(sphere)) {
    keep(slot.batch, index, slot.place, sphere);
  }
}

// `add_leafs_static`'s `MT_LOD` case and `render_lods`'s terms, one invocation an impostor. A clump's screen area is
// its sphere's over its squared distance, times its `lod_factor`: above `r_ssaLOD_B` its trees draw, below `r_ssaLOD_A`
// its impostor does, both between, neither below `r_ssaDISCARD`. For the impostor it picks the two facets facing the
// camera best, how far to blend between them, and how far it has faded in, as `render_lods` writes them.
@compute @workgroup_size(64)
fn cull_impostors(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let index: u32 = compute_index(id, groups, 64u);

  if (index >= params.impostor_count) {
    return;
  }

  let impostor: Impostor = impostors[index];
  let sphere: vec4<f32> = impostor.sphere;
  // An impostor without a sphere never stands in, so its clump always draws its trees.
  var state: u32 = select(LOD_TREES, 0u, sphere.w > 0.0);
  var best: u32 = 0u;
  var next: u32 = 0u;
  var bytes: u32 = 0u;

  if (sphere.w > 0.0) {
    let area: f32 = screen_area(sphere) * impostor.factor;

    if (params.is_impostors == 0u || area > params.lod_b) {
      state |= LOD_TREES;
    }

    if (params.is_impostors != 0u && area < params.lod_a && area >= params.discard_below) {
      let direction: vec3<f32> = normalize(sphere.xyz - camera.position.xyz);
      var first: f32 = -2.0;
      var second: f32 = -2.0;
      var third: f32 = -2.0;

      state |= LOD_IMPOSTOR;

      for (var facet: u32 = 0u; facet < IMPOSTOR_FACETS; facet++) {
        let facing: f32 = dot(direction, impostor.normals[facet].xyz);

        if (facing > first) {
          third = second;
          second = first;
          next = best;
          first = facing;
          best = facet;
        } else if (facing > second) {
          third = second;
          second = facing;
          next = facet;
        } else if (facing > third) {
          third = facing;
        }
      }

      let fade: f32 = 1.0 - (area - params.lod_b) / max(params.lod_a - params.lod_b, RANGE_EPSILON);
      let blend: f32 = 0.5 + 0.5 * (1.0 - (second - third) / max(first - third, RANGE_EPSILON));

      bytes = u32(clamp(floor(fade * 255.0), 0.0, 255.0)) | (u32(clamp(floor(blend * 255.5), 0.0, 255.0)) << 8u);
    }
  }

  terms[index] = vec4<u32>(best, next, bytes, state);

  if ((state & LOD_IMPOSTOR) != 0u && is_in_frustum(sphere)) {
    impostor_list[atomicAdd(&args[stats_at(5u)], 1u)] = index;
  }
}

// Whether a progressive mesh's band is the one its distance picks: windows are spread over bands evenly.
fn is_band_drawn(band_word: u32, area: f32) -> bool {
  let band: u32 = band_word & 255u;
  let bands: u32 = (band_word >> 8u) & 255u;
  let windows: u32 = (band_word >> 16u) & 255u;
  let detail: f32 = sqrt(clamp((area - params.glod_end) / (params.glod_start - params.glod_end), 0.0, 1.0));
  let window: u32 = u32(floor((1.0 - detail) * f32(max(windows, 1u) - 1u) + 0.5));

  return window * bands / max(windows, 1u) == band;
}

@compute @workgroup_size(64)
fn cull_rows(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let index: u32 = compute_index(id, groups, 64u);

  if (index >= params.row_count) {
    return;
  }

  let row: Row = rows[index];
  let group: u32 = row.band >> 24u;

  // A row of a visibility group the view hides.
  if (group != 0u && (params.hidden_groups & (1u << (group - 1u))) != 0u) {
    return;
  }

  // A tree its clump's impostor stands in for is drawn only while the clump is near enough; a shadow casts it always.
  if (!IS_SHADOW && row.lod != NO_LOD && (terms[row.lod].w & LOD_TREES) == 0u) {
    return;
  }

  let area: f32 = screen_area(row.sphere);

  let is_drawn: bool = select(
    // A place too small to see is dropped, as `r_ssaDISCARD` drops it.
    area > params.discard_below && is_band_drawn(row.band, area),
    (row.band & 255u) == 0u,
    IS_FINEST
  );

  if (!is_drawn || !is_in_frustum(row.sphere)) {
    return;
  }

  let slot: Slot = slots[row.slot];
  let place: Place = places[row.place];

  for (var offset: u32 = 0u; offset < slot.cluster_count; offset++) {
    let cluster: u32 = slot.first_cluster + offset;
    let sphere: vec4<f32> = placed_sphere(spheres[cluster], place);

    if (is_in_frustum(sphere)) {
      keep(slot.batch, cluster, row.place, sphere);
    }
  }
}

// Clamps each batch's instance count to its run once every cluster is appended, so an overflow draws what fits rather
// than reading into the next batch's run; its late draw starts where the early one ends, and the late phase is sized to
// the candidates.
@compute @workgroup_size(64)
fn clamp_counts(@builtin(global_invocation_id) id: vec3<u32>) {
  let batch: u32 = id.x;

  if (IS_SHADOW) {
    if (batch < params.batch_count) {
      atomicStore(&args[batch * 4u + 1u], min(atomicLoad(&args[batch * 4u + 1u]), regions[batch].capacity));
    }

    return;
  }

  if (batch == 0u) {
    let candidates: u32 = min(atomicLoad(&late[late_at(3u)]), params.candidate_capacity);

    atomicStore(&late[late_at(0u)], (candidates + 63u) / 64u);
    atomicStore(&late[late_at(3u)], candidates);
  }

  if (batch >= params.batch_count) {
    return;
  }

  let region: Region = regions[batch];
  let count: u32 = min(atomicLoad(&args[batch * 4u + 1u]), region.capacity);

  atomicStore(&args[batch * 4u + 1u], count);
  atomicStore(&late[batch * 4u + 3u], region.base + count);
}

// Tests each candidate again against this frame's pyramid, through this frame's view.
@compute @workgroup_size(64)
fn cull_late(@builtin(global_invocation_id) id: vec3<u32>) {
  let index: u32 = id.x;

  if (index >= atomicLoad(&late[late_at(3u)])) {
    return;
  }

  let candidate: vec2<u32> = candidates[index];
  let cluster: u32 = candidate.x;
  let batch: u32 = slots[clusters[cluster].slot].batch;
  let sphere: vec4<f32> = placed_sphere(spheres[cluster], places[candidate.y]);

  if (is_occluded(sphere, camera.view, camera.projection)) {
    atomicAdd(&args[stats_at(2u)], 1u);
    atomicAdd(&args[stats_at(3u)], clusters[cluster].triangles);
  } else {
    let early: u32 = atomicLoad(&args[batch * 4u + 1u]);

    put(batch, early + atomicAdd(&late[batch * 4u + 1u], 1u), cluster, candidate.y);
  }
}

// Clamps each batch's late count to what is left of its run after the early draw.
@compute @workgroup_size(64)
fn clamp_late(@builtin(global_invocation_id) id: vec3<u32>) {
  let batch: u32 = id.x;

  if (batch >= params.batch_count) {
    return;
  }

  let early: u32 = atomicLoad(&args[batch * 4u + 1u]);
  let count: u32 = atomicLoad(&late[batch * 4u + 1u]);

  atomicStore(&late[batch * 4u + 1u], min(count, regions[batch].capacity - min(early, regions[batch].capacity)));
}

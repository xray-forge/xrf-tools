#import "common/compute_grid"
#import "common/camera"
#import "common/pyramid_occlusion"
#import "static/records"
#import "generated/static/cull"

// Decides which clusters a frame draws: each visible one is appended to its batch's run of the list, which the
// batch's indirect draw then draws as instances, and to the run of each surface pass its surface takes part in, which
// that pass draws alone.
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

// Whether the shadow is an overhead map of the level's water, which keeps its water alone.
override IS_WATER: bool = false;

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

// The draw lists there are: one a batch, then one a layout and surface pass.
fn list_count() -> u32 {
  return params.batch_count + params.batch_count / CLASS_COUNT * SURFACE_PASS_COUNT;
}

// The draw list of a batch's layout's surfaces taking part in a surface pass.
fn pass_list(batch: u32, surface_pass: u32) -> u32 {
  return params.batch_count + batch / CLASS_COUNT * SURFACE_PASS_COUNT + surface_pass;
}

// The four words after the draw lists' arguments: clusters and triangles drawn, then occluded, for the readouts; the
// impostors' draw arguments follow them, their instance count the sixth word.
fn stats_at(word: u32) -> u32 {
  return list_count() * 4u + word;
}

// The word of the late buffer after the draw lists' late arguments: the dispatch's three, then the candidates'.
fn late_at(word: u32) -> u32 {
  return list_count() * 4u + word;
}

// Writes a cluster at an index of a draw list's run; a cluster is counted for the readouts once, by its batch's.
fn put(list: u32, index: u32, cluster: u32, place: u32) {
  let region: Region = regions[list];

  if (index < region.capacity) {
    lists[region.base + index] = vec2<u32>(cluster, place);

    if (list < params.batch_count) {
      atomicAdd(&args[stats_at(0u)], 1u);
      atomicAdd(&args[stats_at(1u)], clusters[cluster].triangles);
    }
  }
}

// Appends a cluster to a draw list's run: the early phase from its start, the late one after where the early ended.
fn append(list: u32, cluster: u32, place: u32, is_late: bool) {
  if (is_late) {
    put(list, atomicLoad(&args[list * 4u + 1u]) + atomicAdd(&late[list * 4u + 1u], 1u), cluster, place);
  } else {
    put(list, atomicAdd(&args[list * 4u + 1u], 1u), cluster, place);
  }
}

// Appends a cluster to its batch's list and to each surface pass's its slot takes part in.
fn append_drawn(slot: Slot, cluster: u32, place: u32, is_late: bool) {
  append(slot.batch, cluster, place, is_late);

  for (var surface_pass: u32 = 0u; surface_pass < SURFACE_PASS_COUNT; surface_pass++) {
    if ((slot.passes & (1u << surface_pass)) != 0u) {
      append(pass_list(slot.batch, surface_pass), cluster, place, is_late);
    }
  }
}

fn set_aside(cluster: u32, place: u32) {
  let index: u32 = atomicAdd(&late[late_at(3u)], 1u);

  if (index < params.candidate_capacity) {
    candidates[index] = vec2<u32>(cluster, place);
  }
}

// Whether a sphere lies wholly behind the depth the pyramid holds, seen through a view.
fn is_occluded(sphere: vec4<f32>, view: mat4x4<f32>, projection: mat4x4<f32>) -> bool {
  return is_sphere_occluded(pyramid, occlusion.size, occlusion.levels, sphere, view, projection);
}

// Whether the early phase sets a cluster aside: hidden by last frame's depth.
fn is_hidden_early(sphere: vec4<f32>) -> bool {
  return params.is_occluding != 0u && occlusion.has_history != 0u &&
    is_occluded(sphere, occlusion.view, occlusion.projection);
}

// Appends a cluster the frustum keeps, or sets it aside for the late phase where last frame's depth hid it.
fn keep(slot: Slot, cluster: u32, place: u32, sphere: vec4<f32>) {
  // Water, composited surfaces and wall marks cast no shadow; a map of the water keeps its water alone.
  let kind: u32 = slot.batch % CLASS_COUNT;

  if (IS_SHADOW && select(kind >= WATER_CLASS, kind != WATER_CLASS, IS_WATER)) {
    return;
  }

  if (!IS_SHADOW && is_hidden_early(sphere)) {
    set_aside(cluster, place);
  } else {
    append_drawn(slot, cluster, place, false);
  }
}

// A cluster's sphere where its place stands it: moved by the place's matrix, grown by its largest scale.
fn placed_sphere(sphere: vec4<f32>, place: Place) -> vec4<f32> {
  let center: vec4<f32> = place.transform * vec4<f32>(sphere.xyz, 1.0);

  return vec4<f32>(center.xyz, sphere.w * place.info.w);
}

@compute @workgroup_size(64)
fn cs_cull_singles(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
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
    keep(slot, index, slot.place, sphere);
  }
}

// `add_leafs_static`'s `MT_LOD` case and `render_lods`'s terms, one invocation an impostor. A clump's screen area is
// its sphere's over its squared distance, times its `lod_factor`: above `r_ssaLOD_B` its trees draw, below `r_ssaLOD_A`
// its impostor does, both between, neither below `r_ssaDISCARD`. For the impostor it picks the two facets facing the
// camera best, how far to blend between them, and how far it has faded in, as `render_lods` writes them.
@compute @workgroup_size(64)
fn cs_cull_impostors(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
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
  let band: u32 = row_band(band_word);
  let bands: u32 = row_bands(band_word);
  let windows: u32 = row_windows(band_word);
  let detail: f32 = sqrt(clamp((area - params.glod_end) / (params.glod_start - params.glod_end), 0.0, 1.0));
  let window: u32 = u32(floor((1.0 - detail) * f32(max(windows, 1u) - 1u) + 0.5));

  return window * bands / max(windows, 1u) == band;
}

@compute @workgroup_size(64)
fn cs_cull_rows(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let index: u32 = compute_index(id, groups, 64u);

  if (index >= params.row_count) {
    return;
  }

  let row: Row = rows[index];

  // A tree its clump's impostor stands in for is drawn only while the clump is near enough; a shadow casts it always.
  if (!IS_SHADOW && row.lod != NO_LOD && (terms[row.lod].w & LOD_TREES) == 0u) {
    return;
  }

  let area: f32 = screen_area(row.sphere);

  let is_drawn: bool = select(
    // A place too small to see is dropped, as `r_ssaDISCARD` drops it.
    area > params.discard_below && is_band_drawn(row.band, area),
    row_band(row.band) == 0u,
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
      keep(slot, cluster, row.place, sphere);
    }
  }
}

// Clamps each draw list's instance count to its run once every cluster is appended, so an overflow draws what fits
// rather than reading into the next list's run; its late draw starts where the early one ends, and the late phase is
// sized to the candidates.
@compute @workgroup_size(64)
fn cs_clamp_counts(@builtin(global_invocation_id) id: vec3<u32>) {
  let list: u32 = id.x;

  if (IS_SHADOW) {
    if (list < list_count()) {
      atomicStore(&args[list * 4u + 1u], min(atomicLoad(&args[list * 4u + 1u]), regions[list].capacity));
    }

    return;
  }

  if (list == 0u) {
    let candidates: u32 = min(atomicLoad(&late[late_at(3u)]), params.candidate_capacity);

    atomicStore(&late[late_at(0u)], (candidates + 63u) / 64u);
    atomicStore(&late[late_at(3u)], candidates);
  }

  if (list >= list_count()) {
    return;
  }

  let region: Region = regions[list];
  let count: u32 = min(atomicLoad(&args[list * 4u + 1u]), region.capacity);

  atomicStore(&args[list * 4u + 1u], count);
  atomicStore(&late[list * 4u + 3u], region.base + count);
}

// Tests each candidate again against this frame's pyramid, through this frame's view.
@compute @workgroup_size(64)
fn cs_cull_late(@builtin(global_invocation_id) id: vec3<u32>) {
  let index: u32 = id.x;

  if (index >= atomicLoad(&late[late_at(3u)])) {
    return;
  }

  let candidate: vec2<u32> = candidates[index];
  let cluster: u32 = candidate.x;
  let slot: Slot = slots[clusters[cluster].slot];
  let sphere: vec4<f32> = placed_sphere(spheres[cluster], places[candidate.y]);

  if (is_occluded(sphere, camera.view, camera.projection)) {
    atomicAdd(&args[stats_at(2u)], 1u);
    atomicAdd(&args[stats_at(3u)], clusters[cluster].triangles);
  } else {
    append_drawn(slot, cluster, candidate.y, true);
  }
}

// Clamps each draw list's late count to what is left of its run after the early draw.
@compute @workgroup_size(64)
fn cs_clamp_late(@builtin(global_invocation_id) id: vec3<u32>) {
  let list: u32 = id.x;

  if (list >= list_count()) {
    return;
  }

  let early: u32 = atomicLoad(&args[list * 4u + 1u]);
  let count: u32 = atomicLoad(&late[list * 4u + 1u]);

  atomicStore(&late[list * 4u + 1u], min(count, regions[list].capacity - min(early, regions[list].capacity)));
}

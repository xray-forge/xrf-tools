#import "common/camera"
#import "common/compute_grid"
#import "common/lighting"
#import "common/fog_colour"
#import "common/octahedral"
#import "common/reflection_end"
#import "common/reflection_trace"
#import "common/scene_light"
#import "common/display_transform"
#import "common/scene_trace"
#import "common/sky_box"
#import "common/water_rays"
#import "generated/frame/reflection_scene"
#import "generated/frame/scene_list"
#import "generated/frame/water_rays"
#import "generated/common/sky"

// The reflections' scene pass: the rays the trace left unanswered on their turn (`TraceRequest`), traced against the
// level's geometry by hardware ray query in waves of nothing but such rays, so the screen's walk runs without the
// trace's registers and no wave waits on a few of its lanes. A ray meeting something writes its pixel's answer over the
// sky the trace wrote: the screen's weighed answer and the hit's light by the share the screen left, the hit's length
// where the screen held less than half, and the reason the trace gave it; a miss leaves the sky. A water's request
// (`TRACE_REQUEST_WATER`) meets the grass's cards too where the view traces them, and is answered into the water's
// reflection (`answer_water`) with the grass it passes past them. The rays come in two ways, each binding its own
// group: a list's (`cs_scene`, group 5), or while every water pixel traces every frame a record at each pixel
// (`cs_water_scene`, group 6, `common/water_rays`). A hit a screen showed takes its colour from that screen, so a traced
// reflection reads as the screen's own: the surfaces' from the last frame's scene and depth, which their trace keeps,
// the water's from this frame's copy before the water.

// Where the screen `view_projection` drew shows a world point, and in `w` one where it shows it: read where the
// screen's depth agrees within `SOURCE_DEPTH_TOLERANCE` of the point's distance. None off the screen, or in front of or
// behind what it shows.
fn screen_place(point: vec3<f32>, view_projection: mat4x4<f32>) -> vec4<f32> {
  let clip: vec4<f32> = view_projection * vec4<f32>(point, 1.0);

  if (clip.w <= 0.0) {
    return vec4<f32>(0.0);
  }

  let uv: vec2<f32> = clip.xy / clip.w * vec2<f32>(0.5, -0.5) + 0.5;

  if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
    return vec4<f32>(0.0);
  }

  let size: vec2<f32> = camera.viewport.xy;
  let texel: vec2<i32> = clamp(vec2<i32>(uv * size), vec2<i32>(0), vec2<i32>(size) - 1);
  let stored: f32 = textureLoad(screen_depth, texel, 0);

  if (stored <= 0.0 || abs(camera_view_distance(stored) - clip.w) > SOURCE_DEPTH_TOLERANCE * clip.w) {
    return vec4<f32>(0.0);
  }

  return vec4<f32>(vec2<f32>(texel), 0.0, 1.0);
}

// What the last frame showed of a surfaces' hit at a world point, as light, and in `w` one where it did: the tonemap
// undone at the scale that frame used and its fog to the point taken off, as the surfaces' screen hits read it
// (`remembered_radiance`). None where that frame did not show the point, or lost it in its fog.
fn remembered_light(point: vec3<f32>) -> vec4<f32> {
  let place: vec4<f32> = screen_place(point, camera.view_projection_previous);

  if (place.w < 0.5) {
    return vec4<f32>(0.0);
  }

  let position: vec3<f32> = (camera.view * vec4<f32>(point, 1.0)).xyz;
  let fog: f32 = fog_amount_at(lighting, position, point.y);

  if (fog > SOURCE_FOG_LIMIT) {
    return vec4<f32>(0.0);
  }

  let scale: f32 = previous_scale(lighting, exposure);
  let shown: vec3<f32> = scene_light(textureLoad(screen_scene, vec2<i32>(place.xy), 0).rgb, scale);
  return vec4<f32>(source_unfogged(shown, fog,
    fog_color_at(lighting, position, normalize(point - camera.position.xyz), point.y)), 1.0);
}

// The wait the list's pixels stretch their turns by the next frame: as many times as the rays they asked this frame,
// counted back to their turns unstretched, overrun the list's room. Past its room the list drops the pixels drawn last,
// as a block of the screen holding what it reflected the frame before; stretched, each pixel waits a turn or two more.
fn keep_trace_wait() {
  let wait: u32 = clamp(trace_request_head[TRACE_REQUEST_WAIT], 1u, TRACE_REQUEST_WAIT_MOST);
  let asked: f32 = f32(trace_request_head[0]) * f32(wait);
  let room: f32 = f32(arrayLength(&trace_requests)) * TRACE_REQUEST_FILL;

  trace_request_head[TRACE_REQUEST_WAIT] = clamp(u32(ceil(asked / room)), 1u, TRACE_REQUEST_WAIT_MOST);
}

// The water's every-pixel rays (`WaterRays`): a thread a drawn pixel, its record traced as the request it stands in
// for, its start read back from the water's depth there and lifted along the normal its reflection leaves by; a pixel
// no record was written at this frame (the screen answered it whole), or where no water drew, passed over.
@compute @workgroup_size(8, 8)
fn cs_water_scene(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec2<u32> = vec2<u32>(camera.viewport.xy);
  let at: u32 = water_ray_at(id.xy);

  if (any(id.xy >= size) || at + WATER_RAY_WORDS > arrayLength(&water_rays)) {
    return;
  }

  let last: u32 = water_rays[at + 3u];
  // Where no water drew this frame there is nothing to answer, even where a record of a frame its stamp's half wraps
  // to stands: its hit would mark a pixel of no water as some.
  let depth: f32 = textureLoad(water_depth, vec2<i32>(id.xy), 0);

  if (!is_water_ray_of(last, water_rays_frame.frame) || depth <= 0.0) {
    return;
  }

  let centre: vec2<f32> = (vec2<f32>(id.xy) + 0.5) / vec2<f32>(size);
  let world: vec3<f32> = camera_unproject(vec2<f32>(centre.x * 2.0 - 1.0, 1.0 - centre.y * 2.0), depth);
  let toward: vec3<f32> = normalize(octahedral_decode(unpack2x16snorm(water_rays[at])));
  // A mirror's ray leaves along its view turned about the normal: their difference is the normal.
  let leaving: vec3<f32> = toward - normalize(world - camera.position.xyz);
  let normal: vec3<f32> = select(vec3<f32>(0.0, 1.0, 0.0), normalize(leaving), dot(leaving, leaving) > 1e-8);
  let distance: f32 = -(camera.view * vec4<f32>(world, 1.0)).z;
  let pixel_angle: f32 = 2.0 / (camera.projection[1][1] * camera.viewport.y);
  var request: TraceRequest;

  request.origin = pack_trace_request_origin(world + normal * TRACED_LIFT);
  request.reach = pack2x16float(vec2<f32>(water_rays_frame.trace_distance, 0.0)) | (last << 16u);
  request.cone = pack2x16float(vec2<f32>(distance * pixel_angle, pixel_angle));
  request.toward = water_rays[at];
  request.pixel = id.x | (id.y << 16u);
  request.screen = vec2<u32>(water_rays[at + 1u], water_rays[at + 2u]);
  request.hit_end = TRACE_REQUEST_WATER | (pack2x16float(vec2<f32>(water_rays_frame.sky_share, 0.0)) & 0xffffu);

  trace_water_request(request);
}

// A water's request traced against the level's geometry and the grass's cards, answered into the water's reflection.
fn trace_water_request(request: TraceRequest) {
  let toward: vec3<f32> = normalize(octahedral_decode(unpack2x16snorm(request.toward)));
  let cone: SceneCone = request_cone(request);
  let screen: vec4<f32> = vec4<f32>(unpack2x16float(request.screen.x), unpack2x16float(request.screen.y));
  let pixel: vec2<u32> = vec2<u32>(request.pixel & 0xffffu, request.pixel >> 16u);
  let reach: f32 = unpack2x16float(request.reach).x;

  answer_water(request, toward, cone,
    trace_water_ray(trace_request_origin(request.origin), toward, 0.0, reach, cone), screen, pixel);
}

// A request's cone as it carries it, two halves.
fn request_cone(request: TraceRequest) -> SceneCone {
  let cone: vec2<f32> = unpack2x16float(request.cone);

  return SceneCone(cone.x, cone.y);
}

@compute @workgroup_size(64)
fn cs_scene(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let index: u32 = compute_index(id, groups, TRACE_REQUEST_WORKGROUP);

  if (index == 0u) {
    keep_trace_wait();
  }

  if (index >= min(trace_request_head[0], arrayLength(&trace_requests))) {
    return;
  }

  let request: TraceRequest = trace_requests[index];
  if ((request.hit_end & TRACE_REQUEST_WATER) != 0u) {
    trace_water_request(request);

    return;
  }

  let toward: vec3<f32> = normalize(octahedral_decode(unpack2x16snorm(request.toward)));
  let cone: SceneCone = request_cone(request);
  let screen: vec4<f32> = vec4<f32>(unpack2x16float(request.screen.x), unpack2x16float(request.screen.y));
  let pixel: vec2<u32> = vec2<u32>(request.pixel & 0xffffu, request.pixel >> 16u);
  let origin: vec3<f32> = trace_request_origin(request.origin);
  // How far the ray reaches, then the length of what the screen answered.
  let lengths: vec2<f32> = unpack2x16float(request.reach);
  let hit: SceneHit = trace_scene_ray(origin, toward, 0.0, lengths.x, cone);

  if (hit.distance < 0.0) {
    return;
  }

  let remembered: vec4<f32> = remembered_light(origin + toward * hit.distance);
  var shown: vec3<f32> = remembered.rgb;

  if (remembered.w < 0.5) {
    shown = shade_scene_hit(hit, toward, cone, lighting, material_lut, lut_sampler, sky_environment_0,
      sky_environment_1, sky_clamp);
  }
  let fog: f32 = path_fog(lighting, hit.distance);
  let met: vec3<f32> =
    fog_mixed(lighting, shown * min(1.0, RADIANCE_LIMIT / traced_luminance(shown)), fog_light(lighting, toward), fog);

  let answer: vec4<f32> = vec4<f32>(screen.rgb + met * (1.0 - screen.w),
    select(hit.distance, lengths.y, screen.w > 0.5));
  let mates: u32 = request.hit_end >> TRACE_REQUEST_MATES_SHIFT;
  let quad: vec2<u32> = pixel & vec2<u32>(~1u);

  textureStore(traced_out, pixel, answer);
  textureStore(ends_out, pixel, vec4<f32>(reflection_end_value(request.hit_end & TRACE_REQUEST_REASON), 0.0, 0.0, 0.0));

  // Under ray budgets, the pixels of its quad that take its answer.
  for (var corner: u32 = 0u; corner < 4u; corner++) {
    if ((mates & (1u << corner)) != 0u) {
      textureStore(traced_out, quad + vec2<u32>(corner & 1u, corner >> 1u), answer);
    }
  }
}

// What the frame's screen, copied before the water, shows of a water's hit at a world point, as the colour shown, and in
// `w` one where it shows it.
fn screen_shown_at(point: vec3<f32>) -> vec4<f32> {
  let place: vec4<f32> = screen_place(point, camera.view_projection);

  if (place.w < 0.5) {
    return vec4<f32>(0.0);
  }

  return vec4<f32>(scene_shown(textureLoad(screen_scene, vec2<i32>(place.xy), 0).rgb), 1.0);
}

// A water's ray answered: what it met as the screen shows it where it does, so a traced hit reads as the march's do,
// else lit and tonemapped as the scene copy the water reads was, or where it met nothing the sky as the water reflects
// it; a grass card it met lit as the frame lights its tufts where the screen does not show it, or past the cards the
// grass it passed on the way over what it met; the screen's weighed answer over both. Its distance is where the mirror
// shows the grass or the hit, the screen's where it met something; a ray meeting neither grass nor geometry keeps the
// sky its pass wrote.
fn answer_water(request: TraceRequest, toward: vec3<f32>, cone: SceneCone, hit: SceneHit, screen: vec4<f32>,
  pixel: vec2<u32>) {
  let is_card: bool = is_scene_grass_card(hit);
  let is_met: bool = hit.distance >= 0.0;
  let start: vec3<f32> = trace_request_origin(request.origin);
  // How far the ray reaches, then the length of what the screen answered.
  let lengths: vec2<f32> = unpack2x16float(request.reach);
  var grass: SceneGrassPassed;

  if (is_card) {
    grass = scene_card_grass(hit, cone.width + cone.spread * hit.distance);
  } else {
    grass = scene_grass_passed(start, toward, select(lengths.x, hit.distance, is_met), is_met);
  }

  if (!is_met && grass.opacity <= 0.0) {
    return;
  }

  let scale: f32 = frame_scale(lighting, exposure);
  // The sky's picture as the frame's display brings it in, as the water's own pass writes it.
  var behind: vec3<f32> = picture_for_display(
    enhanced_sky(toward, lighting, sky_cube_0, sky_cube_1, sky_clamp) * trace_request_sky_share(request.hit_end),
    lighting.display,
  );

  if (is_met) {
    let seen: vec4<f32> = screen_shown_at(hit.position);

    if (seen.w > 0.5) {
      behind = seen.rgb;
      // A card the screen shows reads as the screen shows its blades.
      grass.opacity = select(grass.opacity, 0.0, is_card);
    } else if (!is_card) {
      let shown: vec3<f32> = shade_scene_hit(hit, toward, cone, lighting, material_lut, lut_sampler, sky_environment_0,
        sky_environment_1, sky_clamp);

      behind = shown_from_light(
        fog_mixed(lighting, shown, fog_light(lighting, toward), path_fog(lighting, hit.distance)), scale);
    }
  }

  var met_length: f32 = hit.distance;

  if (grass.opacity > 0.0) {
    let lit: vec3<f32> = shade_scene_grass(grass, start, toward, lighting, material_lut, lut_sampler,
      sky_environment_0, sky_environment_1, sky_clamp);

    behind = mix(behind,
      shown_from_light(fog_mixed(lighting, lit, fog_light(lighting, toward), path_fog(lighting, grass.entry)), scale),
      grass.opacity);
    met_length = select(met_length, grass.entry, !is_met || grass.opacity > 0.5);
  }

  let origin: vec3<f32> = (camera.view * vec4<f32>(start, 1.0)).xyz;
  let along: f32 = -origin.z * (1.0 + met_length / max(length(origin), 1e-4));

  textureStore(traced_out, pixel, vec4<f32>(screen.rgb + behind * (1.0 - screen.w),
    select(along, lengths.y, lengths.y > 0.0)));
}

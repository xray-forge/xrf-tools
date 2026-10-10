#import "common/camera"
#import "common/compute_grid"
#import "common/lighting"
#import "common/octahedral"
#import "common/reflection_end"
#import "common/reflection_trace"
#import "common/scene_trace"
#import "common/sky_box"
#import "generated/frame/reflection_scene"
#import "generated/common/sky"

// The reflections' scene pass: the rays the trace left unanswered on their turn (`TraceRequest`), traced against the
// level's geometry by hardware ray query in waves of nothing but such rays, so the screen's walk runs without the
// trace's registers and no wave waits on a few of its lanes. A ray meeting something writes its pixel's answer over the
// sky the trace wrote: the screen's weighed answer and the hit's light by the share the screen left, the hit's length
// where the screen held less than half, and the reason the trace gave it; a miss leaves the sky. A water's request
// (`TRACE_REQUEST_WATER`) is answered into the water's reflection (`answer_water`), with the grass its ray passes.

// The wait the list's pixels stretch their turns by the next frame: as many times as the rays they asked this frame,
// counted back to their turns unstretched, overrun the list's room. Past its room the list drops the pixels drawn last,
// as a block of the screen holding what it reflected the frame before; stretched, each pixel waits a turn or two more.
fn keep_trace_wait() {
  let wait: u32 = clamp(trace_request_head[TRACE_REQUEST_WAIT], 1u, TRACE_REQUEST_WAIT_MOST);
  let asked: f32 = f32(trace_request_head[0]) * f32(wait);
  let room: f32 = f32(arrayLength(&trace_requests)) * TRACE_REQUEST_FILL;

  trace_request_head[TRACE_REQUEST_WAIT] = clamp(u32(ceil(asked / room)), 1u, TRACE_REQUEST_WAIT_MOST);
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
  let toward: vec3<f32> = normalize(octahedral_decode(unpack2x16snorm(request.toward)));
  let cone: SceneCone = SceneCone(request.cone.x, request.cone.y);
  let hit: SceneHit = trace_scene_ray(request.origin.xyz, toward, 0.0, request.origin.w, cone);
  let screen: vec4<f32> = vec4<f32>(unpack2x16float(request.screen.x), unpack2x16float(request.screen.y));
  let pixel: vec2<u32> = vec2<u32>(request.pixel & 0xffffu, request.pixel >> 16u);

  if ((request.hit_end & TRACE_REQUEST_WATER) != 0u) {
    answer_water(request, toward, cone, hit, screen, pixel);

    return;
  }

  if (hit.distance < 0.0) {
    return;
  }

  let shown: vec3<f32> = shade_scene_hit(hit, toward, cone, lighting, material_lut, lut_sampler, sky_environment_0,
    sky_environment_1, sky_clamp);
  let fog: f32 = path_fog(lighting, hit.distance);
  let met: vec3<f32> = mix(shown * min(1.0, RADIANCE_LIMIT / luminance(shown)), lighting.fog_color.rgb, fog);

  textureStore(traced_out, pixel, vec4<f32>(screen.rgb + met * (1.0 - screen.w),
    select(hit.distance, request.screen_length, screen.w > 0.5)));
  textureStore(ends_out, pixel, vec4<f32>(reflection_end_value(request.hit_end), 0.0, 0.0, 0.0));
}

// A water's ray answered: what it met tonemapped as the scene copy the water reads was, or where it met nothing the sky
// as the water reflects it; the grass it passed on the way over it; the screen's weighed answer over both. Its
// distance is where the mirror shows the grass or the hit, the screen's where it met something; a ray meeting neither
// grass nor geometry keeps the sky its pass wrote.
fn answer_water(request: TraceRequest, toward: vec3<f32>, cone: SceneCone, hit: SceneHit, screen: vec4<f32>,
  pixel: vec2<u32>) {
  let is_met: bool = hit.distance >= 0.0;
  let grass: SceneGrassPassed =
    scene_grass_passed(request.origin.xyz, toward, select(request.origin.w, hit.distance, is_met), is_met);

  if (!is_met && grass.opacity <= 0.0) {
    return;
  }

  let scale: f32 = frame_scale(lighting, exposure);
  var behind: vec3<f32> = enhanced_sky(toward, lighting, sky_cube_0, sky_cube_1, sky_clamp) *
    trace_request_sky_share(request.hit_end);

  if (is_met) {
    let shown: vec3<f32> = shade_scene_hit(hit, toward, cone, lighting, material_lut, lut_sampler, sky_environment_0,
      sky_environment_1, sky_clamp);

    behind = tonemap(mix(shown, lighting.fog_color.rgb, path_fog(lighting, hit.distance)), scale);
  }

  var met_length: f32 = hit.distance;

  if (grass.opacity > 0.0) {
    let lit: vec3<f32> = shade_scene_grass(grass, request.origin.xyz, toward, lighting, material_lut, lut_sampler,
      sky_environment_0, sky_environment_1, sky_clamp);

    behind = mix(behind, tonemap(mix(lit, lighting.fog_color.rgb, path_fog(lighting, grass.entry)), scale),
      grass.opacity);
    met_length = select(met_length, grass.entry, !is_met || grass.opacity > 0.5);
  }

  let origin: vec3<f32> = (camera.view * vec4<f32>(request.origin.xyz, 1.0)).xyz;
  let along: f32 = -origin.z * (1.0 + met_length / max(length(origin), 1e-4));

  textureStore(traced_out, pixel, vec4<f32>(screen.rgb + behind * (1.0 - screen.w),
    select(along, request.screen_length, request.screen_length > 0.0)));
}

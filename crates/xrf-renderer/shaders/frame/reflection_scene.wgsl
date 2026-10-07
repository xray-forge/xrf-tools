#import "common/camera"
#import "common/compute_grid"
#import "common/lighting"
#import "common/octahedral"
#import "common/reflection_end"
#import "common/reflection_trace"
#import "common/scene_trace"
#import "generated/frame/reflection_scene"
#import "generated/common/sky"

// The reflections' scene pass: the rays the trace left unanswered on their turn (`TraceRequest`), traced against the
// level's geometry by hardware ray query in waves of nothing but such rays, so the screen's walk runs without the
// trace's registers and no wave waits on a few of its lanes. A ray meeting something writes its pixel's answer over the
// sky the trace wrote: the screen's weighed answer and the hit's light by the share the screen left, the hit's length
// where the screen held less than half, and the reason the trace gave it; a miss leaves the sky. A water's request
// (`TRACE_REQUEST_WATER`) writes into the water's reflection, its hit tonemapped and unheld, the distance along the
// view where the mirror shows the hit, or the screen's where it met something, and no reason.

@compute @workgroup_size(64)
fn cs_scene(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let index: u32 = compute_index(id, groups, TRACE_REQUEST_WORKGROUP);

  if (index >= min(trace_request_head[0], arrayLength(&trace_requests))) {
    return;
  }

  let request: TraceRequest = trace_requests[index];
  let toward: vec3<f32> = normalize(octahedral_decode(unpack2x16snorm(request.toward)));
  let cone: SceneCone = SceneCone(request.cone.x, request.cone.y);
  let hit: SceneHit = trace_scene_ray(request.origin.xyz, toward, 0.0, request.origin.w, cone);

  if (hit.distance < 0.0) {
    return;
  }

  let shown: vec3<f32> = shade_scene_hit(hit, toward, cone, lighting, material_lut, lut_sampler, sky_environment_0,
    sky_environment_1, sky_clamp);
  let fog: f32 = path_fog(lighting, hit.distance);
  let screen: vec4<f32> = vec4<f32>(unpack2x16float(request.screen.x), unpack2x16float(request.screen.y));
  let pixel: vec2<u32> = vec2<u32>(request.pixel & 0xffffu, request.pixel >> 16u);

  if ((request.hit_end & TRACE_REQUEST_WATER) != 0u) {
    let met: vec3<f32> = tonemap(mix(shown, lighting.fog_color.rgb, fog), frame_scale(lighting, exposure));
    let origin: vec3<f32> = (camera.view * vec4<f32>(request.origin.xyz, 1.0)).xyz;
    let along: f32 = -origin.z * (1.0 + hit.distance / max(length(origin), 1e-4));

    textureStore(traced_out, pixel, vec4<f32>(screen.rgb + met * (1.0 - screen.w),
      select(along, request.screen_length, request.screen_length > 0.0)));

    return;
  }

  let met: vec3<f32> = mix(shown * min(1.0, RADIANCE_LIMIT / luminance(shown)), lighting.fog_color.rgb, fog);

  textureStore(traced_out, pixel, vec4<f32>(screen.rgb + met * (1.0 - screen.w),
    select(hit.distance, request.screen_length, screen.w > 0.5)));
  textureStore(ends_out, pixel, vec4<f32>(reflection_end_value(request.hit_end), 0.0, 0.0, 0.0));
}

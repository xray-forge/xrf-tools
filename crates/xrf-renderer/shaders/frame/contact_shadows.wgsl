#import "common/camera"
#import "common/octahedral"
#import "common/fullscreen"

// Contact shadows: from each drawn pixel a ray towards the sun, marched over the frame's depth on the screen with its
// reciprocal distance interpolated, as McGuire and Mara's "Efficient GPU Screen-Space Ray Tracing" (JCGT 3(4), 2014)
// marches; a read hides the sun where it shows something in front of the ray, but by no more than the thickness, and
// in front of the plane the depth around the pixel lies in, so a surface never shadows itself. The steps start at
// interleaved gradient noise (Jimenez, "Next Generation Post Processing in Call of Duty: Advanced Warfare", SIGGRAPH
// 2014), turned each frame a temporal resolve gathers. The sunlight kept, in red.

#import "generated/frame/contact_shadows"

// Metres in front of the surface's own plane a read must stand to hide it, so the plane's own pixels never do.
const PLANE_MARGIN: f32 = 0.02;
// What the plane's margin grows by each metre from the camera, as the depth spreads.
const PLANE_GROWTH: f32 = 0.004;
// What the thickness grows by each metre from the camera, as a pixel spans more of what stands there.
const THICKNESS_GROWTH: f32 = 0.05;
// Pixels' widths at the surface that the ray starts off it, along its normal.
const NORMAL_OFFSET: f32 = 1.0;
// The share of the way to the near plane a ray towards the camera may go.
const NEAR_SHARE: f32 = 0.9;
// The plane's distance where a view ray runs along it or away from it.
const NO_PLANE: f32 = 1e9;

// Interleaved gradient noise: one value a pixel, from zero to one.
fn noise(pixel: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
}

// The distance along the view of the point a depth shows, from the perspective's depth terms.
fn view_distance(depth: f32) -> f32 {
  return camera.projection[3][2] / (depth + camera.projection[2][2]);
}

// A view space point as the drawn pixels place it, `y` down, with its distance along the view.
fn to_screen(position: vec3<f32>) -> vec3<f32> {
  let clip: vec4<f32> = camera.projection * vec4<f32>(position, 1.0);
  let ndc: vec2<f32> = clip.xy / clip.w;

  return vec3<f32>((ndc * vec2<f32>(0.5, -0.5) + 0.5) * camera.viewport.xy, clip.w);
}

// The view ray under a drawn pixel, as the point it passes a metre along the view.
fn view_ray(pixel: vec2<f32>) -> vec3<f32> {
  let size: vec2<f32> = camera.viewport.xy;
  let ndc: vec2<f32> = vec2<f32>(pixel.x / size.x * 2.0 - 1.0, 1.0 - pixel.y / size.y * 2.0);

  return vec3<f32>(
    (ndc.x + camera.projection[2][0]) / camera.projection[0][0],
    (ndc.y + camera.projection[2][1]) / camera.projection[1][1],
    -1.0
  );
}

// The view space point a neighbouring texel shows, or one far behind where nothing is drawn.
fn neighbour_position(texel: vec2<i32>, pixel: vec2<f32>) -> vec3<f32> {
  let depth: f32 = textureLoad(depth_target, texel, 0);

  return select(vec3<f32>(0.0, 0.0, -NO_PLANE), camera_view_position(pixel, depth), depth > 0.0);
}

// The normal of the plane a pixel's depth lies in, facing the eye: across and down, each to whichever neighbour lies
// nearer along the view, so an edge does not bend it.
fn depth_normal(texel: vec2<i32>, pixel: vec2<f32>, position: vec3<f32>) -> vec3<f32> {
  let last: vec2<i32> = vec2<i32>(camera.viewport.xy) - 1;
  let left: vec3<f32> = neighbour_position(max(texel - vec2<i32>(1, 0), vec2<i32>(0)), pixel - vec2<f32>(1.0, 0.0));
  let right: vec3<f32> = neighbour_position(min(texel + vec2<i32>(1, 0), last), pixel + vec2<f32>(1.0, 0.0));
  let up: vec3<f32> = neighbour_position(max(texel - vec2<i32>(0, 1), vec2<i32>(0)), pixel - vec2<f32>(0.0, 1.0));
  let down: vec3<f32> = neighbour_position(min(texel + vec2<i32>(0, 1), last), pixel + vec2<f32>(0.0, 1.0));
  let is_right: bool = abs(right.z - position.z) < abs(position.z - left.z);
  let is_down: bool = abs(down.z - position.z) < abs(position.z - up.z);
  let across: vec3<f32> = select(position - left, right - position, is_right);
  let downward: vec3<f32> = select(position - up, down - position, is_down);
  let normal: vec3<f32> = normalize(cross(downward, across));

  return select(-normal, normal, dot(normal, position) < 0.0);
}

@fragment
fn fs_contact_shadows(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (depth <= 0.0) {
    return vec4<f32>(1.0);
  }

  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, texel, 0).xy);
  let to_sun: vec3<f32> = contact.to_sun.xyz;

  // A surface facing away is unlit by the sun already.
  if (dot(normal, to_sun) <= 0.0) {
    return vec4<f32>(1.0);
  }

  let size: vec2<f32> = camera.viewport.xy;
  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  // The G-buffer's normal is bumped; the plane the surface's own pixels lie in is the depth's.
  let surface_normal: vec3<f32> = depth_normal(texel, in.clip.xy, position);
  let eye_distance: f32 = -position.z;
  let spread: f32 = 2.0 / (camera.projection[1][1] * size.y);
  let origin: vec3<f32> = position + surface_normal * (eye_distance * spread * NORMAL_OFFSET);
  let near: f32 = camera.projection[3][2] / (1.0 + camera.projection[2][2]);
  var ray_length: f32 = contact.length;

  // A ray towards the camera ends in front of the near plane.
  if (to_sun.z > 0.0) {
    ray_length = min(ray_length, (-near - origin.z) * NEAR_SHARE / to_sun.z);
  }

  let start: vec3<f32> = to_screen(origin);
  let end: vec3<f32> = to_screen(origin + to_sun * max(ray_length, 0.0));
  let span: f32 = distance(start.xy, end.xy);

  // Under a pixel long, the march tells nothing apart.
  if (span < 1.0) {
    return vec4<f32>(1.0);
  }

  // Held to the reach along the screen, the reciprocal distance moved back with it.
  let held: f32 = min(1.0, contact.reach / span);
  let last: vec2<f32> = mix(start.xy, end.xy, held);
  let first_inverse: f32 = 1.0 / start.z;
  let last_inverse: f32 = mix(first_inverse, 1.0 / end.z, held);
  // The surface's own plane, met by each read's view ray: a view ray's facing of it changes evenly over the screen.
  let plane: f32 = dot(position, surface_normal);
  let first_facing: f32 = dot(view_ray(start.xy), surface_normal);
  let last_facing: f32 = dot(view_ray(last), surface_normal);
  let jitter: f32 = noise(in.clip.xy + contact.noise);
  let steps: u32 = contact.steps;
  var occlusion: f32 = 0.0;

  for (var step: u32 = 0u; step < steps; step++) {
    let along: f32 = (f32(step) + jitter) / f32(steps);
    let pixel: vec2<f32> = mix(start.xy, last, along);

    if (any(pixel < vec2<f32>(0.0)) || any(pixel >= size)) {
      break;
    }

    let read: f32 = textureLoad(depth_target, vec2<i32>(pixel), 0);

    // Nothing drawn there hides nothing.
    if (read <= 0.0) {
      continue;
    }

    let scene: f32 = view_distance(read);
    let facing: f32 = mix(first_facing, last_facing, along);
    let surface: f32 = select(NO_PLANE, plane / facing, facing < -1e-4);

    // The surface's own plane, or behind it.
    if (scene > surface - (PLANE_MARGIN + scene * PLANE_GROWTH)) {
      continue;
    }

    let ray: f32 = 1.0 / mix(first_inverse, last_inverse, along);
    let in_front: f32 = ray - scene;

    // The first hit is the strongest: weaker towards the ray's end, so the shadow fades out where the ray does.
    if (in_front > 0.0 && in_front < contact.thickness * (1.0 + scene * THICKNESS_GROWTH)) {
      occlusion = 1.0 - along * along;

      break;
    }
  }

  // Faded in over its second pixel of span, so the march giving out far away does not show as a line.
  let resolved: f32 = saturate(span * held - 1.0);

  return vec4<f32>(1.0 - contact.intensity * occlusion * resolved);
}

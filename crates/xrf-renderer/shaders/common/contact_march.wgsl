#import "common/camera"
#import "generated/structs"

// Contact shadows' march, as the sun's contact pass and the local lights take it: from a drawn pixel a ray towards a
// light, marched over the frame's depth on the screen with its reciprocal distance interpolated, as McGuire and Mara's
// "Efficient GPU Screen-Space Ray Tracing" (JCGT 3(4), 2014) marches; a read hides the light where it shows something
// in front of the ray, but by no more than the thickness, and in front of the plane the depth around the pixel lies in,
// so a surface never shadows itself. The steps start at interleaved gradient noise (Jimenez, "Next Generation Post
// Processing in Call of Duty: Advanced Warfare", SIGGRAPH 2014), turned each frame a temporal resolve gathers. The
// depth is passed in, so each pass binds it where it likes.

// Metres in front of the surface's own plane a read must stand to hide it, so the plane's own pixels never do.
const CONTACT_PLANE_MARGIN: f32 = 0.02;
// What the plane's margin grows by each metre from the camera, as the depth spreads.
const CONTACT_PLANE_GROWTH: f32 = 0.004;
// What the thickness grows by each metre from the camera, as a pixel spans more of what stands there.
const CONTACT_THICKNESS_GROWTH: f32 = 0.05;
// Pixels' widths at the surface that the ray starts off it, along its normal.
const CONTACT_NORMAL_OFFSET: f32 = 1.0;
// The share of the way to the near plane a ray towards the camera may go.
const CONTACT_NEAR_SHARE: f32 = 0.9;
// The plane's distance where a view ray runs along it or away from it.
const CONTACT_NO_PLANE: f32 = 1e9;

// Interleaved gradient noise: one value a pixel, from zero to one.
fn contact_noise(pixel: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
}

// The distance along the view of the point a depth shows, from the perspective's depth terms.
fn contact_view_distance(depth: f32) -> f32 {
  return camera.projection[3][2] / (depth + camera.projection[2][2]);
}

// A view space point as the drawn pixels place it, `y` down, with its distance along the view.
fn contact_to_screen(position: vec3<f32>) -> vec3<f32> {
  let clip: vec4<f32> = camera.projection * vec4<f32>(position, 1.0);
  let ndc: vec2<f32> = clip.xy / clip.w;

  return vec3<f32>((ndc * vec2<f32>(0.5, -0.5) + 0.5) * camera.viewport.xy, clip.w);
}

// The view ray under a drawn pixel, as the point it passes a metre along the view.
fn contact_view_ray(pixel: vec2<f32>) -> vec3<f32> {
  let size: vec2<f32> = camera.viewport.xy;
  let ndc: vec2<f32> = vec2<f32>(pixel.x / size.x * 2.0 - 1.0, 1.0 - pixel.y / size.y * 2.0);

  return vec3<f32>(
    (ndc.x + camera.projection[2][0]) / camera.projection[0][0],
    (ndc.y + camera.projection[2][1]) / camera.projection[1][1],
    -1.0
  );
}

// The view space point a neighbouring texel shows, or one far behind where nothing is drawn.
fn contact_neighbour(depth_target: texture_depth_2d, texel: vec2<i32>, pixel: vec2<f32>) -> vec3<f32> {
  let depth: f32 = textureLoad(depth_target, texel, 0);

  return select(vec3<f32>(0.0, 0.0, -CONTACT_NO_PLANE), camera_view_position(pixel, depth), depth > 0.0);
}

// The normal of the plane a pixel's depth lies in, facing the eye: across and down, each to whichever neighbour lies
// nearer along the view, so an edge does not bend it. The G-buffer's normal is bumped; this is the surface's own.
fn contact_plane_normal(
  depth_target: texture_depth_2d,
  texel: vec2<i32>,
  pixel: vec2<f32>,
  position: vec3<f32>
) -> vec3<f32> {
  let last: vec2<i32> = vec2<i32>(camera.viewport.xy) - 1;
  let x: vec2<i32> = vec2<i32>(1, 0);
  let y: vec2<i32> = vec2<i32>(0, 1);
  let left: vec3<f32> = contact_neighbour(depth_target, max(texel - x, vec2<i32>(0)), pixel - vec2<f32>(x));
  let right: vec3<f32> = contact_neighbour(depth_target, min(texel + x, last), pixel + vec2<f32>(x));
  let up: vec3<f32> = contact_neighbour(depth_target, max(texel - y, vec2<i32>(0)), pixel - vec2<f32>(y));
  let down: vec3<f32> = contact_neighbour(depth_target, min(texel + y, last), pixel + vec2<f32>(y));
  let is_right: bool = abs(right.z - position.z) < abs(position.z - left.z);
  let is_down: bool = abs(down.z - position.z) < abs(position.z - up.z);
  let across: vec3<f32> = select(position - left, right - position, is_right);
  let downward: vec3<f32> = select(position - up, down - position, is_down);
  let normal: vec3<f32> = normalize(cross(downward, across));

  return select(-normal, normal, dot(normal, position) < 0.0);
}

// How much of a light the point a pixel shows keeps past what stands within `ray_length` metres of it along
// `direction`, in view space and of unit length, from none to all. `plane_normal` is `contact_plane_normal`'s.
// `falloff` weakens a hit by how far behind what the depth shows the ray passes: zero counts every hit inside the
// thickness whole, one counts the front half of the thickness whole and fades the back half out to none, as fewer
// occluders are that thick. `step_pixels` spaces the reads that many drawn pixels apart, as few as the span needs and
// no more than the settings' steps; zero takes the settings' steps whatever the span.
fn contact_lit(
  depth_target: texture_depth_2d,
  contact: ContactShadows,
  pixel: vec2<f32>,
  position: vec3<f32>,
  plane_normal: vec3<f32>,
  direction: vec3<f32>,
  ray_length: f32,
  falloff: f32,
  step_pixels: f32
) -> f32 {
  let size: vec2<f32> = camera.viewport.xy;
  let eye_distance: f32 = -position.z;
  let spread: f32 = 2.0 / (camera.projection[1][1] * size.y);
  let origin: vec3<f32> = position + plane_normal * (eye_distance * spread * CONTACT_NORMAL_OFFSET);
  let near: f32 = camera.projection[3][2] / (1.0 + camera.projection[2][2]);
  var reached: f32 = ray_length;

  // A ray towards the camera ends in front of the near plane.
  if (direction.z > 0.0) {
    reached = min(reached, (-near - origin.z) * CONTACT_NEAR_SHARE / direction.z);
  }

  let start: vec3<f32> = contact_to_screen(origin);
  let end: vec3<f32> = contact_to_screen(origin + direction * max(reached, 0.0));
  let span: f32 = distance(start.xy, end.xy);

  // Under a pixel long, the march tells nothing apart.
  if (span < 1.0) {
    return 1.0;
  }

  // Held to the reach along the screen, the reciprocal distance moved back with it.
  let held: f32 = min(1.0, contact.reach / span);
  let last: vec2<f32> = mix(start.xy, end.xy, held);
  let first_inverse: f32 = 1.0 / start.z;
  let last_inverse: f32 = mix(first_inverse, 1.0 / end.z, held);
  // The surface's own plane, met by each read's view ray: a view ray's facing of it changes evenly over the screen.
  let plane: f32 = dot(position, plane_normal);
  let first_facing: f32 = dot(contact_view_ray(start.xy), plane_normal);
  let last_facing: f32 = dot(contact_view_ray(last), plane_normal);
  let jitter: f32 = contact_noise(pixel + contact.noise);
  let spaced: u32 = u32(ceil(span * held / max(step_pixels, 1e-3)));
  let steps: u32 = select(contact.steps, clamp(spaced, 2u, contact.steps), step_pixels > 0.0);
  var occlusion: f32 = 0.0;

  for (var step: u32 = 0u; step < steps; step++) {
    let along: f32 = (f32(step) + jitter) / f32(steps);
    let at: vec2<f32> = mix(start.xy, last, along);

    if (any(at < vec2<f32>(0.0)) || any(at >= size)) {
      break;
    }

    let read: f32 = textureLoad(depth_target, vec2<i32>(at), 0);

    // Nothing drawn there hides nothing.
    if (read <= 0.0) {
      continue;
    }

    let scene: f32 = contact_view_distance(read);
    let facing: f32 = mix(first_facing, last_facing, along);
    let surface: f32 = select(CONTACT_NO_PLANE, plane / facing, facing < -1e-4);

    // The surface's own plane, or behind it.
    if (scene > surface - (CONTACT_PLANE_MARGIN + scene * CONTACT_PLANE_GROWTH)) {
      continue;
    }

    let ray: f32 = 1.0 / mix(first_inverse, last_inverse, along);
    let in_front: f32 = ray - scene;
    let thickness: f32 = contact.thickness * (1.0 + scene * CONTACT_THICKNESS_GROWTH);

    // Weaker towards the ray's end, so the shadow fades out where the ray does, and by the falloff the further behind
    // what the depth shows, as fewer occluders are that thick.
    if (in_front > 0.0 && in_front < thickness) {
      let inside: f32 = 1.0 - falloff * smoothstep(0.5, 1.0, in_front / thickness);

      occlusion = max(occlusion, (1.0 - along * along) * inside);

      // No read further along can take more: without a falloff, the first hit decides.
      let next: f32 = (f32(step) + 1.0 + jitter) / f32(steps);

      if (occlusion >= 1.0 - next * next) {
        break;
      }
    }
  }

  // Faded in over its second pixel of span, so the march giving out far away does not show as a line.
  let resolved: f32 = saturate(span * held - 1.0);

  return 1.0 - contact.intensity * occlusion * resolved;
}

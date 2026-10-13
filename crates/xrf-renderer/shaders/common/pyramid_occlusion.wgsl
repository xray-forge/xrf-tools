// Occlusion against a depth pyramid of the farthest depth (reversed: the least value) under each texel, each level half
// the one before.

// Whether a sphere lies wholly behind the depth `pyramid` holds, seen through a view: its screen rectangle is read at
// the level where it spans at most two texels a side, and its nearest point compared with the farthest depth there.
// `size` is the pyramid's first level's, `levels` how many it has.
fn is_sphere_occluded(
  pyramid: texture_2d<f32>,
  size: vec2<f32>,
  levels: u32,
  sphere: vec4<f32>,
  view: mat4x4<f32>,
  projection: mat4x4<f32>,
) -> bool {
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

  let span: vec2<f32> = (high - low) * size;
  let level: u32 = min(u32(max(ceil(log2(max(span.x, span.y))), 0.0)), levels - 1u);
  let extent: vec2<u32> = textureDimensions(pyramid, level);
  let first: vec2<u32> = min(vec2<u32>(low * vec2<f32>(extent)), extent - 1u);
  let last: vec2<u32> = min(vec2<u32>(high * vec2<f32>(extent)), extent - 1u);
  let farthest: f32 = min(
    min(textureLoad(pyramid, first, level).x, textureLoad(pyramid, vec2<u32>(last.x, first.y), level).x),
    min(textureLoad(pyramid, vec2<u32>(first.x, last.y), level).x, textureLoad(pyramid, last, level).x)
  );
  let nearest: vec4<f32> = projection * vec4<f32>(0.0, 0.0, center.z + radius, 1.0);

  return nearest.z / nearest.w < farthest;
}

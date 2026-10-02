#import "common/camera"

// The ground plane at height zero, ruled every metre and every ten, under a plain sky: what an empty viewport shows
// so that moving its camera can be seen.

struct GridVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) ndc: vec2<f32>,
};

const SKY_HORIZON: vec3<f32> = vec3<f32>(0.32, 0.34, 0.37);
const SKY_ZENITH: vec3<f32> = vec3<f32>(0.13, 0.15, 0.19);
const GROUND: vec3<f32> = vec3<f32>(0.16, 0.16, 0.17);
const MINOR: vec3<f32> = vec3<f32>(0.26, 0.26, 0.27);
const MAJOR: vec3<f32> = vec3<f32>(0.38, 0.38, 0.40);
const AXIS_X: vec3<f32> = vec3<f32>(0.75, 0.25, 0.25);
const AXIS_Z: vec3<f32> = vec3<f32>(0.25, 0.40, 0.80);

// One triangle covering the viewport.
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> GridVarying {
  let uv: vec2<f32> = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
  let ndc: vec2<f32> = uv * 2.0 - 1.0;
  var out: GridVarying;

  out.clip = vec4<f32>(ndc, 0.0, 1.0);
  out.ndc = ndc;

  return out;
}

// Coverage of lines every `spacing` metres, a pixel wide wherever they are seen.
fn grid_lines(point: vec2<f32>, spacing: f32) -> f32 {
  let cell: vec2<f32> = point / spacing;
  let width: vec2<f32> = fwidth(cell);
  let distance: vec2<f32> = abs(fract(cell - 0.5) - 0.5) / max(width, vec2<f32>(1e-6));

  return 1.0 - min(min(distance.x, distance.y), 1.0);
}

@fragment
fn fs_main(in: GridVarying) -> @location(0) vec4<f32> {
  let near: vec3<f32> = camera_unproject(in.ndc, 1.0);
  let far: vec3<f32> = camera_unproject(in.ndc, 0.5);
  let direction: vec3<f32> = normalize(far - near);
  let eye: vec3<f32> = camera.position.xyz;
  let sky: vec3<f32> = mix(SKY_HORIZON, SKY_ZENITH, sqrt(clamp(direction.y, 0.0, 1.0)));

  // Above the plane looking up, or below it looking down, there is no ground in view.
  if (direction.y * eye.y >= 0.0) {
    return vec4<f32>(sky, 1.0);
  }

  let along: f32 = -eye.y / direction.y;
  let ground: vec2<f32> = (eye + direction * along).xz;
  // Lines fade into the sky with distance, as height above the plane sets how far detail holds.
  let fade: f32 = clamp(1.0 - along / (200.0 + abs(eye.y) * 40.0), 0.0, 1.0);

  var color: vec3<f32> = GROUND;

  color = mix(color, MINOR, grid_lines(ground, 1.0) * fade * 0.6);
  color = mix(color, MAJOR, grid_lines(ground, 10.0) * fade);

  let axis_width: vec2<f32> = fwidth(ground) * 1.5;

  color = mix(color, AXIS_X, step(abs(ground.y), axis_width.y) * fade);
  color = mix(color, AXIS_Z, step(abs(ground.x), axis_width.x) * fade);

  // The ground itself melts into the sky at the horizon, rather than meeting it at a hard edge.
  return vec4<f32>(mix(sky, color, sqrt(fade)), 1.0);
}

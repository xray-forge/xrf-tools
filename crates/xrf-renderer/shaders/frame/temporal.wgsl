#import "common/camera"
#import "common/fullscreen"

// The temporal resolve, after Karis's "High Quality Temporal Supersampling": this frame's jittered samples blended
// with the history the camera's motion carries to each pixel, the history clipped to the colours around it, upscaling
// as TAAU where the frame is drawn smaller than the history. Motion is the camera's, read from depth.

struct Temporal {
  // This frame's view projection without its jitter, and the last frame's.
  current: mat4x4<f32>,
  previous: mat4x4<f32>,
  // The last frame's view, which the history's distances were measured along.
  previous_view: mat4x4<f32>,
  // xy: this frame's jitter in drawn pixels, y down; z: one while the history holds a frame; w: the least share of
  // this frame a pixel takes.
  params: vec4<f32>,
};

@group(1) @binding(0) var frame: texture_2d<f32>;
@group(1) @binding(1) var depth_target: texture_depth_2d;
// Colour, then distance along the view, zero where nothing was drawn.
@group(1) @binding(2) var history: texture_2d<f32>;
@group(1) @binding(3) var history_sampler: sampler;
@group(1) @binding(4) var<uniform> temporal: Temporal;

// Share of a point's distance its history's may differ by and still be taken as the same surface.
const DISTANCE_TOLERANCE: f32 = 0.1;
// Output pixels of motion at which the history counts for nothing more than the neighbourhood lets it.
const MAX_MOTION: f32 = 128.0;
// `exp(-2.29 x²)`'s factor: Karis's fit of a Blackman-Harris window a pixel wide.
const WINDOW: f32 = -2.29;
const NEIGHBOURS: f32 = 9.0;

fn load_frame(at: vec2<f32>, size: vec2<f32>) -> vec3<f32> {
  return max(textureLoad(frame, vec2<i32>(clamp(at, vec2<f32>(0.0), size - 1.0)), 0).rgb, vec3<f32>(0.0));
}

fn load_depth(at: vec2<f32>, size: vec2<f32>) -> f32 {
  return textureLoad(depth_target, vec2<i32>(clamp(at, vec2<f32>(0.0), size - 1.0)), 0);
}

// The world point a drawn texel shows at a depth.
fn world_at(texel: vec2<f32>, depth: f32) -> vec3<f32> {
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));

  return rotation * camera_view_position(texel + 0.5, depth) + camera.position.xyz;
}

// Where a world point falls on the screen through a view projection, as a texture coordinate.
fn to_uv(view_projection: mat4x4<f32>, world: vec3<f32>) -> vec2<f32> {
  let clip: vec4<f32> = view_projection * vec4<f32>(world, 1.0);
  let ndc: vec2<f32> = clip.xy / clip.w;

  return vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}

fn luminance(color: vec3<f32>) -> f32 {
  return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// The history clipped towards the neighbourhood's mean onto the box of its colours (Playdead's `clip_aabb`), so a
// colour the neighbourhood no longer has fades out rather than trails.
fn clipped(color: vec3<f32>, mean: vec3<f32>, minimum: vec3<f32>, maximum: vec3<f32>) -> vec3<f32> {
  let center: vec3<f32> = clamp(mean, minimum, maximum);
  let extent: vec3<f32> = (maximum - minimum) * 0.5 + 1e-7;
  let offset: vec3<f32> = color - center;
  let units: vec3<f32> = abs(offset / extent);
  let reach: f32 = max(units.x, max(units.y, units.z));

  return select(color, center + offset / reach, reach > 1.0);
}

// The history at a point, Catmull-Rom filtered in five bilinear fetches: a bilinear one blurs a moving view a little
// more every frame it is resampled.
fn catmull_rom(at: vec2<f32>, size: vec2<f32>) -> vec3<f32> {
  let position: vec2<f32> = at * size;
  let center: vec2<f32> = floor(position - 0.5) + 0.5;
  let f: vec2<f32> = position - center;
  let w0: vec2<f32> = f * (f * (f * -0.5 + 1.0) - 0.5);
  let w1: vec2<f32> = f * f * (f * 1.5 - 2.5) + 1.0;
  let w2: vec2<f32> = f * (f * (f * -1.5 + 2.0) + 0.5);
  let w3: vec2<f32> = f * f * (f * 0.5 - 0.5);
  let w12: vec2<f32> = w1 + w2;
  let at0: vec2<f32> = (center - 1.0) / size;
  let at3: vec2<f32> = (center + 2.0) / size;
  let at12: vec2<f32> = (center + w2 / w12) / size;
  var total: vec4<f32> = vec4<f32>(0.0);

  total += vec4<f32>(textureSampleLevel(history, history_sampler, vec2<f32>(at12.x, at0.y), 0.0).rgb, 1.0)
    * (w12.x * w0.y);
  total += vec4<f32>(textureSampleLevel(history, history_sampler, vec2<f32>(at0.x, at12.y), 0.0).rgb, 1.0)
    * (w0.x * w12.y);
  total += vec4<f32>(textureSampleLevel(history, history_sampler, at12, 0.0).rgb, 1.0) * (w12.x * w12.y);
  total += vec4<f32>(textureSampleLevel(history, history_sampler, vec2<f32>(at3.x, at12.y), 0.0).rgb, 1.0)
    * (w3.x * w12.y);
  total += vec4<f32>(textureSampleLevel(history, history_sampler, vec2<f32>(at12.x, at3.y), 0.0).rgb, 1.0)
    * (w12.x * w3.y);

  return max(total.rgb / max(total.w, 1e-5), vec3<f32>(0.0));
}

// This frame and the history blended, each weighed down by its brightness so a bright sample does not flicker.
fn blended(current: vec3<f32>, previous: vec3<f32>, weight: f32) -> vec3<f32> {
  let current_weight: f32 = weight / (luminance(current) + 1.0);
  let previous_weight: f32 = (1.0 - weight) / (luminance(previous) + 1.0);

  return (current * current_weight + previous * previous_weight) / max(current_weight + previous_weight, 1e-5);
}

@fragment
fn fs_temporal(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let input_size: vec2<f32> = vec2<f32>(textureDimensions(frame));
  let output_size: vec2<f32> = vec2<f32>(textureDimensions(history));
  // Output pixels a drawn one spans across; one unscaled.
  let upscale: f32 = output_size.x / input_size.x;
  let jitter: vec2<f32> = temporal.params.xy;
  let uv: vec2<f32> = in.clip.xy / output_size;
  let position: vec2<f32> = uv * input_size;
  // The drawn texel whose jittered sample lies nearest this pixel's centre.
  let nearest: vec2<f32> = floor(position - jitter);

  // The nearest surface around it, whose motion carries an edge's history with the edge rather than its background.
  var closest_depth: f32 = 0.0;
  var closest: vec2<f32> = nearest;

  for (var index: i32 = 0; index < 9; index++) {
    let at: vec2<f32> = nearest + vec2<f32>(f32(index % 3 - 1), f32(index / 3 - 1));
    let depth: f32 = load_depth(at, input_size);

    if (depth > closest_depth) {
      closest_depth = depth;
      closest = at;
    }
  }

  let depth: f32 = load_depth(nearest, input_size);
  let is_drawn: bool = depth > 0.0;
  let world: vec3<f32> = world_at(nearest, depth);
  let distance: f32 = -camera_view_position(nearest + 0.5, depth).z;
  let moving: vec3<f32> = world_at(closest, closest_depth);
  let moved: vec2<f32> = to_uv(temporal.current, moving) - to_uv(temporal.previous, moving);
  let before: vec2<f32> = uv - moved;

  // The history stands for this surface where it showed something as far from the camera as this point was then.
  let is_inside: bool = all(before >= vec2<f32>(0.0)) && all(before <= vec2<f32>(1.0));
  let history_texel: vec2<i32> = vec2<i32>(clamp(floor(before * output_size), vec2<f32>(0.0), output_size - 1.0));
  let history_distance: f32 = textureLoad(history, history_texel, 0).a;
  let expected: f32 = -(temporal.previous_view * vec4<f32>(world, 1.0)).z;
  let is_same_surface: bool = select(history_distance <= 0.0,
    abs(history_distance - expected) <= expected * DISTANCE_TOLERANCE, is_drawn);
  let is_valid: bool = temporal.params.z > 0.5 && is_inside && is_same_surface;

  // This frame's colour where the pixel's centre is, and the spread of colours around it.
  var sum: vec3<f32> = vec3<f32>(0.0);
  var weights: f32 = 0.0;
  var moment: vec3<f32> = vec3<f32>(0.0);
  var moment_squared: vec3<f32> = vec3<f32>(0.0);

  for (var index: i32 = 0; index < 9; index++) {
    let at: vec2<f32> = nearest + vec2<f32>(f32(index % 3 - 1), f32(index / 3 - 1));
    let color: vec3<f32> = load_frame(at, input_size);
    let offset: vec2<f32> = position - (at + 0.5 + jitter);
    let weight: f32 = exp(dot(offset, offset) * WINDOW);

    sum += color * weight;
    weights += weight;
    moment += color;
    moment_squared += color * color;
  }

  let current: vec3<f32> = sum / max(weights, 1e-5);
  let mean: vec3<f32> = moment / NEIGHBOURS;
  let motion_share: f32 = saturate(length(moved * output_size) / MAX_MOTION);
  // Wider while still, so a sub-pixel edge keeps its history; tighter while moving, so nothing trails.
  let still: f32 = (1.0 - motion_share) * (1.0 - motion_share);
  let spread: vec3<f32> = sqrt(max(moment_squared / NEIGHBOURS - mean * mean, vec3<f32>(0.0))) * mix(0.5, 1.0, still);
  let previous: vec3<f32> = clipped(catmull_rom(before, output_size), mean, mean - spread, mean + spread);
  // The nearest sample's distance from this pixel's centre, in output pixels, weighs this frame; unscaled, as is.
  let landed: vec2<f32> = (position - (nearest + 0.5 + jitter)) * upscale;
  let confidence: f32 = select(1.0, exp(dot(landed, landed) * WINDOW) * upscale * upscale, upscale > 1.001);
  let weight: f32 = select(1.0, saturate(temporal.params.w * confidence + motion_share), is_valid);

  return vec4<f32>(blended(current, previous, weight), select(0.0, distance, is_drawn));
}

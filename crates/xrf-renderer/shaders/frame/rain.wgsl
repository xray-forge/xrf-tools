#import "common/camera"
#import "common/rain_cover"

// The rain `dxRainRender` draws, over the finished scene and tested against its depth: every streak a quad and every
// splash a copy of its model, each placed by its streak's fall, which is `Born` and `RenewItem` made stateless. Each
// streak falls again and again at its own speed from a column of the level the camera's square wraps as it moves, and
// stops where the cover under that column stands, which is where its splash is.

struct Rain {
  // `rain_color`, then the streaks' cover, `factor / 2 + .5`.
  color: vec4<f32>,
  // The way the rain falls, in renderer space, before each streak strays from it.
  axis: vec4<f32>,
  // The cover's centre in `x` and `z`, its half width, and the height it is seen from.
  window: vec4<f32>,
  // Streaks drawn, `0.5 * (1 + factor) * max_desired_items`.
  count: u32,
  // Seconds the rain has fallen.
  time: f32,
  // Indices the splash's model draws.
  splash_indices: u32,
  pad: u32,
};

@group(1) @binding(0) var<uniform> rain: Rain;
@group(1) @binding(1) var cover: texture_depth_2d;
@group(1) @binding(2) var streak_texture: texture_2d<f32>;
@group(1) @binding(3) var splash_texture: texture_2d<f32>;
@group(1) @binding(4) var rain_sampler: sampler;
// The splash's model in renderer space: its position, then its coordinate, a vertex.
@group(1) @binding(5) var<storage, read> splash_vertices: array<vec4<f32>>;
@group(1) @binding(6) var<storage, read> splash_indices: array<u32>;

const RAIN_PI: f32 = 3.14159265;

// `source_offset`: metres above the camera a streak starts, and `max_distance`, metres it falls at most.
const SOURCE_OFFSET: f32 = 40.0;
const MAX_DISTANCE: f32 = SOURCE_OFFSET * 1.25;

// `source_radius`: how far around the camera streaks start, as a square of the disc's area.
const SOURCE_TILE: f32 = 12.5 * 1.7724539;

// `drop_speed_min` and `drop_speed_max`, in metres a second.
const SPEED_MIN: f32 = 40.0;
const SPEED_MAX: f32 = 80.0;

// `cos(drop_angle)`: how far a streak strays from the rain's way, three degrees.
const STRAY_COS: f32 = 0.9986295;

// `drop_length`: metres a streak is long at its heaviest, and `drop_width`, half its width.
const STREAK_LENGTH: f32 = 5.0;
const STREAK_WIDTH: f32 = 0.3;

// `particles_time`: seconds a splash lasts.
const SPLASH_TIME: f32 = 0.3;

// One streak's quad: its corners across it from minus one to one, then from its tail to its head, as two triangles.
const CORNERS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
  vec2<f32>(-1.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(-1.0, 1.0),
  vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
);

struct Fall {
  // Seconds into this fall, and when it lands, past its period where it never does.
  age: f32,
  landing: f32,
  direction: vec3<f32>,
  head: vec3<f32>,
  landed: vec3<f32>,
  is_falling: bool,
  // Which fall of the streak, which every further draw is seeded by.
  cycle: u32,
};

struct RainVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

// `pcg`: a word hashed into another, every bit of it depending on every bit given.
fn hash(seed: u32) -> u32 {
  let state: u32 = seed * 747796405u + 2891336453u;
  let word: u32 = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;

  return (word >> 22u) ^ word;
}

// A number in `[0, 1]` for a streak, a fall of it and a draw.
fn random(streak: u32, cycle: u32, draw: u32) -> f32 {
  return f32(hash(streak * 0x9e3779b1u + cycle * 0x85ebca6bu + draw)) / 4294967295.0;
}

fn fall_of(streak: u32) -> Fall {
  let speed: f32 = mix(SPEED_MIN, SPEED_MAX, random(streak, 0u, 0u));
  let period: f32 = MAX_DISTANCE / speed;
  let phase: f32 = rain.time / period + random(streak, 0u, 1u);
  let cycle: u32 = u32(floor(phase));
  let age: f32 = fract(phase) * period;
  let eye: vec3<f32> = camera.position.xyz;
  // A column of the level, kept where it is as the camera moves and wrapped around it.
  let base: vec2<f32> = vec2<f32>(random(streak, cycle, 2u), random(streak, cycle, 3u)) * SOURCE_TILE;
  let wrapped: vec2<f32> = base - eye.xz + SOURCE_TILE / 2.0;
  let around: vec2<f32> = wrapped - floor(wrapped / SOURCE_TILE) * SOURCE_TILE - SOURCE_TILE / 2.0;
  let column: vec3<f32> = vec3<f32>(eye.x + around.x, eye.y, eye.z + around.y);
  // `random_dir(axis, drop_angle)`: within the cone about the rain's way.
  let axis: vec3<f32> = rain.axis.xyz;
  let across: vec3<f32> = normalize(cross(axis, vec3<f32>(1.0, 0.0, 0.0)));
  let along: vec3<f32> = cross(axis, across);
  let lean: f32 = mix(1.0, STRAY_COS, random(streak, cycle, 4u));
  let turn: f32 = random(streak, cycle, 5u) * RAIN_PI * 2.0;
  let direction: vec3<f32> = normalize(axis * lean + (across * cos(turn) + along * sin(turn)) * sqrt(1.0 - lean * lean));
  // It starts `source_offset` over the camera, set back along its way so it falls through its column.
  let start: f32 = SOURCE_OFFSET / -direction.y;
  let reach: f32 = (rain_cover_height(cover, rain.window, column) - column.y) / direction.y;
  let landing: f32 = (reach + start) / speed;
  var fall: Fall;

  fall.age = age;
  fall.landing = select(period + 1.0, landing, landing < period);
  fall.direction = direction;
  fall.head = column + direction * (speed * age - start);
  fall.landed = column + direction * reach;
  fall.is_falling = streak < rain.count;
  fall.cycle = cycle;

  return fall;
}

// A streak's quad facing the camera from its tail to its head, collapsed onto its head while it has landed or is not
// drawn; its coordinate one of the engine's two sets, one mirroring the other, picked each fall.
@vertex
fn vs_streak(@builtin(vertex_index) vertex_index: u32) -> RainVarying {
  let streak: u32 = vertex_index / 6u;
  var corners: array<vec2<f32>, 6> = CORNERS;
  let corner: vec2<f32> = corners[vertex_index % 6u];
  let fall: Fall = fall_of(streak);
  let tail: vec3<f32> = fall.head - fall.direction * rain.color.w * STREAK_LENGTH;
  let middle: vec3<f32> = mix(tail, fall.head, 0.5);
  let side: vec3<f32> = cross(normalize(middle - camera.position.xyz), fall.direction);
  let placed: vec3<f32> = mix(tail, fall.head, corner.y) + side * corner.x * STREAK_WIDTH;
  let shown: vec3<f32> = select(fall.head, placed, fall.is_falling && fall.age < fall.landing);
  let first: vec2<f32> = vec2<f32>(corner.y, (1.0 - corner.x) * 0.5);
  var out: RainVarying;

  out.clip = camera.view_projection * vec4<f32>(shown, 1.0);
  out.uv = select(vec2<f32>(1.0) - first, first, random(streak, fall.cycle, 6u) < 0.5);

  return out;
}

// `CEffect_Rain::Hit`: half the drops that land leave a splash, turned at random and shrinking to nothing.
@vertex
fn vs_splash(@builtin(vertex_index) vertex_index: u32) -> RainVarying {
  let streak: u32 = vertex_index / rain.splash_indices;
  let index: u32 = splash_indices[vertex_index % rain.splash_indices];
  let position: vec4<f32> = splash_vertices[index * 2u];
  let coordinate: vec4<f32> = splash_vertices[index * 2u + 1u];
  let fall: Fall = fall_of(streak);
  let since: f32 = fall.age - fall.landing;
  let scale: f32 = 1.0 - since / SPLASH_TIME;
  let turn: f32 = random(streak, fall.cycle, 7u) * RAIN_PI * 2.0;
  let c: f32 = cos(turn);
  let s: f32 = sin(turn);
  let turned: vec3<f32> = vec3<f32>(position.x * c + position.z * s, position.y, position.z * c - position.x * s);
  let is_shown: bool = fall.is_falling && random(streak, fall.cycle, 8u) < 0.5 && since >= 0.0 && since < SPLASH_TIME;
  var out: RainVarying;

  out.clip = camera.view_projection * vec4<f32>(select(fall.landed, fall.landed + turned * scale, is_shown), 1.0);
  out.uv = coordinate.xy;

  return out;
}

// `effects\rain` over `stub_default`: the texture times the rain's colour, by its alpha, unlit and past the tonemap.
fn shade(texel: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(texel.rgb * rain.color.rgb, texel.a * rain.color.a);
}

@fragment
fn fs_streak(in: RainVarying) -> @location(0) vec4<f32> {
  return shade(textureSample(streak_texture, rain_sampler, in.uv));
}

@fragment
fn fs_splash(in: RainVarying) -> @location(0) vec4<f32> {
  return shade(textureSample(splash_texture, rain_sampler, in.uv));
}

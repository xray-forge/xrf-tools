#import "common/camera"
#import "common/scene_light"
#import "common/rain_cover"

// The rain `dxRainRender` draws, over the finished scene and tested against its depth: every streak a quad and every
// splash a copy of its model, each placed by its streak's fall, which is `Born` and `RenewItem` made stateless. Each
// streak falls again and again at its own speed from a column of the level a square about the camera wraps as it
// moves, and stops where the cover under that column stands, which is where its splash is. The fall's shape is the
// uniform's: the engine's, or Monolith's as Screen Space Shaders' rain shapes it (`ssfx_rain_1`, `drops_setup`). The
// enhanced streaks draw after the temporal resolve, over a copy of the image it shows, so no history carries them.

// The rain, its cover, the streak's and the splash's textures, and the splash's model in renderer space (its position,
// then its coordinate, a vertex); for the enhanced rain, the shown image blurred small, their maps and their sampler,
// the depth, the light, the lighting and the exposure.
#import "generated/frame/rain"
#import "common/math"
#import "common/luminance"

// `particles_time`: seconds a splash lasts.
const SPLASH_TIME: f32 = 0.3;

// `effects_rain.ps`: how far a drop bends the blurred scene behind it, in screen widths, before its refraction.
const DROP_BEND: f32 = 0.0125;

// Metres from the eye over which an enhanced drop fades in: one nearer would cover much of the screen as a soft
// pillar, where a real one passes too fast to be seen.
const NEAR_FADE: vec2<f32> = vec2<f32>(0.5, 2.0);

// How strongly a drop scatters the sun forwards, Henyey-Greenstein's asymmetry, and how much of the sun it scatters
// at all: water drops send most of their light on past them, so rain against the sun glows.
const SUN_ASYMMETRY: f32 = 0.5;
const SUN_SCATTER: f32 = 0.1;

// How far the light a drop scatters goes towards its own grey: clear water takes little of the sky's colour, and a
// tinted sky tinted every drop with it.
const SCATTER_GREY: f32 = 0.75;

// The least of its length and of its width an enhanced streak is drawn at, each fall picking between it and all.
const STREAK_LEAST: vec2<f32> = vec2<f32>(0.6, 0.75);

// How large a splash stands as it lands, of its full size, which it opens to as it fades.
const SPLASH_OPENING: f32 = 0.6;

// What the distortion target holds where nothing distorts it, `ViewTargets::NEUTRAL_DISTORTION`.
const NEUTRAL_DISTORTION: f32 = 127.0 / 255.0;

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
  // Where the point stands in view space, which the light it scatters towards the eye is measured by.
  @location(1) view_at: vec3<f32>,
  // How far through its life a splash is, from nought to one; nought for a streak.
  @location(2) life: f32,
};

// What the enhanced rain draws before the resolve: its colour over the scene, and the distortion under it, by their
// alpha.
struct RainLook {
  @location(0) color: vec4<f32>,
  @location(1) distortion: vec4<f32>,
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

// The way a fall goes, about the rain's way. The engine's `random_dir(axis, drop_angle)` reads as a cone of the
// stray's half angle; Monolith's `random_dir(axis, randF(-drop_angle, drop_angle) * (1.5 - velocity))` adds a random
// direction scaled by up to that angle's tangent, so most drops stray far less than the widest.
fn direction_of(streak: u32, cycle: u32) -> vec3<f32> {
  let axis: vec3<f32> = rain.axis.xyz;
  let turn: f32 = random(streak, cycle, 5u) * TAU;

  if (rain.is_enhanced != 0u) {
    let rise: f32 = random(streak, cycle, 9u) * 2.0 - 1.0;
    let flat: f32 = sqrt(1.0 - rise * rise);
    let scatter: vec3<f32> = vec3<f32>(flat * cos(turn), rise, flat * sin(turn));
    let angle: f32 = (random(streak, cycle, 4u) * 2.0 - 1.0) * rain.axis.w;

    return normalize(axis + scatter * random(streak, cycle, 10u) * tan(angle));
  }

  let across: vec3<f32> = normalize(cross(axis, vec3<f32>(1.0, 0.0, 0.0)));
  let along: vec3<f32> = cross(axis, across);
  let lean: f32 = mix(1.0, cos(rain.axis.w), random(streak, cycle, 4u));

  return normalize(axis * lean + (across * cos(turn) + along * sin(turn)) * sqrt(1.0 - lean * lean));
}

fn fall_of(streak: u32) -> Fall {
  let speed: f32 = mix(rain.streak.z, rain.streak.w, random(streak, 0u, 0u));
  let period: f32 = rain.fall.y / speed;
  let phase: f32 = rain.time / period + random(streak, 0u, 1u);
  let cycle: u32 = u32(floor(phase));
  let age: f32 = fract(phase) * period;
  let eye: vec3<f32> = camera.position.xyz;
  // Monolith's `Born` aims ahead of where the camera looks (`aim`, none for the engine's).
  let centre: vec2<f32> = eye.xz + rain.aim.xy;
  // A column of the level, kept where it is as the camera moves and wrapped around the square's centre.
  let tile: f32 = rain.fall.z;
  let base: vec2<f32> = vec2<f32>(random(streak, cycle, 2u), random(streak, cycle, 3u)) * tile;
  let wrapped: vec2<f32> = base - centre + tile / 2.0;
  let around: vec2<f32> = wrapped - floor(wrapped / tile) * tile - tile / 2.0;
  let column: vec3<f32> = vec3<f32>(centre.x + around.x, eye.y, centre.y + around.y);
  let direction: vec3<f32> = direction_of(streak, cycle);
  // It starts `source_offset` over the camera, set back along its way so it falls through its column.
  let start: f32 = rain.fall.x / -direction.y;
  let reach: f32 = (overhead_height(cover, rain.window, rain.cover_shape.x, rain.cover_shape.y, column) - column.y) /
    direction.y;
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

// A streak's quad facing the camera from its tail to its head through `view_projection`, collapsed onto its head while
// it has landed or is not drawn; its coordinate one of the engine's two sets, one mirroring the other, picked each
// fall; each enhanced fall its own length and width, so the streaks do not read as one set of lines.
fn streak_vertex(vertex_index: u32, view_projection: mat4x4<f32>) -> RainVarying {
  let streak: u32 = vertex_index / 6u;
  var corners: array<vec2<f32>, 6> = CORNERS;
  let corner: vec2<f32> = corners[vertex_index % 6u];
  let fall: Fall = fall_of(streak);
  let picked: vec2<f32> = vec2<f32>(random(streak, fall.cycle, 11u), random(streak, fall.cycle, 12u));
  let size: vec2<f32> = select(vec2<f32>(1.0), mix(STREAK_LEAST, vec2<f32>(1.0), picked), rain.is_enhanced != 0u);
  let tail: vec3<f32> = fall.head - fall.direction * rain.color.w * rain.streak.x * size.x;
  let middle: vec3<f32> = mix(tail, fall.head, 0.5);
  let side: vec3<f32> = cross(normalize(middle - camera.position.xyz), fall.direction);
  let placed: vec3<f32> = mix(tail, fall.head, corner.y) + side * corner.x * rain.streak.y * size.y;
  let shown: vec3<f32> = select(fall.head, placed, fall.is_falling && fall.age < fall.landing);
  let first: vec2<f32> = vec2<f32>(corner.y, (1.0 - corner.x) * 0.5);
  var out: RainVarying;

  out.clip = view_projection * vec4<f32>(shown, 1.0);
  out.uv = select(vec2<f32>(1.0) - first, first, random(streak, fall.cycle, 6u) < 0.5);
  out.view_at = (camera.view * vec4<f32>(shown, 1.0)).xyz;
  out.life = 0.0;

  return out;
}

// The streaks drawn into the scene, through the frame's jittered camera.
@vertex
fn vs_streak(@builtin(vertex_index) vertex_index: u32) -> RainVarying {
  return streak_vertex(vertex_index, camera.view_projection);
}

// The enhanced streaks drawn over the image the temporal resolve shows, through the camera without its jitter.
@vertex
fn vs_streak_over(@builtin(vertex_index) vertex_index: u32) -> RainVarying {
  return streak_vertex(vertex_index, camera.motion_current);
}

// `CEffect_Rain::Hit`: half the drops that land leave a splash (the enhanced, `splash_look.w` of them), turned at
// random and shrinking to nothing; the enhanced splash opens from `SPLASH_OPENING` of its size to all of it instead,
// as it fades.
@vertex
fn vs_splash(@builtin(vertex_index) vertex_index: u32) -> RainVarying {
  let streak: u32 = vertex_index / rain.splash_indices;
  let index: u32 = splash_indices[vertex_index % rain.splash_indices];
  let position: vec4<f32> = splash_vertices[index * 2u];
  let coordinate: vec4<f32> = splash_vertices[index * 2u + 1u];
  let fall: Fall = fall_of(streak);
  let since: f32 = fall.age - fall.landing;
  let life: f32 = saturate(since / SPLASH_TIME);
  let scale: f32 = select(1.0 - life, mix(SPLASH_OPENING, 1.0, life), rain.is_enhanced != 0u);
  let turn: f32 = random(streak, fall.cycle, 7u) * PI * 2.0;
  let c: f32 = cos(turn);
  let s: f32 = sin(turn);
  let turned: vec3<f32> = vec3<f32>(position.x * c + position.z * s, position.y, position.z * c - position.x * s);
  let splashing: f32 = select(0.5, rain.splash_look.w, rain.is_enhanced != 0u);
  let is_shown: bool =
    fall.is_falling && random(streak, fall.cycle, 8u) < splashing && since >= 0.0 && since < SPLASH_TIME;
  let placed: vec3<f32> = select(fall.landed, fall.landed + turned * scale, is_shown);
  var out: RainVarying;

  out.clip = camera.view_projection * vec4<f32>(placed, 1.0);
  out.uv = coordinate.xy;
  out.view_at = (camera.view * vec4<f32>(placed, 1.0)).xyz;
  out.life = life;

  return out;
}

// `effects\rain` over `stub_default`: the texture times the rain's colour, by its alpha, unlit and past the tonemap.
fn shade(texel: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(scene_from_shown(texel.rgb * rain.color.rgb), texel.a * rain.color.a);
}

@fragment
fn fs_streak(in: RainVarying) -> @location(0) vec4<f32> {
  return shade(textureSample(streak_texture, rain_sampler, in.uv));
}

@fragment
fn fs_splash(in: RainVarying) -> @location(0) vec4<f32> {
  return shade(textureSample(splash_texture, rain_sampler, in.uv));
}

// A drawn-size target's texel at a place on the screen, from nought to one, held within it.
fn screen_texel(at: vec2<f32>) -> vec2<i32> {
  let size: vec2<i32> = vec2<i32>(textureDimensions(depth_target));

  return clamp(vec2<i32>(at * vec2<f32>(size)), vec2<i32>(0), size - 1);
}

// How much of an enhanced drop shows: its map's alpha, faded out within `NEAR_FADE` of the eye.
fn drop_cover(texel: vec4<f32>, view_at: vec3<f32>) -> f32 {
  return texel.a * smoothstep(NEAR_FADE.x, NEAR_FADE.y, -view_at.z);
}

// The light a drop scatters towards the eye, as it stands at `view_at`: the sky's light, as on a white surface, and
// the sun's through a forward-scattering phase (Henyey-Greenstein, one where it scatters evenly), mostly grey
// (`SCATTER_GREY`); shown at the frame's exposure, as a lit surface is.
fn scattered(view_at: vec3<f32>) -> vec3<f32> {
  let facing: f32 = dot(normalize(view_at), lighting.to_sun.xyz);
  let square: f32 = SUN_ASYMMETRY * SUN_ASYMMETRY;
  let phase: f32 = (1.0 - square) / pow(1.0 + square - 2.0 * SUN_ASYMMETRY * facing, 1.5);
  let light: vec3<f32> = lighting.forward_hemi.rgb * 2.0 + lighting.sun.rgb * phase * SUN_SCATTER;

  return tonemap(mix(light, vec3<f32>(luminance(light)), SCATTER_GREY), frame_scale(lighting, exposure));
}

// The enhanced streak in the scene: nothing drawn, only the distortion under it taken towards none by its cover, since
// the window pass bends the shown image by the water's distortion and a drop over water would zigzag with the waves.
@fragment
fn fs_streak_distortion(in: RainVarying) -> RainLook {
  let cover: f32 = drop_cover(textureSample(drop_map, map_sampler, in.uv), in.view_at);
  var out: RainLook;

  out.color = vec4<f32>(0.0);
  out.distortion = vec4<f32>(NEUTRAL_DISTORTION, NEUTRAL_DISTORTION, 0.0, cover);

  return out;
}

// `effects_rain.ps` at its high quality, drawn over the shown image `rain.aim.zw` texels across and lit as modern
// engines light rain: the blurred image behind the drop, bent by its map's offset where what the bend lands on stands
// behind the drop or is sky, and screened with the light about it, read far off by a wider bend, as a fake reflection;
// then towards the light the drop scatters (`scattered`) by `look.y`, in place of SSS's hemisphere added; covered by
// the map's alpha times the drops' cover where nothing nearer stands in front of it, tested against the drawn depth
// here, with no depth attachment of the shown image's size. Its arithmetic is on the colour shown, as the engine's
// forward passes draw, the light read as it is, as `s_accumulator` is read.
@fragment
fn fs_streak_enhanced(in: RainVarying) -> @location(0) vec4<f32> {
  let texel: vec4<f32> = textureSample(drop_map, map_sampler, in.uv);
  let bend: vec2<f32> = texel.xy * 2.0 - 1.0;
  let at: vec2<f32> = in.clip.xy / rain.aim.zw;
  let hemisphere: vec3<f32> = lighting.forward_hemi.rgb;
  let bright: f32 = saturate(dot(hemisphere, vec3<f32>(1.0)));
  let refraction: vec2<f32> = normalize(vec3<f32>(bend * DROP_BEND * rain.look.z, 1.0)).xy;
  let reflection: vec2<f32> = normalize(vec3<f32>(bend * (0.2 + bright * 0.5), 1.0)).xy;
  // The depth is reversed, so what stands behind the drop holds less of it, and the sky none.
  let behind: f32 = textureLoad(depth_target, screen_texel(saturate(at + refraction)), 0);
  let bent: vec2<f32> = select(vec2<f32>(0.0), refraction, behind < in.clip.z);
  let seen: vec3<f32> = scene_shown(textureSampleLevel(refracted, refracted_sampler, saturate(at + bent), 0.0).rgb);
  let light: vec3<f32> = textureLoad(light_target, screen_texel(saturate(at + reflection)), 0).rgb;
  let glint: vec3<f32> = saturate(light * rain.look.w);
  let shown: vec3<f32> = mix(1.0 - (1.0 - seen) * (1.0 - glint), scattered(in.view_at), rain.look.y);
  let is_seen: bool = textureLoad(depth_target, screen_texel(at), 0) < in.clip.z;

  let cover: f32 = select(0.0, drop_cover(texel, in.view_at) * rain.look.x, is_seen);

  return vec4<f32>(scene_from_shown(saturate(shown)), cover);
}

// `effects_rain_splash.ps` as a slight addition: the light the splash scatters, covered by its map's alpha times
// `splash_look.x` and fading as the square of what is left of its life; and the ground under it bent by its map through
// the distortion target, `splash_look.y` of its units, as a distorting particle's `l_special` pass writes it, so the
// ground bends at the frame's own size where SSS bent a blurred copy of it.
@fragment
fn fs_splash_enhanced(in: RainVarying) -> RainLook {
  let texel: vec4<f32> = textureSample(splash_map, map_sampler, in.uv);
  let outward: vec2<f32> = texel.xy * 2.0 - 1.0;
  let cover: f32 = texel.a * (1.0 - in.life) * (1.0 - in.life);
  var out: RainLook;

  out.color = vec4<f32>(scene_from_shown(scattered(in.view_at)), cover * rain.splash_look.x);
  out.distortion = vec4<f32>(vec2<f32>(NEUTRAL_DISTORTION) + outward * rain.splash_look.y, 0.0, cover);

  return out;
}

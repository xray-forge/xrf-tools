#import "common/camera"
#import "common/lighting"
#import "common/present"

// Overlays drawn over a viewport's finished frame in the window's pass, unlit, as the raw colours they name: line
// segments, hidden where the scene stands in front of them if they ask to be, the sun as a disc where the light comes
// from, and the level objects' helpers' wires, faint where the scene hides them. They are placed without the temporal jitter (`motion_current`), so they hold still under a temporal
// resolve; the jitter moves no depth, so the depth test still compares like with like.

#import "generated/frame/overlay"

// How far out the sun's disc stands, in metres: well inside the far plane, and past whatever is near.
const SUN_REACH: f32 = 1000.0;

struct LineVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) color: vec3<f32>,
  // One where what the scene draws in front hides the line.
  @location(1) @interpolate(flat) is_depth_tested: u32,
};

@vertex
fn vs_line(@location(0) position: vec3<f32>, @location(1) color: vec4<f32>) -> LineVarying {
  var out: LineVarying;

  out.clip = camera.motion_current * vec4<f32>(position, 1.0);
  out.color = color.rgb;
  out.is_depth_tested = u32(color.a > 0.5);

  return out;
}

@fragment
fn fs_line(in: LineVarying) -> @location(0) vec4<f32> {
  if (in.is_depth_tested != 0u) {
    let texel: vec2<i32> = to_drawn_texel(floor(in.clip.xy - present.origin), camera.viewport.xy, present.size);

    // Depth is reversed: the scene stands in front where it holds the larger value.
    if (textureLoad(depth_target, texel, 0) > in.clip.z) {
      discard;
    }
  }

  return vec4<f32>(in.color, 1.0);
}

struct PointVarying {
  @builtin(position) clip: vec4<f32>,
  // Where in the disc's square, from minus one to one.
  @location(0) corner: vec2<f32>,
  @location(1) @interpolate(flat) color: vec3<f32>,
  // The point's own depth, which a tested disc is hidden by as a whole.
  @location(2) @interpolate(flat) depth: f32,
  @location(3) @interpolate(flat) is_depth_tested: u32,
};

// A point's disc, two triangles of its square, `point.w` pixels across where it stands.
@vertex
fn vs_point(@builtin(vertex_index) index: u32, @location(0) point: vec4<f32>, @location(1) color: vec4<f32>)
  -> PointVarying {
  var corners: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
  );
  let center: vec4<f32> = camera.motion_current * vec4<f32>(point.xyz, 1.0);
  let corner: vec2<f32> = corners[index];
  var out: PointVarying;

  out.clip = vec4<f32>(center.xy + corner * point.w / present.size * center.w, center.z, center.w);
  out.corner = corner;
  out.color = color.rgb;
  out.depth = center.z / center.w;
  out.is_depth_tested = u32(color.a > 0.5);

  return out;
}

@fragment
fn fs_point(in: PointVarying) -> @location(0) vec4<f32> {
  if (length(in.corner) > 1.0) {
    discard;
  }

  if (in.is_depth_tested != 0u) {
    let texel: vec2<i32> = to_drawn_texel(floor(in.clip.xy - present.origin), camera.viewport.xy, present.size);

    // Depth is reversed: the scene stands in front where it holds the larger value.
    if (textureLoad(depth_target, texel, 0) > in.depth) {
      discard;
    }
  }

  return vec4<f32>(in.color, 1.0);
}

// How much of the draw distance a helper's wire stays whole over, fading out across the rest.
const HELPER_FADE_START: f32 = 0.75;

// How far toward white a selected helper's wire is lifted, so it reads apart from its kind's others; a hovered one is
// lifted half as far.
const HELPER_SELECTED_LIFT: f32 = 0.5;

struct HelperVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  // sRGB, then one where its object is selected, a half where it is hovered (`HelperVertex`).
  @location(1) @interpolate(flat) color: vec4<f32>,
  // The distance it has faded out by, zero for never, then how strongly it shows behind what is drawn.
  @location(2) @interpolate(flat) fade: vec2<f32>,
};

// A wire's end; the id it carries is the pick's, which this pass has no use for.
@vertex
fn vs_helper(@location(0) position: vec3<f32>, @location(1) color: vec4<f32>, @location(2) fade: vec2<f32>)
  -> HelperVarying {
  var out: HelperVarying;

  out.clip = camera.motion_current * vec4<f32>(position, 1.0);
  out.world = position;
  out.color = color;
  out.fade = fade;

  return out;
}

// How far in front of a helper the drawn scene must stand to hide it: some centimetres, and a share of its distance, as
// far as a texel of the jittered depth reaches across ground seen at a slant. A helper lying on what it marks (a
// patrol's links and points, a cover on the floor) is otherwise hidden and whole by turns as the jitter moves.
const HELPER_HIDDEN_SLACK: f32 = 0.2;
const HELPER_HIDDEN_SHARE: f32 = 0.02;

// Whether what is drawn at a texel (`scene`, its depth) hides a helper at `depth`, both reversed: one is the near
// plane.
fn is_helper_hidden(scene: f32, depth: f32) -> bool {
  if (scene <= depth) {
    return false;
  }

  let distance: f32 = camera_view_distance(depth);

  return camera_view_distance(scene) < distance - (HELPER_HIDDEN_SLACK + distance * HELPER_HIDDEN_SHARE);
}

// How much of a helper shows at a point of the world: whole where nothing drawn stands in front of it (`is_hidden`), at
// its occluded strength where something does, and fading out toward its draw distance, unless its object is singled
// out (`color.a`, `fade` as `HelperVertex` holds them).
fn helper_shown(is_hidden: bool, color: vec4<f32>, fade: vec2<f32>, world: vec3<f32>) -> f32 {
  let is_singled: bool = color.a > 0.0;
  let reach: f32 = fade.x;
  let distance: f32 = length(world - camera.position.xyz);
  let faded: f32 = select(1.0 - smoothstep(reach * HELPER_FADE_START, reach, distance), 1.0, reach <= 0.0);

  return select(1.0, select(fade.y, 1.0, is_singled), is_hidden) * faded;
}

// A helper's colour, lifted toward white as far as its object is singled out.
fn helper_colour(color: vec4<f32>) -> vec3<f32> {
  return mix(color.rgb, vec3<f32>(1.0), HELPER_SELECTED_LIFT * color.a);
}

// A helper's wire, hidden by what is drawn in front of it fragment by fragment.
@fragment
fn fs_helper(in: HelperVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = to_drawn_texel(floor(in.clip.xy - present.origin), camera.viewport.xy, present.size);
  let is_hidden: bool = is_helper_hidden(textureLoad(depth_target, texel, 0), in.clip.z);
  let shown: f32 = helper_shown(is_hidden, in.color, in.fade, in.world);

  if (shown <= 0.0) {
    discard;
  }

  return vec4<f32>(helper_colour(in.color), shown);
}

// The helper icons' atlas (`HelperIconAtlas`): cells of `HELPER_ICON_CELL` pixels, white pictures outlined in black.
@group(2) @binding(0) var helper_icon_atlas: texture_2d<f32>;
@group(2) @binding(1) var helper_icon_sampler: sampler;

// Window pixels an icon spans each way, whatever its distance; pixels an atlas cell spans.
const HELPER_ICON_SIZE: f32 = 40.0;
const HELPER_ICON_CELL: f32 = 64.0;

struct HelperIconVarying {
  @builtin(position) clip: vec4<f32>,
  // Where in the atlas.
  @location(0) uv: vec2<f32>,
  @location(1) @interpolate(flat) color: vec4<f32>,
  @location(2) @interpolate(flat) fade: vec2<f32>,
  @location(3) @interpolate(flat) world: vec3<f32>,
  // The icon's point in the viewport's pixels and its depth, which it is hidden by as a whole.
  @location(4) @interpolate(flat) point: vec2<f32>,
  @location(5) @interpolate(flat) depth: f32,
};

// An icon's square, two triangles `HELPER_ICON_SIZE` pixels across about its point, collapsed behind the eye.
@vertex
fn vs_helper_icon(
  @builtin(vertex_index) index: u32,
  @location(0) position: vec3<f32>,
  @location(1) icon: u32,
  @location(2) color: vec4<f32>,
  @location(3) fade: vec2<f32>,
) -> HelperIconVarying {
  var corners: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
  );
  let center: vec4<f32> = camera.motion_current * vec4<f32>(position, 1.0);
  let corner: vec2<f32> = corners[index];
  let atlas: vec2<f32> = vec2<f32>(textureDimensions(helper_icon_atlas));
  let columns: u32 = u32(atlas.x / HELPER_ICON_CELL);
  let cell: vec2<f32> = vec2<f32>(f32(icon % columns), f32(icon / columns));
  var out: HelperIconVarying;

  if (center.w <= 0.0) {
    out.clip = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    return out;
  }

  let ndc: vec2<f32> = center.xy / center.w;

  out.clip = vec4<f32>(center.xy + corner * HELPER_ICON_SIZE / present.size * center.w, center.z, center.w);
  // The atlas runs down where the view runs up.
  out.uv = (cell + corner * vec2<f32>(0.5, -0.5) + 0.5) * HELPER_ICON_CELL / atlas;
  out.color = color;
  out.fade = fade;
  out.world = position;
  out.point = (ndc * vec2<f32>(0.5, -0.5) + 0.5) * present.size;
  out.depth = center.z / center.w;

  return out;
}

// Drawn texels either way of an icon's point whose depth it is hidden by: the farthest of them, so the edge of a pole
// or a rail passing over the point, moving a texel as the frame's jitter does, does not hide it by turns.
const HELPER_ICON_REACH: i32 = 1;

// A helper's icon tinted in its kind's colour, its outline kept dark, hidden as a whole by what is drawn in front of
// its point.
@fragment
fn fs_helper_icon(in: HelperIconVarying) -> @location(0) vec4<f32> {
  let last: vec2<i32> = vec2<i32>(camera.viewport.xy) - 1;
  let center: vec2<i32> = to_drawn_texel(floor(in.point), camera.viewport.xy, present.size);
  var offsets: array<vec2<i32>, 5> = array<vec2<i32>, 5>(
    vec2<i32>(0, 0), vec2<i32>(HELPER_ICON_REACH, 0), vec2<i32>(-HELPER_ICON_REACH, 0),
    vec2<i32>(0, HELPER_ICON_REACH), vec2<i32>(0, -HELPER_ICON_REACH),
  );
  var farthest: f32 = 1.0;

  for (var at: i32 = 0; at < 5; at += 1) {
    // Depth is reversed: the farthest holds the least.
    farthest = min(farthest, textureLoad(depth_target, clamp(center + offsets[at], vec2<i32>(0), last), 0));
  }

  let shown: f32 = helper_shown(is_helper_hidden(farthest, in.depth), in.color, in.fade, in.world);
  let picture: vec4<f32> = textureSample(helper_icon_atlas, helper_icon_sampler, in.uv);

  if (shown * picture.a <= 0.0) {
    discard;
  }

  return vec4<f32>(picture.rgb * helper_colour(in.color), picture.a * shown);
}

struct SunVarying {
  @builtin(position) clip: vec4<f32>,
  // Where in the disc's square, from minus one to one.
  @location(0) corner: vec2<f32>,
  @location(1) @interpolate(flat) color: vec3<f32>,
};

// The disc's six corners, two triangles of its square, `sun.w` pixels across where the sun stands, in `sun.rgb`;
// behind the camera it collapses to nothing.
@vertex
fn vs_sun(@builtin(vertex_index) index: u32, @location(0) sun: vec4<f32>) -> SunVarying {
  var corners: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
  );
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let toward: vec3<f32> = normalize(rotation * lighting.to_sun.xyz);
  var clip: vec4<f32> = camera.motion_current * vec4<f32>(camera.position.xyz + toward * SUN_REACH, 1.0);
  let corner: vec2<f32> = corners[index];
  var out: SunVarying;

  clip = vec4<f32>(clip.xy + corner * sun.w / present.size * clip.w, 0.0, clip.w);
  out.clip = select(vec4<f32>(0.0), clip, clip.w > 0.0);
  out.corner = corner;
  out.color = sun.rgb;

  return out;
}

@fragment
fn fs_sun(in: SunVarying) -> @location(0) vec4<f32> {
  if (length(in.corner) > 1.0) {
    discard;
  }

  return vec4<f32>(in.color, 1.0);
}

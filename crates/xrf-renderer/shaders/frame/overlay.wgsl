#import "common/camera"
#import "common/lighting"
#import "common/present"

// Helpers drawn over a viewport's finished frame in the window's pass, unlit, as the raw colours they name: line
// segments, hidden where the scene stands in front of them if they ask to be, and the sun as a disc where the light
// comes from.

@group(1) @binding(0) var depth_target: texture_depth_2d;
@group(1) @binding(1) var<uniform> present: Present;
@group(1) @binding(2) var<uniform> lighting: Lighting;

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

  out.clip = camera.view_projection * vec4<f32>(position, 1.0);
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
  let center: vec4<f32> = camera.view_projection * vec4<f32>(point.xyz, 1.0);
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
  var clip: vec4<f32> = camera.view_projection * vec4<f32>(camera.position.xyz + toward * SUN_REACH, 1.0);
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

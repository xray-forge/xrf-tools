#import "common/camera"

// The level objects' helpers' wires drawn into a pick's one texel, through the camera narrowed to it, as quads a few
// window pixels wide: a one-pixel wire is too thin to click. Drawn after the scene's clusters and over them, so a click
// near a wire picks its object whatever stands in front, as an editor's helpers do.

// Window pixels either side of a wire, and past its ends, that a pick still meets it.
const HELPER_PICK_REACH: f32 = 4.0;

// The narrowed lens spans one window pixel across its whole view: two units of its device coordinates a pixel.
const PICK_UNITS_PER_PIXEL: f32 = 2.0;

// Nearer than this a wire's end is cut where it crosses, rather than divided by a `w` at or behind the eye.
const HELPER_PICK_NEAR: f32 = 1e-4;

// The pick texel's first word for a helper: the static pass writes one for a cluster and two for an impostor.
const PICKED_HELPER: u32 = 3u;

struct HelperPickVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) @interpolate(flat) object: u32,
};

// One wire a instance, its two ends; six corners of the quad about it, collapsed for a wire wholly behind the eye.
@vertex
fn vs_helper_pick(
  @builtin(vertex_index) index: u32,
  @location(0) start: vec3<f32>,
  @location(1) object: u32,
  @location(2) end: vec3<f32>,
) -> HelperPickVarying {
  var out: HelperPickVarying;
  var a: vec4<f32> = camera.view_projection * vec4<f32>(start, 1.0);
  var b: vec4<f32> = camera.view_projection * vec4<f32>(end, 1.0);

  out.object = object;

  if (a.w < HELPER_PICK_NEAR && b.w < HELPER_PICK_NEAR) {
    out.clip = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    return out;
  }

  if (a.w < HELPER_PICK_NEAR) {
    a = mix(a, b, (HELPER_PICK_NEAR - a.w) / (b.w - a.w));
  }

  if (b.w < HELPER_PICK_NEAR) {
    b = mix(b, a, (HELPER_PICK_NEAR - b.w) / (a.w - b.w));
  }

  let ends: array<vec3<f32>, 2> = array<vec3<f32>, 2>(a.xyz / a.w, b.xyz / b.w);
  let span: vec2<f32> = ends[1].xy - ends[0].xy;
  let along: vec2<f32> = select(vec2<f32>(1.0, 0.0), normalize(span), length(span) > 1e-6);
  let across: vec2<f32> = vec2<f32>(-along.y, along.x);
  let reach: f32 = HELPER_PICK_REACH * PICK_UNITS_PER_PIXEL;
  // The quad's corners: which end, and which side of the wire.
  var corners: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
  );
  let corner: vec2<f32> = corners[index];
  let at: vec3<f32> = select(ends[0], ends[1], corner.x > 0.5);
  let outward: f32 = select(-1.0, 1.0, corner.x > 0.5);

  out.clip = vec4<f32>(at.xy + across * corner.y * reach + along * outward * reach, at.z, 1.0);

  return out;
}

@fragment
fn fs_helper_pick(in: HelperPickVarying) -> @location(0) vec4<u32> {
  return vec4<u32>(PICKED_HELPER, in.object, 0u, bitcast<u32>(in.clip.z));
}

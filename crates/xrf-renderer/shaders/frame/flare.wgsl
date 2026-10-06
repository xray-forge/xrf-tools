#import "common/camera"
#import "common/flares"

// The lens flares and the gradient as `dxLensFlareRender` draws them over the finished frame: quads on a plane ahead of
// the camera, the flares along the line from the screen's centre through the sun and turned along it, the gradient
// about the sun; each its texture times its colour, `srcalpha, one`.

#import "generated/frame/flare"

// The instance the gradient is drawn as; the flares are the ones before it.
const GRADIENT_INSTANCE: u32 = 16u;

// How far ahead of the camera the plane stands, in view space units; the quads' sizes are shares of it.
const PLANE: f32 = 1.0;

// A quad's corners as two triangles, each from minus one to one across and down.
const CORNERS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
  vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
  vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
);

struct FlareVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
};

// A quad about a point of the plane, along one axis and down another; the corner along both has the texel at zero
// across and one down, as the engine lays its four vertices.
fn quad(vertex_index: u32, center: vec3<f32>, across: vec3<f32>, down: vec3<f32>, color: vec4<f32>) -> FlareVarying {
  var corners: array<vec2<f32>, 6> = CORNERS;
  let corner: vec2<f32> = corners[vertex_index % 6u];
  let point: vec3<f32> = center + across * corner.x + down * corner.y;
  var out: FlareVarying;

  out.clip = camera.projection * vec4<f32>(point, 1.0);
  out.clip.z = 0.5 * out.clip.w;
  out.uv = vec2<f32>(1.0 - corner.x, 1.0 + corner.y) * 0.5;
  out.color = color;

  return out;
}

// A quad no fragment of which is drawn.
fn hidden() -> FlareVarying {
  var out: FlareVarying;

  out.clip = vec4<f32>(2.0, 2.0, 2.0, 1.0);

  return out;
}

@vertex
fn vs_flare(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance: u32) -> FlareVarying {
  let to_sun: vec3<f32> = flares.to_sun.xyz;
  let sun_ahead: f32 = -to_sun.z;
  let shown: f32 = state[0];
  let faded: f32 = flares.sun.w;
  let light: vec3<f32> = clamp(flares.color.rgb, vec3<f32>(0.0), vec3<f32>(1.0));

  if (sun_ahead <= 0.01 || shown < FLARE_EPSILON) {
    return hidden();
  }

  let direction: vec3<f32> = vec3<f32>(0.0, 0.0, -1.0);
  let center: vec3<f32> = direction * PLANE;
  let sun: vec3<f32> = to_sun * (PLANE / sun_ahead);
  let axis: vec3<f32> = sun - center;
  // The engine's `vecY`, its right crossed with its view, points down the screen.
  let right: vec3<f32> = vec3<f32>(1.0, 0.0, 0.0);
  let down: vec3<f32> = vec3<f32>(0.0, -1.0, 0.0);

  if (instance == GRADIENT_INSTANCE) {
    let gradient: vec4<f32> = flares.gradient;
    let screen: vec4<f32> = camera.projection * vec4<f32>(sun, 1.0);
    let place: vec2<f32> = abs(screen.xy / screen.w);
    let fall: vec2<f32> = select(vec2<f32>(1.0), (2.5 - place) / 2.0, place > vec2<f32>(0.5));
    let value: f32 = select(fall.x * fall.y * faded * gradient.y * shown, 0.0, any(place > vec2<f32>(2.5)));
    let lit: f32 = value * faded;

    if (gradient.z < 0.5 || value < FLARE_EPSILON) {
      return hidden();
    }

    let extent: f32 = gradient.x * value * PLANE;

    return quad(vertex_index, sun, right * extent, down * extent, vec4<f32>(light * lit, lit));
  }

  let flare: vec4<f32> = flares.flares[instance];
  let along: vec3<f32> = normalize(axis + vec3<f32>(1e-6, 0.0, 0.0));
  // `vecDy`, the axis crossed with the view, the engine's handedness kept.
  let side: vec3<f32> = -cross(along, direction);
  let lit: f32 = flare.z * shown * faded;
  let extent: f32 = flare.y * PLANE;

  return quad(vertex_index, center + axis * flare.x, along * extent, side * extent, vec4<f32>(light * lit, lit));
}

@fragment
fn fs_flare(in: FlareVarying) -> @location(0) vec4<f32> {
  return textureSample(flare_texture, flare_sampler, in.uv) * in.color;
}

#import "common/camera"
#import "common/octahedral"
#import "common/quad"

// Calibration props drawn into the G-buffer like any surface, so every light, shadow, occlusion and reflection takes
// them: three spheres (`CalibrationUniform::SPHERES`: a grey card's 0.18, a white 0.8, a metal mirror), each ray-cast
// inside a quad facing the camera and writing its own depth, and a colour chart, a flat quad of the 24 patches.

#import "generated/frame/calibration"

// The props drawn as spheres, by their index; the chart follows them (`CalibrationUniform::SPHERES`).
const SPHERES: u32 = 3u;
// The chart's patches across and down.
const CHART_COLUMNS: u32 = 6u;
const CHART_ROWS: u32 = 4u;
// The share of a swatch's cell its black surround takes on each side.
const CHART_GAP: f32 = 0.08;
// What the chart's surround is: black, as the reference chart's frame is.
const CHART_SURROUND: vec3<f32> = vec3<f32>(0.08);
// How far past a sphere's silhouette its quad reaches, so no edge is cut by the quad's own.
const SILHOUETTE_MARGIN: f32 = 1.05;

// The 24 patches of the classic colour checker, row by row from the top left, as their published sRGB values.
const CHART_PATCHES: array<vec3<f32>, 24> = array<vec3<f32>, 24>(
  vec3<f32>(115.0, 82.0, 68.0), vec3<f32>(194.0, 150.0, 130.0), vec3<f32>(98.0, 122.0, 157.0),
  vec3<f32>(87.0, 108.0, 67.0), vec3<f32>(133.0, 128.0, 177.0), vec3<f32>(103.0, 189.0, 170.0),
  vec3<f32>(214.0, 126.0, 44.0), vec3<f32>(80.0, 91.0, 166.0), vec3<f32>(193.0, 90.0, 99.0),
  vec3<f32>(94.0, 60.0, 108.0), vec3<f32>(157.0, 188.0, 64.0), vec3<f32>(224.0, 163.0, 46.0),
  vec3<f32>(56.0, 61.0, 150.0), vec3<f32>(70.0, 148.0, 73.0), vec3<f32>(175.0, 54.0, 60.0),
  vec3<f32>(231.0, 199.0, 31.0), vec3<f32>(187.0, 86.0, 149.0), vec3<f32>(8.0, 133.0, 161.0),
  vec3<f32>(243.0, 243.0, 242.0), vec3<f32>(200.0, 200.0, 200.0), vec3<f32>(160.0, 160.0, 160.0),
  vec3<f32>(122.0, 122.0, 121.0), vec3<f32>(85.0, 85.0, 85.0), vec3<f32>(52.0, 52.0, 52.0),
);

struct CalibrationVarying {
  @builtin(position) clip: vec4<f32>,
  // Where the quad's point stands in view space, which a sphere's ray is cast through.
  @location(0) view_position: vec3<f32>,
  // The chart's point across and down its face, from nought to one.
  @location(1) chart: vec2<f32>,
  @location(2) @interpolate(flat) prop: u32,
};

struct CalibrationOutput {
  @builtin(frag_depth) depth: f32,
  @location(0) albedo: vec4<f32>,
  @location(1) normal: vec2<f32>,
  @location(2) material: vec4<f32>,
  @location(3) motion: vec2<f32>,
  @location(4) relief: vec2<f32>,
};

// A view space point in the world.
fn view_to_world(point: vec3<f32>) -> vec3<f32> {
  return (transpose(camera.view) * vec4<f32>(point, 0.0)).xyz + camera.position.xyz;
}

@vertex
fn vs_calibration(@builtin(vertex_index) vertex: u32, @builtin(instance_index) prop: u32) -> CalibrationVarying {
  let corner: vec2<f32> = QUAD_CORNERS[vertex];
  let place: vec4<f32> = calibration.props[prop];
  var out: CalibrationVarying;

  out.prop = prop;
  out.chart = vec2<f32>(corner.x * 0.5 + 0.5, 0.5 - corner.y * 0.5);

  if (prop < SPHERES) {
    // A quad through the sphere's centre, square to the eye's ray to it, as wide as the cone of sight around the sphere
    // meets it there; square to the view plane instead, an off-centre sphere's silhouette, stretched away from the
    // centre, would pass the quad's edge.
    let centre: vec3<f32> = (camera.view * vec4<f32>(place.xyz, 1.0)).xyz;
    let distance: f32 = length(centre);
    let radius: f32 = place.w;
    let toward: vec3<f32> = centre / max(distance, 1e-6);
    let across: vec3<f32> = normalize(cross(toward, select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0),
      abs(toward.y) > 0.99)));
    let up: vec3<f32> = cross(across, toward);
    let reach: f32 = select(0.0, radius * distance / sqrt(max(distance * distance - radius * radius, 1e-6)),
      distance > radius) * SILHOUETTE_MARGIN;
    let point: vec3<f32> = centre + (across * corner.x + up * corner.y) * reach;

    out.view_position = point;
    out.clip = camera.projection * vec4<f32>(point, 1.0);

    return out;
  }

  let world: vec3<f32> = place.xyz + calibration.right.xyz * corner.x * place.w + calibration.up.xyz * corner.y *
    calibration.up.w;
  let point: vec3<f32> = (camera.view * vec4<f32>(world, 1.0)).xyz;

  out.view_position = point;
  out.clip = camera.projection * vec4<f32>(point, 1.0);

  return out;
}

@fragment
fn fs_calibration(in: CalibrationVarying) -> CalibrationOutput {
  let surface: vec4<f32> = calibration.surfaces[in.prop];
  var point: vec3<f32> = in.view_position;
  var normal: vec3<f32>;
  var albedo: vec3<f32> = surface.rgb;

  if (in.prop < SPHERES) {
    // The ray from the eye through the quad's point, met by the sphere where it first enters it.
    let place: vec4<f32> = calibration.props[in.prop];
    let centre: vec3<f32> = (camera.view * vec4<f32>(place.xyz, 1.0)).xyz;
    let toward: vec3<f32> = normalize(in.view_position);
    let along: f32 = dot(toward, centre);
    let spread: f32 = along * along - (dot(centre, centre) - place.w * place.w);

    if (spread < 0.0) {
      discard;
    }

    point = toward * (along - sqrt(spread));
    normal = normalize(point - centre);
  } else {
    let cell: vec2<f32> = in.chart * vec2<f32>(f32(CHART_COLUMNS), f32(CHART_ROWS));
    let swatch: vec2<u32> = min(vec2<u32>(cell), vec2<u32>(CHART_COLUMNS - 1u, CHART_ROWS - 1u));
    let within: vec2<f32> = fract(cell);
    let is_surround: bool = any(within < vec2<f32>(CHART_GAP)) || any(within > vec2<f32>(1.0 - CHART_GAP));

    albedo = select(CHART_PATCHES[swatch.y * CHART_COLUMNS + swatch.x] / 255.0, CHART_SURROUND, is_surround);
    normal = normalize((camera.view * vec4<f32>(cross(calibration.right.xyz, calibration.up.xyz), 0.0)).xyz);
  }

  let clip: vec4<f32> = camera.projection * vec4<f32>(point, 1.0);
  let world: vec3<f32> = view_to_world(point);
  var out: CalibrationOutput;

  out.depth = clip.z / clip.w;
  out.albedo = vec4<f32>(albedo, surface.a);
  out.normal = octahedral_encode(normal);
  // Open to the whole sky and lit by the baked sun, as a prop standing in the open is.
  out.material = vec4<f32>(1.0, 1.0, calibration.slices[in.prop], encode_marks(false, false, false, false, false, false,
    false));
  out.motion = camera_motion(world, world);
  out.relief = vec2<f32>(0.0);

  return out;
}

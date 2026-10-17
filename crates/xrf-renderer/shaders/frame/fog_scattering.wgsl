#import "common/camera"
#import "common/scene_light"
#import "common/fullscreen"
#import "common/lighting"
#import "common/debanding"

// The enhanced fog's scattering, over the finished scene before it blooms: the scene as shown blurred to a quarter of
// its size, that blurred again to half, then the blur, debanded, laid over the scene wherever it is brighter, by how
// much enhanced fog lies there squared and by how alike the two are, so a lit window does not glow into the dark fog
// around it. The fog's own colour is already in the scene; this carries its light into the fog.

// The image a stage blurs or scatters into, the blur a stage before it left, their sampler, the depth the fog is
// measured by, the lighting and the stage's own target size, strength and clock.
#import "generated/frame/fog_scattering"

// The blur's spiral: each read turned this far round from the last.
const SPIRAL_TURN: mat2x2<f32> = mat2x2<f32>(-0.666276, -0.745705, 0.745705, -0.666276);
// Reads the blur takes.
const SPIRAL_READS: i32 = 12;
// The blur's debanding: its passes and the frame's pixels it reads out to at most.
const SCATTER_DEBAND_PASSES: u32 = 2u;
const SCATTER_DEBAND_RADIUS: f32 = 48.0;

// The spiral's start, the same at every pixel: interleaved gradient noise at a texel's centre, a quarter turn of it.
const SPIRAL_START: f32 = 1.4640113;

// A blur stage: twelve reads of the source spiralling out from the pixel, each a target's texel times its turn further;
// the scene's read as it is shown.
@fragment
fn fs_scatter_blur(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let size: vec2<f32> = scattering.size;
  let uv: vec2<f32> = in.clip.xy / size;
  let pixel: vec2<f32> = 1.0 / size;
  var offset: vec2<f32> = vec2<f32>(SPIRAL_START);
  var reach: f32 = 0.9;
  var sum: vec3<f32> = vec3<f32>(0.0);

  for (var index: i32 = 0; index < SPIRAL_READS; index++) {
    reach += 1.0 / reach;
    offset = offset * SPIRAL_TURN;
    let read: vec3<f32> = textureSampleLevel(source, source_sampler, uv + offset * (reach - 1.0) * pixel, 0.0).rgb;

    sum += select(read, scene_shown(read), scattering.is_reading_scene != 0u);
  }

  return vec4<f32>(sum / f32(SPIRAL_READS), 1.0);
}

// The scattering: the scene where nothing scatters, else towards the brighter of it and the half-size blur by the fog's
// share, squared and weighed by how alike the two are, past a tenth.
@fragment
fn fs_scattering(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let held: vec3<f32> = textureLoad(source, texel, 0).rgb;
  let image: vec3<f32> = scene_shown(held);
  let depth: f32 = textureLoad(depth_target, texel, 0);

  // Where nothing was drawn, the fog the engine measures lies at the eye: none.
  if (depth <= 0.0) {
    return vec4<f32>(held, 1.0);
  }

  let position: vec3<f32> = camera_view_position(in.clip.xy, depth);
  let height: f32 = camera.position.y + (transpose(camera.view) * vec4<f32>(position, 0.0)).y;
  let fog: f32 = fog_amount_at(lighting, position, height);
  let uv: vec2<f32> = in.clip.xy / scattering.size;
  let read: vec3<f32> = textureSampleLevel(blurred, source_sampler, uv, 0.0).rgb;
  let blur: vec3<f32> = debanded(blurred, source_sampler, read, uv, 1.0 / scattering.size, SCATTER_DEBAND_PASSES,
    SCATTER_DEBAND_RADIUS, scattering.time);
  let alike: f32 = dot(blur, saturate(image));
  let share: f32 = fog * fog * alike * alike;
  let scattered: vec3<f32> = mix(image, max(image, blur),
    smoothstep(0.1, 1.0 - scattering.intensity * 0.8, share));

  return vec4<f32>(scene_from_shown(scattered), 1.0);
}

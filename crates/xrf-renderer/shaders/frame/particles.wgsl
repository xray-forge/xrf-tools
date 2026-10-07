enable wgpu_binding_array;

#import "common/camera"
#import "common/lighting"

// The particle sprites `CParticleEffect::Render` draws forward over the finished scene: each quad's corners as the
// engine fills them, coloured by the particle and textured by its effect, faded where they meet the scene behind them
// (`USE_SOFT_PARTICLES`) and, on Anomaly, into the fog. Unlit: `particle.ps` is the vertex colour times the texture.
// A distorting effect's quads draw again into the distortion target, as `particle_distort.ps` writes them.

// The quads' corners as the engine fills them, the effects' surfaces, the frame's lighting, and the scene's depth.
#import "generated/frame/particles"

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var texture_sampler: sampler;

const SURFACE_IS_CLAMPED: u32 = 1u;

// `QuadIB`: a quad's four corners as two triangles.
const QUAD_CORNERS: array<u32, 6> = array<u32, 6>(0u, 1u, 2u, 3u, 2u, 1u);

// `DEPTH_EPSILON`: metres a particle stands in front of the scene before it is faded at all.
const DEPTH_EPSILON: f32 = 0.1;

// What the depth behind a particle reads where nothing was drawn: `100000`, as the sky writes no position.
const FAR_BEHIND: f32 = 100000.0;

// The least alpha drawn, `clip(result.a - 0.01 / 255)`.
const LEAST_ALPHA: f32 = 0.01 / 255.0;

struct ParticleVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) view_position: vec3<f32>,
  @location(3) @interpolate(flat) surface: u32,
};

@vertex
fn vs_particle(@builtin(vertex_index) index: u32) -> ParticleVarying {
  let corner: ParticleVertex = vertices[index / 6u * 4u + QUAD_CORNERS[index % 6u]];
  let world: vec4<f32> = vec4<f32>(corner.position, 1.0);
  var out: ParticleVarying;

  out.clip = camera.view_projection * world;
  out.uv = corner.uv;
  out.color = unpack4x8unorm(corner.color);
  out.view_position = (camera.view * world).xyz;
  out.surface = corner.surface;

  return out;
}

// `Contrast`: a piecewise curve steepening the middle of a fade.
fn contrast(input: f32, power: f32) -> f32 {
  let is_above_half: bool = input > 0.5;
  let raised: f32 = 0.5 * pow(saturate(2.0 * select(input, 1.0 - input, is_above_half)), power);

  return select(raised, 1.0 - raised, is_above_half);
}

// A texture of the surface's at the varying's coordinate, clamped or wrapped as the surface says.
fn sample_surface(surface: ParticleSurface, texture: u32, uv: vec2<f32>) -> vec4<f32> {
  // Taken before the branch: the texture's index and its sampler vary across the frame.
  let dx: vec2<f32> = dpdx(uv);
  let dy: vec2<f32> = dpdy(uv);

  if ((surface.flags & SURFACE_IS_CLAMPED) != 0u) {
    return textureSampleGrad(textures[texture], clamped_sampler, uv, dx, dy);
  }

  return textureSampleGrad(textures[texture], texture_sampler, uv, dx, dy);
}

// Anomaly's `particle.ps` and `particle_distort.ps` fade their alpha by what of the fog is left, squared; OpenXRay
// fogs no particle.
fn fogged(alpha: f32, view_position: vec3<f32>) -> f32 {
  if (lighting.engine.x > 0.5) {
    let left: f32 = 1.0 - fog_amount(lighting, view_position);

    return alpha * left * left;
  }

  return alpha;
}

@fragment
fn fs_particle(in: ParticleVarying) -> @location(0) vec4<f32> {
  let surface: ParticleSurface = surfaces[in.surface];
  var result: vec4<f32> = in.color * sample_surface(surface, surface.texture, in.uv);

  // `USE_SOFT_PARTICLES`: faded by how far the scene behind stands past it.
  let stored: f32 = textureLoad(depth_target, vec2<i32>(in.clip.xy), 0);
  let behind: f32 = -camera_view_position(in.clip.xy, stored).z;
  var space: f32 = behind + in.view_position.z - DEPTH_EPSILON;

  if (stored <= 0.0 || space < -2.0 * DEPTH_EPSILON) {
    space = FAR_BEHIND;
  }

  let fade: f32 = contrast(saturate(space * 1.3), 2.0);

  result = result * fade;

  if (result.a <= max(LEAST_ALPHA, surface.alpha_reference)) {
    discard;
  }

  result.a = fogged(result.a, in.view_position);

  return result;
}

// `particle_distort.ps`: the distortion map's colour, blended in by its alpha times the particle's mean colour; neither
// soft nor tested.
@fragment
fn fs_distort(in: ParticleVarying) -> @location(0) vec4<f32> {
  let surface: ParticleSurface = surfaces[in.surface];
  let distort: vec4<f32> = sample_surface(surface, surface.distortion, in.uv);
  let factor: f32 = distort.a * dot(in.color.rgb, vec3<f32>(0.33));

  return vec4<f32>(distort.rgb, fogged(factor, in.view_position));
}

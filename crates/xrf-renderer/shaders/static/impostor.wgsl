enable wgpu_binding_array;

#import "common/camera"
#import "common/octahedral"
#import "static/records"

// `details\lod` (`lod.vs`, `lod.ps`, `render_lods`): an impostor drawn as a quad of the two facets facing the camera
// best, each corner blended between them by the cull's factor and pulled half the sphere's radius towards the eye. The
// atlas is sampled at both facets' coordinates and blended the same way, its alpha faded by the cull and cut at 96; the
// `_nm` companion gives the view normal and the hemisphere term, times the corners'.

@group(2) @binding(0) var<storage, read> impostors: array<Impostor>;
// Two a corner: its position and hemisphere term, then its atlas coordinate and sun term.
@group(2) @binding(1) var<storage, read> corners: array<vec4<f32>>;
@group(2) @binding(2) var<storage, read> terms: array<vec4<u32>>;
@group(2) @binding(3) var<storage, read> impostor_list: array<u32>;
@group(2) @binding(4) var<storage, read> surfaces: array<Surface>;

@group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var texture_sampler: sampler;

// `clip(D.w - 96.h/255.h)`: what a faded impostor's alpha is cut at.
const ALPHA_REFERENCE: f32 = 96.0 / 255.0;

// `L_SCALE`, what `lod.vs` scales a corner's hemisphere term by.
const HEMI_SCALE: f32 = 3.1;

// What `def_gloss` writes for a surface without a bump.
const DEFAULT_GLOSS: f32 = 2.0 / 255.0;

// The material slice `lod.ps` writes: the default lighting model.
const IMPOSTOR_SLICE: f32 = 0.5 / 4.0;

// The quad's corners in draw order, then each corner's vertex within a facet, in the order `render_lods` takes them.
const QUAD: array<u32, 6> = array<u32, 6>(0u, 1u, 2u, 3u, 2u, 1u);
const FACET_VERTEX: array<u32, 4> = array<u32, 4>(3u, 0u, 2u, 1u);

struct ImpostorVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) next_uv: vec2<f32>,
  @location(1) best_uv: vec2<f32>,
  @location(2) hemi: f32,
  @location(3) @interpolate(flat) blend: f32,
  @location(4) @interpolate(flat) fade: f32,
  @location(5) @interpolate(flat) surface: u32,
  @location(6) @interpolate(flat) impostor: u32,
  // Where the quad stands in the world, taken to stand still there, as the tree it stands for does.
  @location(7) world: vec3<f32>,
};

struct GBufferOutput {
  @location(0) albedo: vec4<f32>,
  @location(1) normal: vec2<f32>,
  @location(2) material: vec4<f32>,
  @location(3) motion: vec2<f32>,
};

@vertex
fn vs_impostor(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32)
  -> ImpostorVarying {
  let index: u32 = impostor_list[instance_index];
  let impostor: Impostor = impostors[index];
  let term: vec4<u32> = terms[index];
  let factor: f32 = f32((term.z >> 8u) & 255u) / 255.0;
  let vertex: u32 = FACET_VERTEX[QUAD[vertex_index]];
  let next_at: u32 = ((index * IMPOSTOR_FACETS + term.y) * 4u + vertex) * 2u;
  let best_at: u32 = ((index * IMPOSTOR_FACETS + term.x) * 4u + vertex) * 2u;
  let next: vec4<f32> = corners[next_at];
  let best: vec4<f32> = corners[best_at];
  let sphere: vec4<f32> = impostor.sphere;
  let shift: vec3<f32> = normalize(sphere.xyz - camera.position.xyz) * (-0.5 * sphere.w);
  let world: vec3<f32> = mix(next.xyz, best.xyz, factor) + shift;
  var out: ImpostorVarying;

  out.clip = camera.view_projection * vec4<f32>(world, 1.0);
  out.world = world;
  out.next_uv = corners[next_at + 1u].xy;
  out.best_uv = corners[best_at + 1u].xy;
  out.hemi = mix(next.w, best.w, factor) * HEMI_SCALE;
  out.blend = factor;
  out.fade = f32(term.z & 255u) / 255.0;
  out.surface = impostor.surface;
  out.impostor = index;

  return out;
}

// Samples a slot at both facets' coordinates, blended, with the derivatives taken before any branch.
fn sample_blended(slot: u32, in: ImpostorVarying) -> vec4<f32> {
  let next: vec4<f32> = textureSampleGrad(textures[slot], texture_sampler, in.next_uv, dpdx(in.next_uv),
    dpdy(in.next_uv));
  let best: vec4<f32> = textureSampleGrad(textures[slot], texture_sampler, in.best_uv, dpdx(in.best_uv),
    dpdy(in.best_uv));

  return mix(next, best, in.blend);
}

fn is_cut(color: vec4<f32>, in: ImpostorVarying) -> bool {
  return color.a * in.fade < ALPHA_REFERENCE;
}

@fragment
fn fs_impostor(in: ImpostorVarying) -> GBufferOutput {
  let surface: Surface = surfaces[in.surface];
  let color: vec4<f32> = sample_blended(surface.base, in);
  let companion: vec4<f32> = sample_blended(surface.hemi, in);

  if (is_cut(color, in)) {
    discard;
  }

  // Baked in the engine's view space, where `+z` looks away from the eye.
  let baked: vec3<f32> = normalize(companion.xyz * 2.0 - 1.0);
  var out: GBufferOutput;

  out.albedo = vec4<f32>(mix(untextured_color(surface.color), color.rgb, camera.switches.x), DEFAULT_GLOSS);
  out.normal = octahedral_encode(vec3<f32>(baked.xy, -baked.z));
  // No baked sun term, as a tree writes none.
  out.material = vec4<f32>(companion.a * in.hemi, 1.0, IMPOSTOR_SLICE, 0.0);
  out.motion = camera_motion(in.world, in.world);

  return out;
}

// A pick's texel: an impostor, by its index, and its depth's bits.
@fragment
fn fs_pick_impostor(in: ImpostorVarying) -> @location(0) vec4<u32> {
  if (is_cut(sample_blended(surfaces[in.surface].base, in), in)) {
    discard;
  }

  return vec4<u32>(2u, in.impostor, 0u, bitcast<u32>(in.clip.z));
}

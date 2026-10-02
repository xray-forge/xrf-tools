enable dual_source_blending;

#import "static/gbuffer"
#import "common/hmodel"
#import "common/sun_shadow"

// Static surfaces composited over the lit frame, as the engine's forward passes draw them after the deferred ones:
// blended surfaces lit by the sun and the hemisphere, fogged and tonemapped, a spawned model's as `model_def_lq` or
// `model_env_lq` lights it; added and multiplied ones as their texture reads, fading into the fog by leaving what is
// behind them; and wall marks laid into the G-buffer's albedo before any light. Every blend is one equation, the surface's colour plus what is behind it times a second colour, so one
// pipeline a target draws them all.

@group(3) @binding(0) var<uniform> lighting: Lighting;
@group(3) @binding(1) var<storage, read> exposure: Exposure;
@group(3) @binding(2) var material_lut: texture_3d<f32>;
@group(3) @binding(3) var lut_sampler: sampler;
@group(3) @binding(4) var shadow_maps: texture_depth_2d_array;
@group(3) @binding(5) var<uniform> shadows: Shadows;
// The sky's own bind group, of which both skies, the irradiance cubes and their sampler are read.
@group(4) @binding(0) var sky_cube_0: texture_cube<f32>;
@group(4) @binding(1) var sky_cube_1: texture_cube<f32>;
@group(4) @binding(2) var sky_environment_0: texture_cube<f32>;
@group(4) @binding(3) var sky_environment_1: texture_cube<f32>;
@group(4) @binding(6) var sky_clamp: sampler;

struct CompositedOutput {
  // What the surface adds.
  @location(0) @blend_src(0) color: vec4<f32>,
  // What is behind it is multiplied by.
  @location(0) @blend_src(1) behind: vec4<f32>,
};

// A blended surface lit as the deferred frame lights it, the sun weighed by the lightmap's own term, then fogged and
// tonemapped as `combine_2` finishes a colour.
fn lit_color(shaded: GBufferOutput, position: vec3<f32>, fog: f32) -> vec3<f32> {
  let normal: vec3<f32> = octahedral_decode(shaded.normal);
  let slice: f32 = shaded.material.z;
  let light: vec4<f32> = sun_light(lighting, material_lut, lut_sampler, normal, position, slice) * shaded.material.y;
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let occlusion: f32 = mix(1.0, shaded.material.x, camera.switches.z);
  let color: vec3<f32> = hmodel(lighting, material_lut, lut_sampler, sky_environment_0, sky_environment_1, sky_clamp,
    shaded.albedo, light, normalize(rotation * normal), normalize(rotation * normalize(position)), slice, occlusion, 1.0);

  return tonemap(mix(color, lighting.fog_color.rgb, fog), frame_scale(lighting, exposure));
}

// A blended model as the engine draws it forward, lit per vertex rather than by `hmodel`: by its object's sky share over
// the up-facing hemisphere, the ambient, and the sun where it reaches, times its base, doubled. `model_env_lq` mixes the
// base toward the sky's reflection where its alpha is thin; `model_def_lq` takes it as it is. The sun's reach is read
// from the cascades at the pixel, where the engine casts one ray an object.
fn model_color(in: GBufferVarying, texel: vec4<f32>, position: vec3<f32>, is_mirrored: bool) -> vec3<f32> {
  let normal: vec3<f32> = normalize(in.normal);
  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let normal_world: vec3<f32> = normalize(rotation * normal);
  let world: vec3<f32> = rotation * position + camera.position.xyz;
  let facing: f32 = dot(normal, lighting.to_sun.xyz);
  let sun: f32 = sun_shadow(shadow_maps, shadows, world, normal_world, facing);
  let light: vec3<f32> = in.sky * max(0.0, normal_world.y) * lighting.environment.rgb + lighting.ambient.rgb
    + sun * lighting.sun.rgb * saturate(facing);

  if (!is_mirrored) {
    return light * texel.rgb * 2.0;
  }

  let reflected: vec3<f32> = reflect(normalize(rotation * position), normal_world);
  let lookup: vec3<f32> = vec3<f32>(reflected.x, reflected.y, -reflected.z);
  let reflection: vec3<f32> = mix(textureSampleLevel(sky_cube_0, sky_clamp, lookup, 0.0).rgb,
    textureSampleLevel(sky_cube_1, sky_clamp, lookup, 0.0).rgb, lighting.sky.w);

  return light * mix(reflection, texel.rgb, texel.a) * 2.0;
}

// A surface composited by its flags: multiplied into what is behind it, added to it, or laid over it by its alpha,
// `laid` being what a blended one lays. Fog fades each into leaving what is behind it alone, and its alpha is kept.
fn composite(flags: u32, texel: vec3<f32>, alpha: f32, laid: vec3<f32>, fog: f32) -> CompositedOutput {
  var out: CompositedOutput;

  if ((flags & SURFACE_IS_MULTIPLIED) != 0u) {
    let factor: f32 = select(1.0, 2.0, (flags & SURFACE_IS_DOUBLED) != 0u);

    out.color = vec4<f32>(0.0);
    out.behind = vec4<f32>(mix(texel * factor, vec3<f32>(1.0), fog), 1.0);
  } else if ((flags & SURFACE_IS_ADDED) != 0u) {
    let weight: f32 = select(1.0, alpha, (flags & SURFACE_IS_WEIGHTED) != 0u);

    out.color = vec4<f32>(texel * weight * (1.0 - fog), 0.0);
    out.behind = vec4<f32>(1.0);
  } else {
    out.color = vec4<f32>(laid * alpha, 0.0);
    out.behind = vec4<f32>(vec3<f32>(1.0 - alpha), 1.0);
  }

  return out;
}

@fragment
fn fs_composited(in: GBufferVarying) -> CompositedOutput {
  let at: Footprint = take_footprint(in);
  let base: vec4<f32> = base_texel(in, at);
  let surface: Surface = surfaces[in.surface];
  let position: vec3<f32> = camera_view_position(in.clip.xy, in.clip.z);
  let is_lit: bool = lighting.params.y > 0.5;
  let fog: f32 = select(0.0, fog_amount(lighting, position), is_lit);

  // Its own reference where it has one; and past total fog, nothing, as the far plane ends there.
  if (((surface.flags & SURFACE_IS_CUT_OUT) != 0u && base.a <= surface.alpha_reference) || fog >= 1.0) {
    discard;
  }

  let shaded: GBufferOutput = shade(in, base, at);
  let texel: vec3<f32> = mix(surface.color, base.rgb, camera.switches.x);

  let is_blended: bool = (surface.flags & (SURFACE_IS_ADDED | SURFACE_IS_MULTIPLIED)) == 0u;

  if ((surface.flags & SURFACE_IS_MODEL) != 0u && is_blended && is_lit) {
    let is_mirrored: bool = (surface.flags & SURFACE_IS_ENVIRONMENT_MAPPED) != 0u;
    let color: vec3<f32> = model_color(in, vec4<f32>(texel, base.a), position, is_mirrored);
    let scale: f32 = frame_scale(lighting, exposure);

    // `model_env_lq` fogs, and fades by the fog's square in its alpha as well; `model_def_lq` does neither.
    if (!is_mirrored) {
      return composite(surface.flags, texel, base.a, tonemap(color, scale), 0.0);
    }

    let laid: vec3<f32> = tonemap(mix(color, lighting.fog_color.rgb, fog), scale);

    return composite(surface.flags, texel, base.a * (1.0 - fog) * (1.0 - fog), laid, 0.0);
  }

  let laid: vec3<f32> = select(shaded.albedo.rgb, lit_color(shaded, position, fog), is_lit);

  return composite(surface.flags, texel, base.a, laid, fog);
}

// A wall mark laid into the G-buffer's albedo as `wmark` lays it: its base alone, at its top level and clamped, unlit
// and unfogged, the gloss left alone. Nothing cuts it, as `simple.ps` reads no reference.
@fragment
fn fs_wallmark(in: GBufferVarying) -> CompositedOutput {
  let surface: Surface = surfaces[in.surface];
  var base: vec4<f32> = vec4<f32>(1.0);

  if ((surface.flags & SURFACE_HAS_BASE) != 0u) {
    base = textureSampleLevel(textures[surface.base], texture_sampler, saturate(in.uv * surface.tiling), 0.0);
  }

  let texel: vec3<f32> = mix(surface.color, base.rgb, camera.switches.x);

  return composite(surface.flags, texel, base.a, texel, 0.0);
}

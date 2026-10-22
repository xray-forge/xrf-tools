#import "static/gbuffer"
#import "common/scene_light"
#import "common/hmodel"
#import "common/sky_box"
#import "common/sun_shadow"

// The enhanced glass, physically based: what stands behind a pane read from the frame as it stood before the glass,
// moved by the pane's slight waviness by less the further it stands, refused where that read lands on something nearer
// than the pane, blurred where its base is dirty; the transmitted light less what the fresnel reflects, the sky
// reflected by it as far as the pane stands open to it, its dirt lit as the forward passes light theirs, and the sun's
// highlight on it through the cascades. Written whole, the frame behind already read.

#import "generated/static/glass"
// The sky's own bind group, of which both skies and their sampler are read.
@group(4) @binding(0) var sky_cube_0: texture_cube<f32>;
@group(4) @binding(1) var sky_cube_1: texture_cube<f32>;
@group(4) @binding(6) var sky_clamp: sampler;

// Glass's reflectance head on, `F0` for an index of 1.5.
const GLASS_F0: f32 = 0.04;
// How far a pane's waviness tilts its normal, and the metres one wave of it spans.
const GLASS_WAVINESS: f32 = 0.035;
const GLASS_WAVE: f32 = 0.6;
// The share of the screen a tilt of one moves what is behind a pane a metre away, less the further it stands.
const GLASS_SHIFT: f32 = 0.12;
// Pixels the frame behind a fully dirty pane is blurred over at a metre, less the further it stands, and the reads.
const GLASS_BLUR: f32 = 10.0;
const GLASS_BLUR_TAPS: u32 = 8u;
// A clean pane's roughness and a fully dirty one's, which the sun's highlight spreads by.
const GLASS_ROUGHNESS: vec2<f32> = vec2<f32>(0.06, 0.45);
// How much of the sky a pane reflects at an object's sky share, as `OPEN_SKY` is open sky.
const GLASS_OPEN: f32 = 1.0 / OPEN_SKY;

// The sky's cube a world direction reflects, between the two keyframes as the sky itself is.
fn glass_sky(reflected: vec3<f32>) -> vec3<f32> {
  let lookup: vec3<f32> = cube_lookup(reflected);

  return mix(textureSampleLevel(sky_cube_0, sky_clamp, lookup, 0.0).rgb,
    textureSampleLevel(sky_cube_1, sky_clamp, lookup, 0.0).rgb, lighting.sky.w);
}

// A pane's normal in view space, tilted by a slow waviness over the world so what is behind it bends as old glass bends
// it, as two crossed waves.
fn glass_normal(in: GBufferVarying) -> vec3<f32> {
  let normal: vec3<f32> = normalize(in.normal);
  let tangent: vec3<f32> = normalize(in.tangent);
  let binormal: vec3<f32> = normalize(in.binormal);
  let place: vec3<f32> = in.world / GLASS_WAVE;
  let tilt: vec2<f32> = vec2<f32>(sin(place.x * 1.7 + place.y * 0.6 + sin(place.z * 1.3)),
    sin(place.z * 1.9 - place.y * 0.8 + sin(place.x * 1.1))) * GLASS_WAVINESS;

  return normalize(normal + tangent * tilt.x + binormal * tilt.y);
}

// The frame's copy at a share of the screen, or at `fallback` where that read lands on something nearer than the pane.
fn behind(uv: vec2<f32>, fallback: vec2<f32>, pane: f32) -> vec3<f32> {
  let size: vec2<f32> = vec2<f32>(textureDimensions(glass_scene));
  let texel: vec2<i32> = vec2<i32>(clamp(uv * size, vec2<f32>(0.0), size - 1.0));
  // Depth is reversed: the larger stands nearer.
  let is_nearer: bool = textureLoad(depth_target, texel, 0) > pane;

  return scene_shown(textureSampleLevel(glass_scene, scene_sampler, select(uv, fallback, is_nearer), 0.0).rgb);
}

// The light a pane's dirt is lit by, as the forward passes light a surface: the ambient, the hemisphere as open as the
// pane stands, and the sun where it reaches.
fn dirt_light(normal_world: vec3<f32>, open: f32, sun: f32, to_sun: vec3<f32>) -> vec3<f32> {
  return lighting.forward_ambient.rgb + lighting.forward_hemi.rgb * (0.5 + 0.5 * normal_world.y) * open +
    lighting.forward_sun.rgb * saturate(dot(normal_world, to_sun)) * sun;
}

@fragment
fn fs_glass(in: GBufferVarying) -> @location(0) vec4<f32> {
  let surface: Surface = surfaces[in.surface];
  let at: Footprint = take_footprint(in);
  let base: vec4<f32> = base_texel(in, at);

  if (base.a <= surface.alpha_reference) {
    discard;
  }

  let position: vec3<f32> = camera_view_position(in.clip.xy, in.clip.z);
  let fog: f32 = fog_amount_at(lighting, position, in.world.y);

  if (fog_amount(lighting, position) >= 1.0) {
    discard;
  }

  let size: vec2<f32> = camera.viewport.xy;
  let uv: vec2<f32> = in.clip.xy / size;
  let distance: f32 = max(-position.z, 1.0);
  let dirt: f32 = base.a;
  let normal: vec3<f32> = glass_normal(in);
  let flat_normal: vec3<f32> = normalize(in.normal);
  // Seen from behind, the pane faces the eye the other way.
  let eye: vec3<f32> = camera_view_ray(position);
  let facing: vec3<f32> = select(normal, -normal, dot(flat_normal, eye) > 0.0);
  let cosine: f32 = saturate(dot(facing, -eye));
  let fresnel: f32 = GLASS_F0 + (1.0 - GLASS_F0) * pow(1.0 - cosine, 5.0);

  // What stands behind, moved by the waviness's tilt against the flat pane and blurred by the dirt over a disc.
  let shift: vec2<f32> = (facing.xy - select(flat_normal, -flat_normal, dot(flat_normal, position) > 0.0).xy) *
    vec2<f32>(1.0, -1.0) * glass.refraction * GLASS_SHIFT / distance;
  let moved: vec2<f32> = uv + shift;
  let radius: f32 = dirt * GLASS_BLUR / distance / size.y;
  var transmitted: vec3<f32> = behind(moved, uv, in.clip.z);

  if (radius * size.y > 0.5) {
    for (var tap: u32 = 0u; tap < GLASS_BLUR_TAPS; tap++) {
      let angle: f32 = f32(tap) * 2.39996323;
      let along: f32 = sqrt((f32(tap) + 0.5) / f32(GLASS_BLUR_TAPS)) * radius;
      let offset: vec2<f32> = vec2<f32>(cos(angle) * size.y / size.x, sin(angle)) * along;

      transmitted += behind(moved + offset, uv, in.clip.z);
    }

    transmitted /= f32(GLASS_BLUR_TAPS + 1u);
  }

  let rotation: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
  let normal_world: vec3<f32> = normalize(rotation * facing);
  let to_sun_world: vec3<f32> = normalize(rotation * lighting.to_sun.xyz);
  // How open to the sky the pane stands: a model's sky share, a level surface's baked hemisphere.
  let open: f32 = select(saturate(in.hemi), saturate(in.sky * GLASS_OPEN), (surface.flags & SURFACE_IS_MODEL) != 0u);
  let sun: f32 = sun_shadow(shadow_maps, shadows, in.world, normal_world, dot(facing, lighting.to_sun.xyz));
  let texel: vec3<f32> = mix(untextured_color(surface.color), base.rgb, camera.switches.x);
  let dirt_lit: vec3<f32> = saturate(dirt_light(normal_world, open, sun, to_sun_world) * texel * 2.0);
  let reflection: vec3<f32> = glass_sky(reflect(normalize(rotation * position), normal_world)) * open;

  // The sun's highlight, a normalised Blinn lobe as rough as the pane is dirty, through the cascades.
  let roughness: f32 = mix(GLASS_ROUGHNESS.x, GLASS_ROUGHNESS.y, dirt);
  let power: f32 = 2.0 / max(roughness * roughness * roughness * roughness, 1e-4) - 2.0;
  let half_way: vec3<f32> = normalize(lighting.to_sun.xyz - eye);
  let lobe: f32 = (power + 2.0) / 8.0 * pow(saturate(dot(facing, half_way)), power) *
    saturate(dot(facing, lighting.to_sun.xyz));
  let highlight: vec3<f32> = shown_from_light(lighting.sun.rgb * lobe * fresnel * sun, frame_scale(lighting, exposure));

  let through: f32 = (1.0 - fresnel) * (1.0 - dirt);
  let laid: vec3<f32> = reflection * fresnel * (1.0 - dirt) + dirt_lit * dirt + highlight * (1.0 - dirt);
  let covered: f32 = 1.0 - through;

  return vec4<f32>(scene_from_shown(transmitted * through + mix(laid, lighting.fog_color.rgb * covered, fog)), 1.0);
}

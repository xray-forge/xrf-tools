enable wgpu_binding_array;

#import "common/camera"
#import "common/fullscreen"
#import "common/local_lights"
#import "common/scene_light"
#import "common/lighting"

// The shafts of the shadowed local lights, as the engines' `accum_volumetric` draws them: slices faced to the camera
// across the box about each light's sphere, stepping away from it, each pixel of a slice the light's falloff, shadow,
// projector and a scrolling noise times a fixed density, added up where the scene stands behind it. A spot is clipped
// by its frustum; a shadowed point is one volume over its sphere, each pixel in the face its direction falls in, as the
// engine's six omni parts are. Drawn at half the frame's size against its depth taken there, the shafts being soft;
// the sun shafts' composite brings them up by depth.

#import "generated/frame/light_volumes"

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var texture_sampler: sampler;

// `VOLUMETRIC_SLICES`: the slices the engine's buffer holds, the first of them drawn, spanning nought to one.
const VOLUME_SLICES: f32 = 100.0;

// `fDensity`: what one slice adds of the light.
const VOLUME_DENSITY: f32 = 1.0 / 40.0;

// The corner of a slice each of its six vertices takes, its `x` in the high bit and `y` in the low.
const SLICE_CORNERS: array<u32, 6> = array<u32, 6>(0u, 1u, 2u, 2u, 1u, 3u);

struct VolumeVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) position: vec3<f32>,
  @location(1) @interpolate(flat) volume: u32,
};

@vertex
fn vs_light_volume(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) volume: u32) -> VolumeVarying {
  let record: LightVolume = light_volumes[volume];
  let corner: u32 = SLICE_CORNERS[vertex_index % 6u];
  let along: f32 = f32(vertex_index / 6u) / (VOLUME_SLICES - 1.0);
  let depth: f32 = mix(record.near.z, record.far.z, along);
  // The slice held to the square about the falloff's circle at its depth: past it the engine's slice adds nothing, and
  // a slice the circle misses draws nothing at all.
  let light: vec3<f32> = records[u32(record.far.w)].position.xyz;
  let reach: f32 = inverseSqrt(max(record.color.w, 1e-12));
  let across: f32 = reach * reach - (depth - light.z) * (depth - light.z);
  let half_side: f32 = sqrt(max(across, 0.0));
  let least: vec2<f32> = max(record.near.xy, light.xy - half_side);
  let most: vec2<f32> = max(least, min(record.far.xy, light.xy + half_side));
  let position: vec3<f32> = vec3<f32>(mix(least, most, vec2<f32>(f32(corner >> 1u), f32(corner & 1u))), depth);
  var out: VolumeVarying;

  out.clip = camera.projection * vec4<f32>(position, 1.0);
  out.position = position;
  out.volume = volume;

  return out;
}

// The frame's depth at half its size, which the slices are tested against: each texel the first of its two by two, as
// the composite brings them up by.
@fragment
fn fs_volume_depth(in: FullscreenVarying) -> @builtin(frag_depth) f32 {
  let last: vec2<i32> = vec2<i32>(textureDimensions(depth_target)) - 1;

  return textureLoad(depth_target, min(vec2<i32>(in.clip.xy) * 2, last), 0);
}

// The face of a point light a direction from it, in the world, falls in: its longest axis, as `light_shadow` picks it.
fn point_face(world: vec3<f32>) -> u32 {
  let size: vec3<f32> = abs(world);

  if (size.x >= size.y && size.x >= size.z) {
    return select(1u, 0u, world.x > 0.0);
  }

  if (size.y >= size.z) {
    return select(3u, 2u, world.y > 0.0);
  }

  return select(5u, 4u, world.z > 0.0);
}

@fragment
fn fs_light_volume(in: VolumeVarying) -> @location(0) vec4<f32> {
  let volume: LightVolume = light_volumes[in.volume];
  let record: LightRecord = records[u32(volume.far.w)];
  let to_point: vec3<f32> = in.position - record.position.xyz;
  // `plight_local`'s falloff, before anything is read: most of a slice stands outside the light's reach, where the
  // engine adds nothing either.
  let falloff: f32 = saturate(1.0 - dot(to_point, to_point) * volume.color.w);

  if (falloff <= 0.0) {
    discard;
  }

  let is_spot: bool = volume.frustum.z > 0.5;
  var across: vec2<f32>;
  var along: f32;
  var scale: f32;

  if (is_spot) {
    across = vec2<f32>(dot(to_point, record.right.xyz), dot(to_point, record.up.xyz));
    along = dot(to_point, record.axis.xyz);
    scale = record.right.w;

    // `FrustumClipPlane`: the light's shadow frustum, its far plane pulled in by its distance.
    if (along < record.shadow.x || along > volume.frustum.x || any(abs(across) * scale > vec2<f32>(along))) {
      discard;
    }
  } else {
    let to_world: mat3x3<f32> = transpose(mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz));
    let world: vec3<f32> = to_world * to_point;
    let face: u32 = point_face(world);

    across = vec2<f32>(dot(world, POINT_RIGHTS[face]), dot(world, POINT_UPS[face]));
    along = dot(world, POINT_DIRECTIONS[face]);
    scale = POINT_SCALE;
  }

  // One comparison at the point, filtered between its four texels by the sampler: at the shafts' half size and under
  // their noise, `shadow_hw`'s four taps look the same at four times the cost.
  let place: ShadowPlace = shadow_place(record, to_point, vec3<f32>(0.0), is_spot, false);
  let shadow: f32 = textureSampleCompareLevel(shadow_atlas, shadow_sampler, place.centre * ATLAS_TEXEL,
    1.0 - place.moved) * place.fade;
  // The projector's coordinates (`m_lmap`), which the noise reads a third of, scrolling two ways.
  let uv: vec2<f32> = face_uv(across, along, scale);
  let projected: vec3<f32> = textureSampleLevel(textures[u32(volume.frustum.y)], texture_sampler, uv, 0.0).rgb;
  let noise_uv: vec2<f32> = uv / 3.0;
  let noise_slot: u32 = volumes.noise;
  let noise: vec4<f32> = textureSampleLevel(textures[noise_slot], texture_sampler, noise_uv + vec2<f32>(volumes.scroll, 0.0),
    0.0) * textureSampleLevel(textures[noise_slot], texture_sampler,
    noise_uv - vec2<f32>(0.0, volumes.scroll * 0.70091), 0.0);
  // The density, the falloff and the noise are amounts the engine tunes as display values, the projector a picture.
  // At the authored levels they gather display values, in the lamp's authored colour, which the composite adds to the
  // frame's as the engine adds its shafts (`fs_sun_shafts_over`), as the froxels gather them; else, under the physical
  // light, each is decoded, as the lamp's own colour already is, or the shafts stand many times brighter against the
  // light they come from than the engine's.
  let amount: vec3<f32> = vec3<f32>(VOLUME_DENSITY * shadow * falloff) * (noise.rgb * 0.5 + 0.5);

  if (is_light_summed(lighting)) {
    return vec4<f32>(amount * projected * srgb_light_authored(volume.color.rgb / local_light_scale(lighting)), 0.0);
  }

  let shaft: vec3<f32> = select(amount, srgb_decoded(amount), is_light_physical()) * authored_colour(projected);

  return vec4<f32>(shaft * volume.color.rgb, 0.0);
}

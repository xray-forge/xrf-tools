#import "common/sky_box"
#import "common/sky_deband"

// A sky cube smoothed where its stored steps band (`common/sky_deband`) into a float copy of its six faces, each tap
// read through the cube by a direction off the texel's on its face's plane, so a tap past a face's edge reads the face
// beside it and no seam shows.

// The cube as the weather holds it, its sampler, and the copy's six faces.
#import "generated/frame/sky_deband_cube"

@compute @workgroup_size(8, 8, 1)
fn cs_deband_cube(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec2<u32> = textureDimensions(faces);

  if (any(id.xy >= size) || id.z >= 6u) {
    return;
  }

  let texel_uv: vec2<f32> = 1.0 / vec2<f32>(size);
  let uv: vec2<f32> = (vec2<f32>(id.xy) + 0.5) * texel_uv;
  let texel: vec4<f32> = textureSampleLevel(cube, cube_sampler, cube_face_direction(id.z, uv), 0.0);
  var sum: vec4<f32> = texel;
  var weight: vec4<f32> = vec4<f32>(1.0);

  for (var index: u32 = 0u; index < SKY_DEBAND_RINGS * SKY_DEBAND_TAPS; index++) {
    let at: vec3<f32> = cube_face_direction(id.z, uv + sky_deband_offset(index) * texel_uv);
    let tap: vec4<f32> = textureSampleLevel(cube, cube_sampler, at, 0.0);
    let blend: vec4<f32> = sky_deband_weight(texel, tap);

    sum += tap * blend;
    weight += blend;
  }

  textureStore(faces, vec2<i32>(id.xy), i32(id.z), sum / weight);
}

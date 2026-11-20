#import "common/sky_deband"

// A cloud texture smoothed where its stored steps band (`common/sky_deband`) into a float copy, its taps wrapping round
// as the clouds tile.

// The clouds as the weather holds them, their repeating sampler, and the copy.
#import "generated/frame/sky_deband_flat"

@compute @workgroup_size(8, 8, 1)
fn cs_deband_flat(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec2<u32> = textureDimensions(smoothed);

  if (any(id.xy >= size)) {
    return;
  }

  let texel_uv: vec2<f32> = 1.0 / vec2<f32>(size);
  let uv: vec2<f32> = (vec2<f32>(id.xy) + 0.5) * texel_uv;
  let texel: vec4<f32> = textureSampleLevel(clouds, clouds_sampler, uv, 0.0);
  var sum: vec4<f32> = texel;
  var weight: vec4<f32> = vec4<f32>(1.0);

  for (var index: u32 = 0u; index < SKY_DEBAND_RINGS * SKY_DEBAND_TAPS; index++) {
    let tap: vec4<f32> = textureSampleLevel(clouds, clouds_sampler, uv + sky_deband_offset(index) * texel_uv, 0.0);
    let blend: vec4<f32> = sky_deband_weight(texel, tap);

    sum += tap * blend;
    weight += blend;
  }

  textureStore(smoothed, vec2<i32>(id.xy), sum / weight);
}

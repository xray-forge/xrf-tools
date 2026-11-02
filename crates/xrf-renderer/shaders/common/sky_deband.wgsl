// The sky's textures smoothed where their stored steps band, once a texture is up, for the sky the frame draws: each
// texel averaged with the taps about it that lie within a few of the steps it is stored in, a channel at a time, so a
// gradient's 8-bit and block-compressed steps blend into one another and a cloud's edge, many steps high, stays. Stated
// in the textures' own steps, not the shown image's, so it reads the same at any exposure and hour, the night's few
// shown levels included, where a filter over the shown image blends the clouds' shapes away.

// What a tap may differ from the texel by, a channel, and still blend: a few of the 8-bit steps the textures are stored
// in, which a block's interpolated colours step by too.
const SKY_DEBAND_STEP: f32 = 4.0 / 255.0;
// Rings of taps about a texel, each twice as far out as the last from one texel, and the taps a ring.
const SKY_DEBAND_RINGS: u32 = 4u;
const SKY_DEBAND_TAPS: u32 = 8u;

// A tap's weight a channel: whole within a third of a step of the texel, none past a step.
fn sky_deband_weight(texel: vec4<f32>, tap: vec4<f32>) -> vec4<f32> {
  return saturate((1.0 - abs(tap - texel) / SKY_DEBAND_STEP) * 1.5);
}

// Where tap `index` lies from its texel, in texels: its ring's distance along its direction, each ring turned half a
// tap from the last so the rings do not line up.
fn sky_deband_offset(index: u32) -> vec2<f32> {
  let ring: u32 = index / SKY_DEBAND_TAPS;
  let angle: f32 = (f32(index % SKY_DEBAND_TAPS) + 0.5 * f32(ring % 2u)) * 6.2831853 / f32(SKY_DEBAND_TAPS);

  return vec2<f32>(cos(angle), sin(angle)) * f32(1u << ring);
}

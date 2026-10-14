// The enhanced parallax's fade with distance, which the G-buffer's march and the wet pass's water in the relief share,
// so the water stands exactly where the relief does.

// The share of the range the enhanced parallax fades out over, ending there.
const RELIEF_FADE_SHARE: f32 = 0.2;

// How much of the enhanced parallax stands at `distance`: whole short of the last of `range`, none at it. Its depth,
// hollows and height blending all fade by it, so nothing it changes ends at a line.
fn relief_fade(range: f32, distance: f32) -> f32 {
  return 1.0 - smoothstep(range * (1.0 - RELIEF_FADE_SHARE), range, distance);
}

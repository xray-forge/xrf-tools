// A linear colour's luminance, by Rec. 709's weights.
fn luminance(colour: vec3<f32>) -> f32 {
  return dot(colour, vec3<f32>(0.2126, 0.7152, 0.0722));
}

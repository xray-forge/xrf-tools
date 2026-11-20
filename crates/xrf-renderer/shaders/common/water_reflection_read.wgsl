// The enhanced water's reflection read only where water drew it. Its targets hold nothing, alpha nought, where no
// water stands, which is under whatever stands in the water too; a filtered read beside a reed or a pipe would mix
// that nothing in as black and draw a dark band about it. Engines' reflection filters weigh only their valid samples
// the same way.

// The reflection at `at` filtered as a linear sampler would, the texels where no water drew left out and the rest
// weighed back to a whole: its colour, and in `w` the share of the filter that was water, nought where none was.
fn read_water_reflection(reflection: texture_2d<f32>, at: vec2<f32>) -> vec4<f32> {
  let size: vec2<f32> = vec2<f32>(textureDimensions(reflection));
  let place: vec2<f32> = at * size - 0.5;
  let corner: vec2<f32> = floor(place);
  let blend: vec2<f32> = place - corner;
  let last: vec2<i32> = vec2<i32>(size) - 1;
  var sum: vec3<f32> = vec3<f32>(0.0);
  var weight: f32 = 0.0;

  for (var y: i32 = 0; y < 2; y++) {
    for (var x: i32 = 0; x < 2; x++) {
      let texel: vec4<f32> = textureLoad(reflection, clamp(vec2<i32>(corner) + vec2<i32>(x, y), vec2<i32>(0), last), 0);
      let share: f32 = select(1.0 - blend.x, blend.x, x == 1) * select(1.0 - blend.y, blend.y, y == 1) *
        f32(texel.a > 0.0);

      sum += texel.rgb * share;
      weight += share;
    }
  }

  return vec4<f32>(sum / max(weight, 1e-6), weight);
}

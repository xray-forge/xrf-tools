// A unit normal folded onto two channels: the octahedron's faces unfolded onto a square.

fn octahedral_wrap(value: vec2<f32>) -> vec2<f32> {
  return (1.0 - abs(value.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), value >= vec2<f32>(0.0));
}

fn octahedral_encode(normal: vec3<f32>) -> vec2<f32> {
  let folded: vec3<f32> = normal / (abs(normal.x) + abs(normal.y) + abs(normal.z));

  return select(octahedral_wrap(folded.xy), folded.xy, folded.z >= 0.0);
}

fn octahedral_decode(encoded: vec2<f32>) -> vec3<f32> {
  var normal: vec3<f32> = vec3<f32>(encoded, 1.0 - abs(encoded.x) - abs(encoded.y));
  let fold: f32 = saturate(-normal.z);

  normal.x += select(fold, -fold, normal.x >= 0.0);
  normal.y += select(fold, -fold, normal.y >= 0.0);

  return normalize(normal);
}

// A static vertex as its layout's arena holds it (`StaticLayout`), eight words: its position first, three plain
// floats, where an acceleration structure reads it in place; its binormal, normal and tangent as `D3DCOLOR`s; its base
// coordinate; and a word its layout gives the meaning of: a sector's lightmap coordinate or its water's colour, a
// tree's rigidity, a spawned model's coordinate's second float.
const VERTEX_WORDS: u32 = 8u;
const VERTEX_POSITION: u32 = 0u;
const VERTEX_BINORMAL: u32 = 3u;
const VERTEX_NORMAL: u32 = 4u;
const VERTEX_TANGENT: u32 = 5u;
const VERTEX_UV: u32 = 6u;
const VERTEX_EXTRA: u32 = 7u;

// Two shorts from a word, the low half first, sign extended.
fn unpack_shorts(word: u32) -> vec2<f32> {
  return vec2<f32>(f32(bitcast<i32>(word << 16u) >> 16u), f32(bitcast<i32>(word) >> 16u));
}

// A direction packed as a `D3DCOLOR`: blue, green, red bytes as z, y, x.
fn unpack_direction(packed: vec4<f32>) -> vec3<f32> {
  return packed.zyx * 2.0 - 1.0;
}

// A sector's base coordinate: two shorts over 1024, their fraction riding in the tangent's and binormal's fourth
// bytes.
fn baked_uv(word: u32, tangent: vec4<f32>, binormal: vec4<f32>) -> vec2<f32> {
  return (unpack_shorts(word) + vec2<f32>(tangent.w, binormal.w)) / 1024.0;
}

// A sector's lightmap coordinate: two shorts over 32768.
fn baked_lightmap_uv(word: u32) -> vec2<f32> {
  return unpack_shorts(word) / 32768.0;
}

// A position from its three words.
fn vertex_position(x: u32, y: u32, z: u32) -> vec3<f32> {
  return vec3<f32>(bitcast<f32>(x), bitcast<f32>(y), bitcast<f32>(z));
}

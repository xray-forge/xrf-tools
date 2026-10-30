#import "common/octahedral"
#import "common/impostor_frames"
#import "generated/static/impostor_bake"

// Bakes a tree type's frames, drawn into a G-buffer of `IMPOSTOR_FRAMES` × `IMPOSTOR_FRAMES` square frames by the
// static scene's own shaders, into its atlases (`ImpostorAtlas`), in three steps:
// - `cs_pack` lays the frames into three maps at the same texels: colour, and coverage where the frame's depth was
//   written; the normal turned from the frame's view into the type's own space, octahedral; and the gloss, the
//   hemisphere term, the material slice and the depth across the type's sphere (one at its front, nought at its back).
//   A texel nothing covers is clear in all three.
// - `cs_reduce` halves a level into the next, each map weighed by its texels' coverage, so an edge keeps its colour
//   rather than darkening towards the clear texels around it.
// - `cs_encode` compresses a level into blocks: the colour and coverage, and the gloss, hemisphere, slice and depth, as
//   BC3 (a BC1 colour block under a BC4 alpha block); the normal as BC5 (two BC4 blocks). A block's endpoints are taken
//   over its covered texels alone, which are all that is drawn.

// The share of coverage below which a texel counts as clear.
const IMPOSTOR_CLEAR: f32 = 0.5 / 255.0;

@compute @workgroup_size(8, 8)
fn cs_pack(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec2<u32> = textureDimensions(colour_level);

  if (any(id.xy >= size)) {
    return;
  }

  let depth: f32 = textureLoad(depth_target, id.xy, 0);

  if (depth <= 0.0) {
    textureStore(colour_level, id.xy, vec4<f32>(0.0));
    textureStore(normal_level, id.xy, vec4<f32>(0.0));
    textureStore(material_level, id.xy, vec4<f32>(0.0));

    return;
  }

  let frame: vec2<u32> = min(id.xy / (size / IMPOSTOR_FRAMES), vec2<u32>(IMPOSTOR_FRAMES - 1u));
  let direction: vec3<f32> = impostor_frame_direction(frame);
  let basis: mat2x3<f32> = impostor_frame_basis(direction);
  let albedo: vec4<f32> = textureLoad(albedo_target, id.xy, 0);
  let material: vec4<f32> = textureLoad(material_target, id.xy, 0);
  // The view space a frame's camera saw: right, up, and towards the camera, which is the frame's direction.
  let seen: vec3<f32> = octahedral_decode(textureLoad(normal_target, id.xy, 0).xy);
  let normal: vec3<f32> = normalize(basis[0] * seen.x + basis[1] * seen.y + direction * seen.z);

  textureStore(colour_level, id.xy, vec4<f32>(albedo.rgb, 1.0));
  textureStore(normal_level, id.xy, vec4<f32>(octahedral_encode(normal) * 0.5 + 0.5, 0.0, 1.0));
  textureStore(material_level, id.xy, vec4<f32>(albedo.a, material.x, material.z, depth));
}

@compute @workgroup_size(8, 8)
fn cs_reduce(@builtin(global_invocation_id) id: vec3<u32>) {
  let size: vec2<u32> = textureDimensions(colour_level);

  if (any(id.xy >= size)) {
    return;
  }

  let last: vec2<u32> = textureDimensions(colour_source) - 1u;
  var coverage: f32 = 0.0;
  var colour: vec3<f32> = vec3<f32>(0.0);
  var normal: vec3<f32> = vec3<f32>(0.0);
  var material: vec4<f32> = vec4<f32>(0.0);

  for (var texel: u32 = 0u; texel < 4u; texel++) {
    let at: vec2<u32> = min(id.xy * 2u + vec2<u32>(texel & 1u, texel >> 1u), last);
    let source: vec4<f32> = textureLoad(colour_source, at, 0);

    coverage += source.a;
    colour += source.rgb * source.a;
    normal += octahedral_decode(textureLoad(normal_source, at, 0).xy * 2.0 - 1.0) * source.a;
    material += textureLoad(material_source, at, 0) * source.a;
  }

  if (coverage <= IMPOSTOR_CLEAR) {
    textureStore(colour_level, id.xy, vec4<f32>(0.0));
    textureStore(normal_level, id.xy, vec4<f32>(0.0));
    textureStore(material_level, id.xy, vec4<f32>(0.0));

    return;
  }

  let encoded: vec2<f32> = octahedral_encode(normalize(normal + vec3<f32>(0.0, 0.0, 1e-6))) * 0.5 + 0.5;

  textureStore(colour_level, id.xy, vec4<f32>(colour / coverage, coverage * 0.25));
  textureStore(normal_level, id.xy, vec4<f32>(encoded, 0.0, 1.0));
  textureStore(material_level, id.xy, material / coverage);
}

// A BC4 block of 16 values: its two endpoints in its first word's low bytes, the brighter first so the block keeps
// eight steps between them, then a three-bit index a texel. Clear texels take any index.
fn encode_bc4(values: array<f32, 16>, covered: array<bool, 16>) -> vec2<u32> {
  var low: f32 = 1.0;
  var high: f32 = 0.0;

  for (var texel: u32 = 0u; texel < 16u; texel++) {
    if (covered[texel]) {
      low = min(low, values[texel]);
      high = max(high, values[texel]);
    }
  }

  high = max(high, low);

  let first: u32 = u32(round(high * 255.0));
  let second: u32 = u32(round(low * 255.0));
  let span: f32 = max(f32(first) - f32(second), 1.0);
  var words: vec2<u32> = vec2<u32>(first | (second << 8u), 0u);

  for (var texel: u32 = 0u; texel < 16u; texel++) {
    // The step from the second endpoint up, then its index: seven is the first, nought the second, the rest between.
    let step: u32 = u32(clamp(round((values[texel] * 255.0 - f32(second)) / span * 7.0), 0.0, 7.0));
    let index: u32 = select(select(8u - step, 1u, step == 0u), 0u, step == 7u || first == second);
    let bit: u32 = 16u + texel * 3u;

    if (bit < 32u) {
      words.x |= index << bit;

      if (bit > 29u) {
        words.y |= index >> (32u - bit);
      }
    } else {
      words.y |= index << (bit - 32u);
    }
  }

  return words;
}

// A colour as BC1 keeps it, five, six and five bits.
fn pack_565(colour: vec3<f32>) -> u32 {
  let scaled: vec3<u32> = vec3<u32>(round(saturate(colour) * vec3<f32>(31.0, 63.0, 31.0)));

  return (scaled.r << 11u) | (scaled.g << 5u) | scaled.b;
}

fn unpack_565(packed: u32) -> vec3<f32> {
  return vec3<f32>(f32((packed >> 11u) & 31u), f32((packed >> 5u) & 63u), f32(packed & 31u)) /
    vec3<f32>(31.0, 63.0, 31.0);
}

// A BC1 block of 16 colours: the covered colours' box, inset by a sixteenth, as its endpoints, the larger first so it
// keeps four colours, then a two-bit index a texel to the nearest of them.
fn encode_bc1(colours: array<vec3<f32>, 16>, covered: array<bool, 16>) -> vec2<u32> {
  var low: vec3<f32> = vec3<f32>(1.0);
  var high: vec3<f32> = vec3<f32>(0.0);

  for (var texel: u32 = 0u; texel < 16u; texel++) {
    if (covered[texel]) {
      low = min(low, colours[texel]);
      high = max(high, colours[texel]);
    }
  }

  high = max(high, low);

  let inset: vec3<f32> = (high - low) / 16.0;
  var first: u32 = pack_565(high - inset);
  var second: u32 = pack_565(low + inset);

  if (first < second) {
    let swap: u32 = first;

    first = second;
    second = swap;
  }

  if (first == second) {
    return vec2<u32>(first | (second << 16u), 0u);
  }

  let a: vec3<f32> = unpack_565(first);
  let b: vec3<f32> = unpack_565(second);
  let palette: array<vec3<f32>, 4> = array<vec3<f32>, 4>(a, b, (2.0 * a + b) / 3.0, (a + 2.0 * b) / 3.0);
  var indices: u32 = 0u;

  for (var texel: u32 = 0u; texel < 16u; texel++) {
    var best: u32 = 0u;
    var nearest: f32 = 1e9;

    for (var entry: u32 = 0u; entry < 4u; entry++) {
      let off: vec3<f32> = colours[texel] - palette[entry];
      let distance: f32 = dot(off, off);

      if (distance < nearest) {
        nearest = distance;
        best = entry;
      }
    }

    indices |= best << (texel * 2u);
  }

  return vec2<u32>(first | (second << 16u), indices);
}

@compute @workgroup_size(8, 8)
fn cs_encode(@builtin(global_invocation_id) id: vec3<u32>) {
  let blocks_across: vec2<u32> = settings.size.yz;

  if (any(id.xy >= blocks_across)) {
    return;
  }

  let last: vec2<u32> = vec2<u32>(settings.size.x - 1u);
  var colours: array<vec3<f32>, 16>;
  var coverage: array<f32, 16>;
  var covered: array<bool, 16>;
  var normal_x: array<f32, 16>;
  var normal_y: array<f32, 16>;
  var surfaces: array<vec3<f32>, 16>;
  var depths: array<f32, 16>;

  for (var texel: u32 = 0u; texel < 16u; texel++) {
    let at: vec2<u32> = min(id.xy * 4u + vec2<u32>(texel & 3u, texel >> 2u), last);
    let colour: vec4<f32> = textureLoad(colour_source, at, 0);
    let normal: vec4<f32> = textureLoad(normal_source, at, 0);
    let material: vec4<f32> = textureLoad(material_source, at, 0);

    colours[texel] = colour.rgb;
    coverage[texel] = colour.a;
    covered[texel] = colour.a > IMPOSTOR_CLEAR;
    normal_x[texel] = normal.x;
    normal_y[texel] = normal.y;
    surfaces[texel] = material.rgb;
    depths[texel] = material.a;
  }

  // Coverage's own block weighs every texel, the clear ones too, which are what it records.
  var every: array<bool, 16>;

  for (var texel: u32 = 0u; texel < 16u; texel++) {
    every[texel] = true;
  }

  let block: u32 = id.y * settings.size.w + id.x;

  blocks[settings.starts.x + block] = vec4<u32>(encode_bc4(coverage, every), encode_bc1(colours, covered));
  blocks[settings.starts.y + block] = vec4<u32>(encode_bc4(normal_x, covered), encode_bc4(normal_y, covered));
  blocks[settings.starts.z + block] = vec4<u32>(encode_bc4(depths, covered), encode_bc1(surfaces, covered));
}

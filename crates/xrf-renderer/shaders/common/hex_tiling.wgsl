// Hex tiling (Mikkelsen, "Practical Real-Time Hex-Tiling", JCGT 11(3), 2022, and its sample code `hextile-demo`, MIT):
// a texture read in three copies, each turned and shifted at random about a corner of the triangle of a hexagon grid the
// coordinate falls in, and blended by how near each corner lies, so no repeat of the texture lines up with the next.
// The grid, its corners' centres and the blend's weights are the demo's; each corner's turn and shift come from
// Hoskins's hash (`hash22`) rather than the demo's sine, whose randomness fades far from the origin, where a terrain's
// detail coordinates run to the hundreds.

#import "common/noise"

// How far each copy's own brightness weighs in the blend, so where they meet the brighter one shows through rather than
// both at half: the demo's `g_fallOffContrast`. How sharply they meet is the caller's (`hex_weights`).
const HEX_BRIGHTNESS_SHARE: f32 = 0.6;

// The grid's scale, `2√3`, and its skew onto equilateral triangles and back, the demo's `gridToSkewedGrid` and
// `invSkewMat`.
const HEX_GRID_SCALE: f32 = 3.4641016;
const HEX_SKEW: vec2<f32> = vec2<f32>(0.57735027, 1.15470054);
const HEX_UNSKEW: vec2<f32> = vec2<f32>(0.5, 0.8660254);

const HEX_PI: f32 = 3.14159265;

// One copy a coordinate is read in: a coordinate `uv` lands at `turn · uv + shift`, the turn its cosine and sine.
struct HexCopy {
  turn: vec2<f32>,
  shift: vec2<f32>,
};

// The three copies about a coordinate, and how near each one's corner lies, the triangle's barycentric weights.
struct HexTiles {
  first: HexCopy,
  second: HexCopy,
  third: HexCopy,
  nearness: vec3<f32>,
};

// The copy that leaves a coordinate where it is.
fn hex_unmoved() -> HexCopy {
  return HexCopy(vec2<f32>(1.0, 0.0), vec2<f32>(0.0));
}

// The copies about `uv`, on a grid of `rate` hexagons to the demo's one, each turned by up to `turning` of a half turn
// either way: nought keeps every copy upright, as a texture with a direction to it needs.
fn hex_tiles(uv: vec2<f32>, rate: f32, turning: f32) -> HexTiles {
  let grid: vec2<f32> = uv * (rate * HEX_GRID_SCALE);
  let skewed: vec2<f32> = vec2<f32>(grid.x - HEX_SKEW.x * grid.y, HEX_SKEW.y * grid.y);
  let base: vec2<f32> = floor(skewed);
  let within: vec2<f32> = skewed - base;
  let rest: f32 = 1.0 - within.x - within.y;
  // The triangle's upper half where the rest falls under nought: its corners and weights mirrored.
  let upper: f32 = select(0.0, 1.0, rest <= 0.0);
  let sign: f32 = 2.0 * upper - 1.0;

  return HexTiles(
    hex_copy_at(base + vec2<f32>(upper, upper), rate, turning),
    hex_copy_at(base + vec2<f32>(upper, 1.0 - upper), rate, turning),
    hex_copy_at(base + vec2<f32>(1.0 - upper, upper), rate, turning),
    vec3<f32>(-rest * sign, upper - within.y * sign, upper - within.x * sign),
  );
}

// The copy a corner of the grid draws: turned about the corner's centre and shifted, both its own at random.
fn hex_copy_at(corner: vec2<f32>, rate: f32, turning: f32) -> HexCopy {
  let centre: vec2<f32> = vec2<f32>(corner.x + HEX_UNSKEW.x * corner.y, HEX_UNSKEW.y * corner.y) /
    (rate * HEX_GRID_SCALE);
  let angle: f32 = (hash22(corner + vec2<f32>(37.0, 17.0)).x * 2.0 - 1.0) * HEX_PI * turning;
  let turn: vec2<f32> = vec2<f32>(cos(angle), sin(angle));

  return HexCopy(turn, centre + hash22(corner) - hex_turned(turn, centre));
}

// A direction turned as a copy turns its coordinates, and back.
fn hex_turned(turn: vec2<f32>, direction: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(turn.x * direction.x - turn.y * direction.y, turn.y * direction.x + turn.x * direction.y);
}

fn hex_unturned(turn: vec2<f32>, direction: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(turn.x * direction.x + turn.y * direction.y, turn.x * direction.y - turn.y * direction.x);
}

// Where a copy reads a coordinate, which way it reads a direction (a texture's derivatives, a parallax march's
// reach), and a direction read in it as the surface has it (a normal's tangent part, how far a march moved).
fn hex_at(copy: HexCopy, uv: vec2<f32>) -> vec2<f32> {
  return hex_turned(copy.turn, uv) + copy.shift;
}

fn hex_along(copy: HexCopy, direction: vec2<f32>) -> vec2<f32> {
  return hex_turned(copy.turn, direction);
}

fn hex_back(copy: HexCopy, direction: vec2<f32>) -> vec2<f32> {
  return hex_unturned(copy.turn, direction);
}

// One of the three copies, by its index.
fn hex_copy(tiles: HexTiles, index: u32) -> HexCopy {
  switch index {
    case 0u: {
      return tiles.first;
    }
    case 1u: {
      return tiles.second;
    }
    default: {
      return tiles.third;
    }
  }
}

// The copy whose corner lies nearest, which shows alone across most of a hexagon.
fn hex_nearest(tiles: HexTiles) -> u32 {
  let near: vec3<f32> = tiles.nearness;

  return select(select(2u, 1u, near.y >= near.z), 0u, near.x >= near.y && near.x >= near.z);
}

// How much each copy shows, from how near its corner lies, raised to `sharpness` (the demo's `g_exp`, seven), and how
// bright it reads there (`brightness`, each copy's luminance), summing to one.
fn hex_weights(tiles: HexTiles, brightness: vec3<f32>, sharpness: f32) -> vec3<f32> {
  let shares: vec3<f32> = mix(vec3<f32>(1.0), brightness, HEX_BRIGHTNESS_SHARE) *
    pow(max(tiles.nearness, vec3<f32>(0.0)), vec3<f32>(sharpness));

  return shares / max(shares.x + shares.y + shares.z, 1e-6);
}

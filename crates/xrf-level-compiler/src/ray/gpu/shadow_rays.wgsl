enable wgpu_ray_query;

// How much light each of a batch of shadow rays lets through, as the CPU's `LightPoint` answers it by the batch's
// rules (`RayGpuTracer::trace`): the product over every face of the ray model the ray crosses within its range, from
// its front alone or from either side, each alpha face letting `1 - alpha` or `1 - alpha²` through, and where the rules
// say so a face of the ray's own passed. The structure holds two geometries: the opaque faces, any one of which stops
// the light, so the hardware ends the ray at the first; and the alpha faces, each dimming the light by its texture's
// alpha at the nearest texel to the hit's texture coordinates, wrapped as the SDK wraps them. Faces passing light are
// left out, as they change nothing. Where a ray passes a face of its own, every face is a candidate, an opaque one
// taken unless it is the ray's own; where the rules ask for the CPU's ray test, every face is a candidate too, taken
// only where that test meets it, so grazing crossings the device's watertight test takes count as the CPU counts them.
//
// The rays come as the plans describe them (`LightRayGroup`): a light's packet, made here from its points' origins, a
// workgroup a packet, toward a directional light or a light at a position; and single rays toward lights at
// positions, made on the CPU where a lighter wants them as the CPU makes them to the last bit.

struct Packet {
  // The direction toward a directional light, or a light's position, as `kind` says.
  aim: vec3<f32>,
  lift: f32,
  // Its points by bit, their origins from `first_origin`; its rays' transmittances from `first_ray`.
  facing: array<u32, 8>,
  first_origin: u32,
  first_ray: u32,
  range: f32,
  kind: u32,
}

struct Origin {
  // Where the point's rays start, lifted off it.
  lifted: vec3<f32>,
  // The skip code of the face its rays pass.
  skip: u32,
  point: vec3<f32>,
  _pad: u32,
}

struct Single {
  origin: vec3<f32>,
  range: f32,
  direction: vec3<f32>,
  // Where its transmittance goes.
  index: u32,
  // The skip code of the face it passes.
  skip: u32,
  _pad0: u32,
  _pad1: u32,
  _pad2: u32,
}

struct AlphaFace {
  corners: array<vec2<f32>, 3>,
  texture: u32,
  _pad: u32,
}

struct AlphaTexture {
  // The first of its texels' bytes among `alpha`'s.
  offset: u32,
  width: u32,
  height: u32,
  _pad: u32,
}

struct Batch {
  packets: u32,
  singles: u32,
  // `RULE_*` bits.
  rules: u32,
  _pad: u32,
}

@group(0) @binding(0) var scene: acceleration_structure;
@group(0) @binding(1) var<storage, read> packets: array<Packet>;
@group(0) @binding(2) var<storage, read> origins: array<Origin>;
@group(0) @binding(3) var<storage, read> singles: array<Single>;
// The alpha geometry's faces, by their place in it.
@group(0) @binding(4) var<storage, read> alpha_faces: array<AlphaFace>;
@group(0) @binding(5) var<storage, read> alpha_textures: array<AlphaTexture>;
// Every alpha texture's texels, a byte each, four to a word from its lowest byte, top row first.
@group(0) @binding(6) var<storage, read> alpha: array<u32>;
@group(0) @binding(7) var<storage, read_write> transmittances: array<f32>;
@group(0) @binding(8) var<uniform> batch: Batch;
// The opaque geometry's corners and the alpha geometry's, three floats a corner, nine a face, as the structure holds
// them.
@group(0) @binding(9) var<storage, read> opaque_corners: array<f32>;
@group(0) @binding(10) var<storage, read> alpha_corners: array<f32>;

// Lanes a packet's workgroup runs, a lane a point: room for a slot's 225 (`DetailGpuTracer::PACKET_LANES`); and a
// single rays' workgroup (`DetailGpuTracer::SINGLE_LANES`).
const PACKET_LANES: u32 = 256u;
const SINGLE_LANES: u32 = 64u;

// The batch's rules (`RayGpuTracer::RULE_*`): a face stops a ray from either side, not its front alone; an alpha texel
// lets `1 - alpha²` through, not `1 - alpha`; a ray passes the face its skip code names; a face the device meets counts
// only where the CPU's ray test meets it too.
const RULE_BOTH_SIDES: u32 = 1u;
const RULE_SQUARED_ALPHA: u32 = 2u;
const RULE_SKIPPING: u32 = 4u;
const RULE_CPU_TEST: u32 = 8u;

// The CPU's ray test's least determinant, `xrf_math::EPS`.
const EPS: f32 = 0.00001;

// The kinds of packet (`GpuPacket::KIND_*`): a directional light's, and a light at a position's.
const KIND_DIRECTIONAL: u32 = 0u;
const KIND_LAMP: u32 = 1u;

// `Fvector::normalize_safe`'s least squared length, `f32::MIN_POSITIVE`.
const LEAST_SQUARED_LENGTH: f32 = 1.17549435e-38;

// A skip code's bit naming the alpha geometry, and the code of no face.
const ALPHA_GEOMETRY: u32 = 0x80000000u;
const NO_SKIP: u32 = 0xffffffffu;

// The texel `coordinate` rounds to along a side of `size` texels, wrapped as the SDK takes an `int`'s remainder by an
// unsigned size (`ShadowTexture::get_texel`).
fn texel(coordinate: f32, size: u32) -> u32 {
  return bitcast<u32>(i32(floor(coordinate * f32(size) + 0.5))) % size;
}

// How much light an alpha face lets through at barycentrics `hit`, weighed `(1 - u - v, u, v)` as the SDK weighs them.
fn alpha_transmittance(face: AlphaFace, hit: vec2<f32>) -> f32 {
  let weight: f32 = 1.0 - hit.x - hit.y;
  let coordinates: vec2<f32> = face.corners[0] * weight + face.corners[1] * hit.x + face.corners[2] * hit.y;
  let texture: AlphaTexture = alpha_textures[face.texture];
  let index: u32 = texture.offset + texel(coordinates.y, texture.height) * texture.width +
    texel(coordinates.x, texture.width);
  let opacity: f32 = f32((alpha[index / 4u] >> ((index % 4u) * 8u)) & 0xffu) / 255.0;

  return select(1.0 - opacity, 1.0 - opacity * opacity, (batch.rules & RULE_SQUARED_ALPHA) != 0u);
}

// Corner `corner` of face `face` of the alpha geometry or the opaque one.
fn get_corner(is_alpha: bool, face: u32, corner: u32) -> vec3<f32> {
  let at: u32 = face * 9u + corner * 3u;

  if (is_alpha) {
    return vec3<f32>(alpha_corners[at], alpha_corners[at + 1u], alpha_corners[at + 2u]);
  }

  return vec3<f32>(opaque_corners[at], opaque_corners[at + 1u], opaque_corners[at + 2u]);
}

// Where the CPU's two-sided ray test (`Triangle::cross`, CDB's `TestRayTri`) meets a face, its arithmetic in the same
// order: its barycentrics in `xy`, and 1 in `z` where it meets the face within `range`, 0 where it does not.
fn cross_face(is_alpha: bool, face: u32, origin: vec3<f32>, direction: vec3<f32>, range: f32) -> vec3<f32> {
  let first: vec3<f32> = get_corner(is_alpha, face, 0u);
  let edge1: vec3<f32> = get_corner(is_alpha, face, 1u) - first;
  let edge2: vec3<f32> = get_corner(is_alpha, face, 2u) - first;
  let pvec: vec3<f32> = cross(direction, edge2);
  let det: f32 = dot(edge1, pvec);

  if (det > -EPS && det < EPS) {
    return vec3<f32>(0.0);
  }

  let inverse: f32 = 1.0 / det;
  let tvec: vec3<f32> = origin - first;
  let u: f32 = dot(tvec, pvec) * inverse;

  if (!(u >= 0.0 && u <= 1.0)) {
    return vec3<f32>(0.0);
  }

  let qvec: vec3<f32> = cross(tvec, edge1);
  let v: f32 = dot(direction, qvec) * inverse;

  if (v < 0.0 || u + v > 1.0) {
    return vec3<f32>(0.0);
  }

  let distance: f32 = dot(edge2, qvec) * inverse;

  return vec3<f32>(u, v, select(0.0, 1.0, distance > 0.0 && distance <= range));
}

// How much light passes from `origin` along `direction` within `range`, the face of skip code `skip` passed where the
// rules pass a ray's own.
fn trace(origin: vec3<f32>, direction: vec3<f32>, range: f32, skip: u32) -> f32 {
  let is_skipping: bool = (batch.rules & RULE_SKIPPING) != 0u && skip != NO_SKIP;
  let is_cpu_test: bool = (batch.rules & RULE_CPU_TEST) != 0u;
  // The face the SDK's culled ray test keeps is the side its corners' normal, `(p1 - p0) x (p2 - p0)`, points to,
  // which the device names its front; the first opaque face met ends the ray, its light gone whatever else it crosses.
  var flags: u32 = RAY_FLAG_TERMINATE_ON_FIRST_HIT;

  if ((batch.rules & RULE_BOTH_SIDES) == 0u) {
    flags |= RAY_FLAG_CULL_BACK_FACING;
  }

  if (is_skipping || is_cpu_test) {
    flags |= RAY_FLAG_FORCE_NO_OPAQUE;
  }

  var transmittance: f32 = 1.0;
  var query: ray_query;

  rayQueryInitialize(&query, scene, RayDesc(flags, 0xffu, 0.0, range, origin, direction));

  // The alpha geometry's faces are candidates, and where a face is passed or tested again the opaque ones too, taken
  // but for those; else the hardware commits the opaque ones itself.
  while (rayQueryProceed(&query)) {
    let candidate: RayIntersection = rayQueryGetCandidateIntersection(&query);
    let is_alpha: bool = candidate.geometry_index != 0u;
    let code: u32 = select(candidate.primitive_index, ALPHA_GEOMETRY | candidate.primitive_index, is_alpha);

    if (is_skipping && code == skip) {
      continue;
    }

    var barycentrics: vec2<f32> = candidate.barycentrics;

    if (is_cpu_test) {
      let crossing: vec3<f32> = cross_face(is_alpha, candidate.primitive_index, origin, direction, range);

      if (crossing.z == 0.0) {
        continue;
      }

      barycentrics = crossing.xy;
    }

    if (!is_alpha) {
      rayQueryConfirmIntersection(&query);
      continue;
    }

    transmittance *= alpha_transmittance(alpha_faces[candidate.primitive_index], barycentrics);
  }

  if (rayQueryGetCommittedIntersection(&query).kind != RAY_QUERY_INTERSECTION_NONE) {
    return 0.0;
  }

  return transmittance;
}

@compute @workgroup_size(PACKET_LANES)
fn cs_packets(@builtin(workgroup_id) group: vec3<u32>, @builtin(local_invocation_index) point: u32) {
  if (group.x >= batch.packets) {
    return;
  }

  let packet: Packet = packets[group.x];
  let word: u32 = point / 32u;
  let bit: u32 = 1u << (point % 32u);

  if (word >= 8u || (packet.facing[word] & bit) == 0u) {
    return;
  }

  // The ray's place among the packet's: how many facing points come before this one.
  var rank: u32 = countOneBits(packet.facing[word] & (bit - 1u));

  for (var before: u32 = 0u; before < word; before++) {
    rank += countOneBits(packet.facing[before]);
  }

  let start: Origin = origins[packet.first_origin + point];

  if (packet.kind == KIND_LAMP) {
    // Toward the light and as far as it, as `LightPoint::get_lamp_ray` makes the ray.
    let toward: vec3<f32> = packet.aim - start.point;
    let distance_squared: f32 = dot(toward, toward);
    let direction: vec3<f32> = select(toward, toward * sqrt(1.0 / distance_squared),
      distance_squared > LEAST_SQUARED_LENGTH);

    transmittances[packet.first_ray + rank] = trace(start.lifted, direction, sqrt(distance_squared), start.skip);
    return;
  }

  // No lift leaves the origin's own floats, as the CPU makes it.
  let origin: vec3<f32> = select(start.lifted + packet.aim * packet.lift, start.lifted, packet.lift == 0.0);

  transmittances[packet.first_ray + rank] = trace(origin, packet.aim, packet.range, start.skip);
}

@compute @workgroup_size(SINGLE_LANES)
fn cs_singles(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= batch.singles) {
    return;
  }

  let single: Single = singles[id.x];

  transmittances[single.index] = trace(single.origin, single.direction, single.range, single.skip);
}

enable wgpu_ray_query;

// How much light each of a batch of shadow rays lets through, as the detail lighter's CPU trace answers it
// (`DetailSlotLighter::trace`, `ShadowMaterials::get_transmittance`): the product over every face of the ray model the
// ray crosses from its front within its range. The structure holds two geometries: the opaque faces, any one of which
// stops the light, so the hardware takes them alone and ends the ray at the first; and the alpha faces, each dimming
// the light by its texture's alpha at the nearest texel to the hit's texture coordinates, wrapped as the SDK wraps them.
// Faces passing light are left out, as they change nothing.
//
// The rays come as the plans describe them (`DetailRayGroup`): a directional light's packet, made here from its
// slot's points' origins, a workgroup a packet; and single rays toward lights at positions, made on the CPU.

struct Packet {
  toward: vec3<f32>,
  lift: f32,
  // Its points by bit, their origins from `first_origin`; its rays' transmittances from `first_ray`.
  facing: array<u32, 8>,
  first_origin: u32,
  first_ray: u32,
  range: f32,
  _pad: u32,
}

struct Single {
  origin: vec3<f32>,
  range: f32,
  direction: vec3<f32>,
  // Where its transmittance goes.
  index: u32,
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
  _pad0: u32,
  _pad1: u32,
}

@group(0) @binding(0) var scene: acceleration_structure;
@group(0) @binding(1) var<storage, read> packets: array<Packet>;
@group(0) @binding(2) var<storage, read> origins: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> singles: array<Single>;
// The alpha geometry's faces, by their place in it.
@group(0) @binding(4) var<storage, read> alpha_faces: array<AlphaFace>;
@group(0) @binding(5) var<storage, read> alpha_textures: array<AlphaTexture>;
// Every alpha texture's texels, a byte each, four to a word from its lowest byte, top row first.
@group(0) @binding(6) var<storage, read> alpha: array<u32>;
@group(0) @binding(7) var<storage, read_write> transmittances: array<f32>;
@group(0) @binding(8) var<uniform> batch: Batch;

// Lanes a packet's workgroup runs, a lane a point: room for a slot's 225 (`DetailGpuTracer::PACKET_LANES`); and a
// single rays' workgroup (`DetailGpuTracer::SINGLE_LANES`).
const PACKET_LANES: u32 = 256u;
const SINGLE_LANES: u32 = 64u;

// The face the SDK's culled ray test keeps: the side its corners' normal, `(p1 - p0) x (p2 - p0)`, points to, which
// the device names its front; and the first opaque face met ends the ray, its light gone whatever else it crosses.
const FLAGS: u32 = RAY_FLAG_CULL_BACK_FACING | RAY_FLAG_TERMINATE_ON_FIRST_HIT;

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
  let value: u32 = (alpha[index / 4u] >> ((index % 4u) * 8u)) & 0xffu;

  return 1.0 - f32(value) / 255.0;
}

// How much light passes from `origin` along `direction` within `range`.
fn trace(origin: vec3<f32>, direction: vec3<f32>, range: f32) -> f32 {
  var transmittance: f32 = 1.0;
  var query: ray_query;

  rayQueryInitialize(&query, scene, RayDesc(FLAGS, 0xffu, 0.0, range, origin, direction));

  // Only the alpha geometry's faces are candidates; the opaque ones the hardware commits itself.
  while (rayQueryProceed(&query)) {
    let candidate: RayIntersection = rayQueryGetCandidateIntersection(&query);

    transmittance *= alpha_transmittance(alpha_faces[candidate.primitive_index], candidate.barycentrics);
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

  let point_origin: vec3<f32> = origins[packet.first_origin + point].xyz;
  // No lift leaves the origin's own floats, as the CPU makes it.
  let origin: vec3<f32> = select(point_origin + packet.toward * packet.lift, point_origin, packet.lift == 0.0);

  transmittances[packet.first_ray + rank] = trace(origin, packet.toward, packet.range);
}

@compute @workgroup_size(SINGLE_LANES)
fn cs_singles(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= batch.singles) {
    return;
  }

  let single: Single = singles[id.x];

  transmittances[single.index] = trace(single.origin, single.direction, single.range);
}

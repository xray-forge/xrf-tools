use glam::Vec3A;
use obvhs::bvh2::node::Bvh2Node;

use crate::ray::facing::Facing;
use crate::ray::ray::Ray;
use crate::ray::slab_ray::SlabRay;
use crate::tracer::bvh_tracer::BvhTracer;
use crate::tracer::trace_hit::TraceHit;

/// Which of a packet's rays a node is walked for, a bit each.
type PacketMask = [u64; BvhTracer::MASK_WORDS];

impl BvhTracer {
  /// One packet of at most [`Self::PACKET_RAYS`] rays through the hierarchy: each node is tested once for the packet,
  /// then for each ray still active in it. When every ray shares one direction, as a directional light's do, the packet
  /// is first tested as one ray from the centre of its origins against the node grown by their spread, which no ray can
  /// reach the node without passing.
  pub(super) fn trace_packet(&self, rays: &[Ray], mut visit: impl FnMut(usize, TraceHit)) {
    debug_assert!(
      rays.len() <= Self::PACKET_RAYS,
      "a packet holds at most {} rays",
      Self::PACKET_RAYS
    );

    if self.nodes.is_empty() || rays.is_empty() {
      return;
    }

    let slabs: Vec<SlabRay> = rays.iter().map(|ray| SlabRay::new(ray.origin, ray.direction)).collect();
    let (low, high) = slabs
      .iter()
      .fold((Vec3A::INFINITY, Vec3A::NEG_INFINITY), |(low, high), slab| {
        (low.min(slab.origin), high.max(slab.origin))
      });
    let spread: Vec3A = (high - low) * 0.5;
    let is_shared: bool = rays.iter().all(|ray| ray.direction == rays[0].direction);
    let centre: SlabRay = SlabRay {
      origin: (low + high) * 0.5,
      inverse: slabs[0].inverse,
    };
    let reach: f32 = rays.iter().map(|ray| ray.range).fold(0.0, f32::max);
    let mut full: PacketMask = [0; Self::MASK_WORDS];

    for index in 0..rays.len() {
      full[index / 64] |= 1 << (index % 64);
    }

    let mut stack: Vec<PacketVisit> = vec![PacketVisit { node: 0, mask: full }];

    while let Some(PacketVisit { node: at, mask }) = stack.pop() {
      let node: &Bvh2Node = &self.nodes[at as usize];
      let (node_low, node_high) = (node.aabb.min, node.aabb.max);

      if is_shared && !centre.reaches(reach, node_low - spread, node_high + spread) {
        continue;
      }

      let mut active: PacketMask = [0; Self::MASK_WORDS];
      let mut is_any: bool = false;

      for (word, bits) in mask.iter().enumerate() {
        let mut bits: u64 = *bits;

        while bits != 0 {
          let index: usize = word * 64 + bits.trailing_zeros() as usize;

          bits &= bits - 1;

          if slabs[index].reaches(rays[index].range, node_low, node_high) {
            active[word] |= 1 << (index % 64);
            is_any = true;
          }
        }
      }

      if !is_any {
        continue;
      }

      if node.is_leaf() {
        for face in &self.faces[node.first_index as usize..(node.first_index + node.prim_count) as usize] {
          for (word, bits) in active.iter().enumerate() {
            let mut bits: u64 = *bits;

            while bits != 0 {
              let index: usize = word * 64 + bits.trailing_zeros() as usize;

              bits &= bits - 1;

              if let Some(crossing) = self.triangles[*face as usize].cross(&rays[index], Facing::Front) {
                visit(index, TraceHit::new(*face, crossing));
              }
            }
          }
        }
      } else {
        stack.push(PacketVisit {
          node: node.first_index + 1,
          mask: active,
        });
        stack.push(PacketVisit {
          node: node.first_index,
          mask: active,
        });
      }
    }
  }
}

/// A node still to be walked, for the packet's rays its mask names.
#[derive(Clone, Copy, Debug)]
struct PacketVisit {
  node: u32,
  mask: PacketMask,
}

//! Where a described pack waits for the read that serves it, which is one entry per read because reads overlap.

use std::str::FromStr;
use std::sync::{Arc, Mutex};

use xrf_visual::{SectorDescription, SectorGeometry, VisualSection};

use crate::core::session::SessionId;
use crate::plugins::levels::state::{PackedSector, PackedSectors};

fn new_session(at: u8) -> SessionId {
  SessionId::from_str(&format!("00000000-0000-4000-8000-0000000000{at:02x}")).expect("a session id")
}

fn new_pack(sector: u32, bytes: &[u8]) -> PackedSector {
  PackedSector {
    buffer: Mutex::new(Some(bytes.to_vec())),
    description: SectorDescription {
      bounds: None,
      buffer_length: bytes.len() as u32,
      geometry: SectorGeometry {
        binormals: None,
        index_count: 0,
        indices: VisualSection {
          byte_length: 0,
          byte_offset: 0,
        },
        lightmap_uvs: None,
        normals: None,
        positions: VisualSection {
          byte_length: 0,
          byte_offset: 0,
        },
        tangents: None,
        uv_components: 0,
        uvs: None,
        vertex_count: 0,
      },
      impostors: None,
      instances: Vec::new(),
      sections: Vec::new(),
      sector,
      skipped: Vec::new(),
    },
  }
}

/// The defect this exists for. `open_sector` and `read_sector` are two calls, so a viewer reading three sectors at
/// once has three pairs of them in the air. Published one at a time, the second pack replaced the first and the
/// first read then failed to find the pack it had just been described - which read, to the viewer, as a level that
/// would not load until the camera moved somewhere else.
#[test]
fn serves_each_read_the_pack_it_was_described() {
  let parked: PackedSectors = PackedSectors::new();

  parked.park(new_session(1), new_pack(10, &[1, 2, 3])).expect("parked");
  parked.park(new_session(2), new_pack(20, &[4, 5])).expect("parked");
  parked.park(new_session(3), new_pack(30, &[6])).expect("parked");

  // Out of order, because a pack that took longer is served later than one that took no time at all.
  let second: Arc<PackedSector> = parked.take(new_session(2)).expect("the second pack");
  let first: Arc<PackedSector> = parked.take(new_session(1)).expect("the first pack");
  let third: Arc<PackedSector> = parked.take(new_session(3)).expect("the third pack");

  assert_eq!(first.description.sector, 10);
  assert_eq!(second.description.sector, 20);
  assert_eq!(third.description.sector, 30);
  assert_eq!(first.take_buffer().expect("the first bytes"), vec![1, 2, 3]);
}

#[test]
fn serves_a_pack_once() {
  let parked: PackedSectors = PackedSectors::new();

  parked.park(new_session(1), new_pack(10, &[1])).expect("parked");
  parked.take(new_session(1)).expect("the pack");

  assert!(parked.take(new_session(1)).is_err());
}

#[test]
fn says_so_when_no_pack_answers_to_a_read() {
  assert!(PackedSectors::new().take(new_session(9)).is_err());
}

use xrf_math::Vector3d;
use xrf_math::glam::Vec3;
use xrf_spawn::{Patrol, PatrolPoint, SpawnPatrolsChunk};

use crate::id::game_vertex_id::GameVertexId;
use crate::patrol::game_patrols::GamePatrols;

#[test]
fn keeps_each_patrols_points_in_order_by_name() {
  let point = |name: &str, x: f32| PatrolPoint {
    name: String::from(name),
    position: Vector3d::new(x, 2.0, 3.0),
    flags: 0,
    level_vertex_id: 40,
    game_vertex_id: 7,
  };
  let patrols: GamePatrols = GamePatrols::of_spawn(&SpawnPatrolsChunk {
    patrols: vec![Patrol {
      name: String::from("zat_sim_15_spawn"),
      points: vec![point("wp00", 1.0), point("wp01", 5.0)],
      links: Vec::new(),
    }],
  });

  assert_eq!(patrols.len(), 1);
  assert!(patrols.get("absent").is_none());

  let points = patrols.get("zat_sim_15_spawn").expect("the patrol");

  assert_eq!(points.len(), 2);
  assert_eq!(points[0].name, "wp00");
  assert_eq!(points[0].position, Vec3::new(1.0, 2.0, 3.0));
  assert_eq!(points[0].level_vertex, 40);
  assert_eq!(points[0].game_vertex, GameVertexId(7));
}

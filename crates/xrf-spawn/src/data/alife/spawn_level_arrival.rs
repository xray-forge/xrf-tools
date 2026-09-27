use xrf_math::Vector3d;

use crate::data::alife::inherited::alife_level_changer::AlifeLevelChanger;

/// Where a level changer puts the actor on the level it leads to, which is where the game has a player arrive.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnLevelArrival {
  /// The changer's name, which says where it leads from.
  pub changer: String,
  /// Where the actor stands, in the engine's space.
  pub position: Vector3d<f32>,
  /// The actor's angles there, as `CActor::MoveActor` takes them: pitch, heading and bank.
  pub angles: Vector3d<f32>,
}

impl SpawnLevelArrival {
  /// The arrival a changer makes on a level, when it leads there: by the level its destination vertex stands on, or
  /// by the name it gives where the graph does not hold that vertex. Levels are matched by name without case.
  pub fn of(name: &str, changer: &AlifeLevelChanger, destination: Option<&str>, level: &str) -> Option<Self> {
    destination
      .unwrap_or(&changer.dest_level_name)
      .eq_ignore_ascii_case(level)
      .then(|| Self {
        angles: changer.dest_direction.clone(),
        changer: name.to_owned(),
        position: changer.dest_position.clone(),
      })
  }
}

#[cfg(test)]
mod tests {
  use xrf_math::Vector3d;

  use crate::data::alife::inherited::alife_level_changer::AlifeLevelChanger;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
  use crate::data::alife::spawn_level_arrival::SpawnLevelArrival;

  fn new_changer(dest_level_name: &str) -> AlifeLevelChanger {
    AlifeLevelChanger {
      angle_y: 0.0,
      base: AlifeObjectSpaceRestrictor {
        base: AlifeObjectAbstract {
          custom_data: String::new(),
          direct_control: 0,
          distance: 0.0,
          flags: 0,
          game_vertex_id: 0,
          level_vertex_id: 0,
          spawn_story_id: 0,
          story_id: 0,
        },
        restrictor_type: 0,
        shape: Vec::new(),
      },
      dest_direction: Vector3d::new(0.0, 1.5, 0.0),
      dest_game_vertex_id: 0,
      dest_graph_point: String::new(),
      dest_level_name: String::from(dest_level_name),
      dest_level_vertex_id: 0,
      dest_position: Vector3d::new(10.0, 2.0, -30.0),
      enabled: 1,
      hint: String::new(),
      save_marker: 0,
      silent_mode: 0,
    }
  }

  #[test]
  fn arrives_where_a_changer_leading_to_the_level_puts_the_actor() {
    let arrival: SpawnLevelArrival =
      SpawnLevelArrival::of("zat_to_pripyat", &new_changer("other"), Some("Pripyat"), "pripyat").expect("an arrival");

    assert_eq!(arrival.changer, "zat_to_pripyat");
    assert_eq!(arrival.position, Vector3d::new(10.0, 2.0, -30.0));
    assert_eq!(arrival.angles, Vector3d::new(0.0, 1.5, 0.0));
  }

  #[test]
  fn takes_the_level_its_vertex_stands_on_over_the_name_it_gives() {
    assert!(SpawnLevelArrival::of("changer", &new_changer("pripyat"), Some("zaton"), "pripyat").is_none());
  }

  #[test]
  fn falls_back_to_the_name_it_gives_where_the_graph_holds_no_vertex() {
    assert!(SpawnLevelArrival::of("changer", &new_changer("PRIPYAT"), None, "pripyat").is_some());
    assert!(SpawnLevelArrival::of("changer", &new_changer("zaton"), None, "pripyat").is_none());
  }
}

use std::collections::BTreeMap;

use xrf_game_graph::{GameGraph, GamePatrols, GameVertexId};
use xrf_game_hooks::HookRegistry;
use xrf_game_objects::{
  ObjectCreature, ObjectId, ObjectPlace, ObjectRecord, ObjectRegistry, ObjectSchedule, ObjectScheduleKind,
  ObjectSwitching,
};
use xrf_math::glam::Vec3;
use xrf_random::{RandomAddress, RandomStream};
use xrf_spawn::{AlifeClass, ClsId};

use crate::config::simulation_config::SimulationConfig;
use crate::facts::simulation_facts::SimulationFacts;
use crate::squad::squad_description::SquadDescription;
use crate::start::simulation_place::SimulationPlace;
use crate::start::simulation_start_report::SimulationStartReport;

/// A new game's start squads (`sim_board:fill_start_position`): each start squad created at its smart terrain in the
/// configs' order, its members drawn and created at its spawn point, the squad assigned to the smart. Ids go out as
/// X-Ray hands them: the squad, then its members, then the next squad (`create_squad`, then `create_npc`).
///
/// Every draw comes from the game's seed at the squad's id, so a start is the same for the same seed whatever else
/// draws. Fixed from vanilla: a spawn point naming a patrol the spawn lacks puts the squad at its smart, logged, where
/// vanilla's `patrol(name)` aborts. The condition functions its condlists call are the hook registry's.
pub struct SimulationStart<'a> {
  pub config: &'a SimulationConfig,
  pub graph: &'a GameGraph,
  pub patrols: &'a GamePatrols,
  pub hooks: &'a HookRegistry,
  pub seed: u64,
}

impl SimulationStart<'_> {
  /// Where a squad's members spawn (`spawn_point`), and how many and which random members it draws.
  const SPAWN_POINT_DOMAIN: u64 = RandomAddress::domain("simulation.squad.spawn_point");
  const MEMBERS_DOMAIN: u64 = RandomAddress::domain("simulation.squad.members");

  /// The spawn point naming the smart terrain itself.
  const SPAWN_POINT_SELF: &'static str = "self";

  /// The health a created creature starts with.
  const CREATED_HEALTH: f32 = 1.0;

  /// Creates every start squad in `objects`, answering what it made.
  pub fn create(&self, objects: &mut ObjectRegistry) -> SimulationStartReport {
    let smarts: BTreeMap<String, ObjectId> = objects
      .get_records()
      .iter()
      .filter(|(_, record)| AlifeClass::from_cls_id(&record.class) == AlifeClass::SeSmartTerrain)
      .map(|(id, record)| (record.name.clone(), id))
      .collect();
    let mut report: SimulationStartReport = SimulationStartReport::default();

    for start in self.config.get_start() {
      let Some(smart) = smarts.get(&start.smart).copied() else {
        report.issues.push(format!(
          "Start squad [{}] names smart terrain '{}', which the spawn lacks",
          start.squad, start.smart
        ));
        continue;
      };

      if let Err(issue) = self.create_squad(objects, &start.squad, smart, &mut report) {
        report
          .issues
          .push(format!("Start squad [{}] at '{}' {issue}", start.squad, start.smart));
      }
    }

    report
  }

  /// Creates the squad `section` at `smart` with its members (`sim_board:create_squad`).
  fn create_squad(
    &self,
    objects: &mut ObjectRegistry,
    section: &str,
    smart: ObjectId,
    report: &mut SimulationStartReport,
  ) -> Result<(), String> {
    let squad: &SquadDescription = self
      .config
      .get_squad(section)
      .ok_or_else(|| String::from("has no squad section that reads"))?;
    let class: &ClsId = self
      .config
      .get_class(section)
      .ok_or_else(|| String::from("has no class"))?;
    let smart_place: SimulationPlace =
      Self::find_place(objects, smart).ok_or_else(|| String::from("stands at a smart terrain with no game vertex"))?;
    let id: ObjectId = self
      .create_object(objects, section, class, smart_place)
      .ok_or_else(|| String::from("found no free id"))?;
    // A squad is a squad from its creation, assigned to the smart it is made at, whatever members it comes to have.
    objects.form_group(id, Some(smart));

    let place: SimulationPlace = match self.pick_spawn_point(squad, id) {
      Some(name) if name != Self::SPAWN_POINT_SELF => match self.find_patrol_point(name) {
        Some(place) => place,
        None => {
          report.issues.push(format!(
            "Squad [{section}] spawns at patrol '{name}', which the spawn lacks; it spawns at its smart terrain"
          ));
          smart_place
        }
      },
      _ => smart_place,
    };

    for member in self.draw_members(squad, id) {
      let Some(member_class) = self.config.get_class(member) else {
        continue;
      };
      let Some(member_id) = self.create_object(objects, member, member_class, place) else {
        report
          .issues
          .push(format!("Squad [{section}]'s member [{member}] found no free id"));
        continue;
      };

      objects.set_creature(
        member_id,
        ObjectCreature {
          health: Self::CREATED_HEALTH,
        },
      );
      objects.join_group(id, member_id);
      report.members += 1;
    }

    if objects.get_group_members(id).is_empty() {
      report.issues.push(format!("Squad [{section}] created no member"));
    } else {
      // The squad stands where its first member does (`register_member`).
      objects.move_to(id, self.place_of(place));
    }

    report.squads += 1;

    Ok(())
  }

  /// Creates an object of `section` and `class` at `place`, named and with its schedule row as X-Ray creates one.
  fn create_object(
    &self,
    objects: &mut ObjectRegistry,
    section: &str,
    class: &ClsId,
    place: SimulationPlace,
  ) -> Option<ObjectId> {
    let alife_class: AlifeClass = AlifeClass::from_cls_id(class);
    let kind: Option<ObjectScheduleKind> = ObjectScheduleKind::of_class(alife_class.clone());
    let id: ObjectId = objects.create(
      ObjectRecord {
        section: String::from(section),
        name: String::new(),
        class: class.clone(),
        position: place.position,
        direction: Vec3::ZERO,
        game_vertex: Some(place.game_vertex.0),
        level_vertex: Some(place.level_vertex),
        story_id: None,
        spawn_id: u16::MAX,
        switching: ObjectSwitching::of_class(alife_class, ObjectSwitching::CREATED_FLAGS),
      },
      self.find_level(place),
    )?;

    if let Some(record) = objects.get_records_mut().get_mut(id) {
      record.name = ObjectRecord::name_created(section, id);
    }

    if let Some(kind) = kind {
      objects.set_schedule(
        id,
        ObjectSchedule {
          kind,
          direct_control: true,
          updated: None,
        },
      );
    }

    Some(id)
  }

  /// The spawn point squad `id` picks (`create_npc`'s `spawn_point`).
  fn pick_spawn_point<'a>(&self, squad: &'a SquadDescription, id: ObjectId) -> Option<&'a str> {
    let mut random: RandomStream = RandomStream::new(self.seed, Self::address(Self::SPAWN_POINT_DOMAIN, id));

    SimulationFacts::default().pick(&squad.spawn_point, self.hooks, &mut random)
  }

  /// The members squad `id` is made of: each fixed member once, then `npc_in_squad` drawn from its random members, a
  /// name listed twice drawn twice as often (`create_npc`'s `math.random`).
  fn draw_members<'a>(&self, squad: &'a SquadDescription, id: ObjectId) -> Vec<&'a str> {
    let mut members: Vec<&str> = squad.members.iter().map(String::as_str).collect();

    if squad.random_members.is_empty() {
      return members;
    }

    let mut random: RandomStream = RandomStream::new(self.seed, Self::address(Self::MEMBERS_DOMAIN, id));
    let count: u64 = u64::from(squad.size.min) + random.below(u64::from(squad.size.max - squad.size.min) + 1);

    for _ in 0..count {
      let index: u64 = random.below(squad.random_members.len() as u64);

      members.push(&squad.random_members[usize::try_from(index).unwrap_or_default()]);
    }

    members
  }

  /// Where object `id` stands, none for one with no game vertex.
  fn find_place(objects: &ObjectRegistry, id: ObjectId) -> Option<SimulationPlace> {
    let record: &ObjectRecord = objects.get_records().get(id)?;

    Some(SimulationPlace {
      position: record.position,
      level_vertex: record.level_vertex.unwrap_or(u32::MAX),
      game_vertex: GameVertexId(record.game_vertex?),
    })
  }

  /// The first point of the patrol `name` (`patrol(name):point(0)`).
  fn find_patrol_point(&self, name: &str) -> Option<SimulationPlace> {
    let point = self.patrols.get(name)?.first()?;

    Some(SimulationPlace {
      position: point.position,
      level_vertex: point.level_vertex,
      game_vertex: point.game_vertex,
    })
  }

  /// The graph level `place` is on, by its game vertex.
  fn find_level(&self, place: SimulationPlace) -> Option<u8> {
    self.graph.get_vertex(place.game_vertex).map(|vertex| vertex.level)
  }

  /// Where `place` stands, as the registry places an object.
  fn place_of(&self, place: SimulationPlace) -> ObjectPlace {
    ObjectPlace {
      position: place.position,
      level: self.find_level(place),
      game_vertex: Some(place.game_vertex.0),
      level_vertex: Some(place.level_vertex),
    }
  }

  /// The address of a draw about squad `id` in `domain`, at a new game's first step.
  fn address(domain: u64, id: ObjectId) -> RandomAddress {
    RandomAddress {
      domain,
      entity: id.get_raw(),
      step: 0,
    }
  }
}

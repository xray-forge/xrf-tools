use std::collections::BTreeMap;

use xrf_spawn::{AlifeObjectInherited, SpawnPlacedObject, SpawnPlacedObjects};

use crate::id::id_map::IdMap;
use crate::id::object_id::ObjectId;
use crate::record::object_ammo::ObjectAmmo;
use crate::record::object_creature::ObjectCreature;
use crate::record::object_level::ObjectLevel;
use crate::record::object_online::ObjectOnline;
use crate::record::object_record::ObjectRecord;
use crate::record::object_schedule::ObjectSchedule;
use crate::record::object_schedule_kind::ObjectScheduleKind;

/// The game's objects, X-Ray's object registry as id-keyed tables: the one store systems share. It is data, not a
/// system: the game's state owns it and hands a system the tables it reads or writes as that system's context, so the
/// compiler sees who writes what. Beside the records, every object's, stand the concern tables, each holding rows for
/// the objects it concerns (creatures, ammo, the schedule), the online table (who is online) and the children index
/// (what each object holds); a system needing more of some objects adds a table, never a field to every record.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjectRegistry {
  levels: Vec<ObjectLevel>,
  records: IdMap<ObjectRecord>,
  creatures: IdMap<ObjectCreature>,
  ammo: IdMap<ObjectAmmo>,
  schedules: IdMap<ObjectSchedule>,
  online: IdMap<ObjectOnline>,
  children: IdMap<Vec<ObjectId>>,
}

impl ObjectRegistry {
  /// The health a creature needs above to be updated offline (`EPS_L`, which `CSE_ALifeMonsterAbstract::need_update`
  /// compares it with); going offline needs only more than none.
  const UPDATED_HEALTH: f32 = 0.0001;

  /// A new game's objects: every object the spawn places, given ids in the spawn's order as a new game creates them
  /// (a spawn packet's own id is not the object's: X-Ray hands ids out as it creates each, `xrServer::PerformIDgen`),
  /// with the game graph's levels, all offline. A parent is found by its spawn id; objects past the sixteen-bit id space
  /// are left out, as X-Ray could not create them either.
  pub fn of_spawn(placed: &SpawnPlacedObjects) -> Self {
    // Numbered once, so a parent's id and its record's id are the same number.
    let numbered: Vec<(ObjectId, &SpawnPlacedObject)> =
      (0..ObjectId::NONE).map(ObjectId).zip(&placed.objects).collect();
    let ids: BTreeMap<u16, ObjectId> = numbered
      .iter()
      .map(|(id, object)| (object.object.header.spawn_id, *id))
      .collect();
    let mut registry: Self = Self {
      levels: placed
        .levels
        .iter()
        .map(|level| ObjectLevel {
          id: level.id,
          name: level.name.clone(),
        })
        .collect(),
      ..Self::default()
    };

    for (id, object) in &numbered {
      registry.insert(*id, Self::create_record(object, &ids));

      if let Some(creature) = object.object.inherited.get_creature() {
        registry.creatures.insert(
          *id,
          ObjectCreature {
            health: creature.health,
          },
        );
      }

      if let AlifeObjectInherited::CseAlifeItemAmmo(ammo) = &object.object.inherited {
        registry.ammo.insert(
          *id,
          ObjectAmmo {
            rounds: ammo.update_ammo_left,
          },
        );
      }

      if let Some(kind) = ObjectScheduleKind::of_class(object.object.inherited.get_class()) {
        let direct_control: bool = object
          .object
          .inherited
          .get_abstract()
          .is_some_and(|placed| placed.direct_control != 0);

        registry.schedules.insert(
          *id,
          ObjectSchedule {
            kind,
            direct_control,
            updated: None,
          },
        );
      }
    }

    registry
  }

  /// The schedule table: a row for each object A-Life may update offline (stalkers, monsters, smart terrains, squads).
  pub fn get_schedules(&self) -> &IdMap<ObjectSchedule> {
    &self.schedules
  }

  pub fn get_schedules_mut(&mut self) -> &mut IdMap<ObjectSchedule> {
    &mut self.schedules
  }

  /// Whether A-Life updates `id` offline now: it has a schedule row, it is offline, and by its kind a creature is under
  /// direct control and alive, a smart zone under direct control, a group always (`need_update` of
  /// `CSE_ALifeSchedulable`, `CSE_ALifeMonsterAbstract` and `CSE_ALifeOnlineOfflineGroup`). Read from the tables each
  /// time rather than kept as a list beside them, so it cannot fall out of step with them as X-Ray's schedule registry
  /// does (Anomaly's engine silences the asserts that catch it).
  pub fn needs_update(&self, id: ObjectId) -> bool {
    let Some(schedule) = self.schedules.get(id) else {
      return false;
    };

    if self.is_online(id) {
      return false;
    }

    match schedule.kind {
      ObjectScheduleKind::Group => true,
      ObjectScheduleKind::SmartZone => schedule.direct_control,
      ObjectScheduleKind::Creature => {
        schedule.direct_control
          && self
            .creatures
            .get(id)
            .is_some_and(|creature| creature.health > Self::UPDATED_HEALTH)
      }
    }
  }

  /// The creatures table: a row for each stalker, monster and the actor.
  pub fn get_creatures(&self) -> &IdMap<ObjectCreature> {
    &self.creatures
  }

  pub fn get_creatures_mut(&mut self) -> &mut IdMap<ObjectCreature> {
    &mut self.creatures
  }

  /// The ammo table: a row for each ammo box.
  pub fn get_ammo(&self) -> &IdMap<ObjectAmmo> {
    &self.ammo
  }

  pub fn get_ammo_mut(&mut self) -> &mut IdMap<ObjectAmmo> {
    &mut self.ammo
  }

  /// Whether `id` may go offline as it stands, beyond its switching: neither a dead creature nor an empty ammo box
  /// (`CSE_ALifeCreatureAbstract::can_switch_offline`, `CSE_ALifeItemAmmo::can_switch_offline`).
  pub fn may_go_offline(&self, id: ObjectId) -> bool {
    self.creatures.get(id).is_none_or(|creature| creature.health > 0.0)
      && self.ammo.get(id).is_none_or(|ammo| ammo.rounds != 0)
  }

  /// Puts `record` at `id`, offline, under its parent's children.
  pub fn insert(&mut self, id: ObjectId, record: ObjectRecord) {
    if let Some(parent) = record.parent {
      match self.children.get_mut(parent) {
        Some(children) => children.push(id),
        None => {
          self.children.insert(parent, vec![id]);
        }
      }
    }

    self.records.insert(id, record);
  }

  pub fn get_levels(&self) -> &[ObjectLevel] {
    &self.levels
  }

  /// The id of the level named `name`, case aside.
  pub fn find_level(&self, name: &str) -> Option<u8> {
    self
      .levels
      .iter()
      .find(|level| level.name.eq_ignore_ascii_case(name))
      .map(|level| level.id)
  }

  pub fn get_records(&self) -> &IdMap<ObjectRecord> {
    &self.records
  }

  pub fn get_records_mut(&mut self) -> &mut IdMap<ObjectRecord> {
    &mut self.records
  }

  /// Who is online, by id.
  pub fn get_online(&self) -> &IdMap<ObjectOnline> {
    &self.online
  }

  pub fn is_online(&self, id: ObjectId) -> bool {
    self.online.contains(id)
  }

  /// The objects `id` holds, in the order they were put under it.
  pub fn get_children(&self, id: ObjectId) -> &[ObjectId] {
    self.children.get(id).map_or(&[], Vec::as_slice)
  }

  /// Brings `id` online at `tick` with everything it holds, as X-Ray's `add_online` brings an object's children
  /// (`CSE_ALifeDynamicObject::add_online`). One already online keeps its tick.
  pub fn bring_online(&mut self, id: ObjectId, tick: u64) {
    let mut pending: Vec<ObjectId> = vec![id];

    while let Some(id) = pending.pop() {
      if !self.online.contains(id) {
        self.online.insert(id, ObjectOnline { since: tick });
      }

      pending.extend_from_slice(self.get_children(id));
    }
  }

  /// Takes `id` offline with everything it holds.
  pub fn take_offline(&mut self, id: ObjectId) {
    let mut pending: Vec<ObjectId> = vec![id];

    while let Some(id) = pending.pop() {
      self.online.remove(id);
      pending.extend_from_slice(self.get_children(id));
    }
  }

  /// How many objects stand on the level `level` names.
  pub fn count_on_level(&self, level: u8) -> usize {
    self
      .records
      .iter()
      .filter(|(_, record)| record.level == Some(level))
      .count()
  }

  /// The record of a spawned object, its parent found among the objects by spawn id.
  fn create_record(placed: &SpawnPlacedObject, ids: &BTreeMap<u16, ObjectId>) -> ObjectRecord {
    let parent: Option<ObjectId> =
      ObjectId::of(placed.object.header.parent_id).and_then(|parent| ids.get(&parent.0).copied());

    ObjectRecord::of_spawn(&placed.object, placed.level, parent)
  }
}

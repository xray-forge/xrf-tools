use std::collections::{BTreeMap, BTreeSet};

use xrf_spawn::{AlifeObjectInherited, SpawnPlacedObject, SpawnPlacedObjects};

use crate::id::id_map::IdMap;
use crate::id::object_id::ObjectId;
use crate::id::object_ids::ObjectIds;
use crate::record::object_ammo::ObjectAmmo;
use crate::record::object_creature::ObjectCreature;
use crate::record::object_group::ObjectGroup;
use crate::record::object_member::ObjectMember;
use crate::record::object_online::ObjectOnline;
use crate::record::object_place::ObjectPlace;
use crate::record::object_record::ObjectRecord;
use crate::record::object_schedule::ObjectSchedule;
use crate::record::object_schedule_kind::ObjectScheduleKind;

/// The game's objects, X-Ray's object registry as id-keyed tables: the one store systems share. It is data, not a
/// system: the game's state owns it and hands a system what it reads or writes as that system's context. Beside the
/// records, every object's, stand the concern tables, each holding rows for the objects it concerns (creatures, ammo,
/// the schedule, squads and their members), the online table (who is online), and the relations: the level each object
/// stands on and what holds it, with their indexes (each level's objects, what each object holds). A system needing
/// more of some objects adds a table, never a field to every record.
///
/// Rows are added and taken, and relations changed, only through its methods, which keep every table to the objects
/// that exist and every index in step with what it indexes; a value already in a row of the records or the schedule
/// may be changed in place (`get_records_mut`, `get_schedules_mut`), as nothing indexes those values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjectRegistry {
  records: IdMap<ObjectRecord>,
  creatures: IdMap<ObjectCreature>,
  ammo: IdMap<ObjectAmmo>,
  schedules: IdMap<ObjectSchedule>,
  groups: IdMap<ObjectGroup>,
  members: IdMap<ObjectMember>,
  online: IdMap<ObjectOnline>,
  /// The graph level each object on one stands on.
  levels: IdMap<u8>,
  /// The object holding each held one, as a stalker holds an item.
  parents: IdMap<ObjectId>,
  /// What each object holds, in the order it was put under it: the index of `parents`.
  children: IdMap<Vec<ObjectId>>,
  /// The objects standing on each level, in id order: the index of `levels`.
  on_level: BTreeMap<u8, BTreeSet<ObjectId>>,
  ids: ObjectIds,
}

impl ObjectRegistry {
  /// The health a creature needs above to be updated offline (`EPS_L`, which `CSE_ALifeMonsterAbstract::need_update`
  /// compares it with); going offline needs only more than none.
  const UPDATED_HEALTH: f32 = 0.0001;

  /// A new game's objects: every object the spawn places, given ids in the spawn's order as a new game creates them
  /// (a spawn packet's own id is not the object's: X-Ray hands ids out as it creates each, `xrServer::PerformIDgen`),
  /// all offline. A parent is found by its spawn id. The levels a level id names are the game graph's
  /// (`xrf-game-graph`).
  pub fn of_spawn(placed: &SpawnPlacedObjects) -> Self {
    let numbered: Vec<(ObjectId, &SpawnPlacedObject)> =
      (0..=u32::MAX).map(ObjectId::of_index).zip(&placed.objects).collect();
    let ids: BTreeMap<u16, ObjectId> = numbered
      .iter()
      .map(|(id, object)| (object.object.header.spawn_id, *id))
      .collect();
    let mut registry: Self = Self::default();

    for (id, object) in &numbered {
      registry.insert(*id, ObjectRecord::of_spawn(&object.object), object.level);
      registry.fill_concerns(*id, object);
    }

    // Held after all are put, as a holder may come after what it holds.
    for (id, object) in &numbered {
      let parent_id: u16 = object.object.header.parent_id;

      if parent_id != u16::MAX
        && let Some(parent) = ids.get(&parent_id)
      {
        registry.set_parent(*id, Some(*parent));
      }
    }

    registry
  }

  /// The schedule table: a row for each object A-Life may update offline (stalkers, monsters, smart terrains, squads).
  pub fn get_schedules(&self) -> &IdMap<ObjectSchedule> {
    &self.schedules
  }

  /// The schedule table, its rows' values to change.
  pub fn get_schedules_mut(&mut self) -> &mut IdMap<ObjectSchedule> {
    &mut self.schedules
  }

  /// Gives object `id` its schedule row, `false` for an object that does not exist.
  pub fn set_schedule(&mut self, id: ObjectId, schedule: ObjectSchedule) -> bool {
    if !self.records.contains(id) {
      return false;
    }

    self.schedules.insert(id, schedule);

    true
  }

  /// Whether A-Life updates `id` offline now: it has a schedule row, it is offline, it is no squad's member (its squad
  /// is updated for it, as `register_member` takes a member off the schedule), and by its kind a creature is under
  /// direct control and alive, a smart zone under direct control, a group always (`need_update` of
  /// `CSE_ALifeSchedulable`, `CSE_ALifeMonsterAbstract` and `CSE_ALifeOnlineOfflineGroup`). Read from the tables each
  /// time rather than kept as a list beside them, so it cannot fall out of step with them as X-Ray's schedule registry
  /// does (Anomaly's engine silences the asserts that catch it).
  pub fn needs_update(&self, id: ObjectId) -> bool {
    let Some(schedule) = self.schedules.get(id) else {
      return false;
    };

    if self.is_online(id) || self.members.contains(id) {
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

  /// Gives object `id` its creature row, `false` for an object that does not exist.
  pub fn set_creature(&mut self, id: ObjectId, creature: ObjectCreature) -> bool {
    if !self.records.contains(id) {
      return false;
    }

    self.creatures.insert(id, creature);

    true
  }

  /// The ammo table: a row for each ammo box.
  pub fn get_ammo(&self) -> &IdMap<ObjectAmmo> {
    &self.ammo
  }

  /// Gives object `id` its ammo row, `false` for an object that does not exist.
  pub fn set_ammo(&mut self, id: ObjectId, ammo: ObjectAmmo) -> bool {
    if !self.records.contains(id) {
      return false;
    }

    self.ammo.insert(id, ammo);

    true
  }

  /// Whether `id` may go offline as it stands, beyond its switching: neither a dead creature nor an empty ammo box
  /// (`CSE_ALifeCreatureAbstract::can_switch_offline`, `CSE_ALifeItemAmmo::can_switch_offline`).
  pub fn may_go_offline(&self, id: ObjectId) -> bool {
    self.creatures.get(id).is_none_or(|creature| creature.health > 0.0)
      && self.ammo.get(id).is_none_or(|ammo| ammo.rounds != 0)
  }

  /// Puts `record` at `id` on `level`, offline and held by nothing, as a new game's spawn and a save put objects at the
  /// ids they hold; an object created after takes another id. `false`, and nothing put, where an object holds the slot.
  pub fn insert(&mut self, id: ObjectId, record: ObjectRecord, level: Option<u8>) -> bool {
    if self.records.find_in_slot(id.get_index()).is_some() {
      return false;
    }

    self.ids.reserve(id);
    self.records.insert(id, record);
    self.place_on(id, level);

    true
  }

  /// Creates `record` as a new object on `level`, offline and held by nothing, under the next id: a freed slot a
  /// generation on, else a slot never used (`ObjectIds`). None once every slot is taken or retired, past four billion.
  pub fn create(&mut self, record: ObjectRecord, level: Option<u8>) -> Option<ObjectId> {
    let id: ObjectId = self.ids.take()?;

    self.records.insert(id, record);
    self.place_on(id, level);

    Some(id)
  }

  /// Releases object `id` and everything it holds, as X-Ray's `CALifeSimulatorBase::release` releases an object's
  /// children with it: each leaves what holds it and its squad, a released squad's members leave it, and every row of
  /// theirs goes. Their slots are handed out again a generation on. `false` for an object that does not exist.
  pub fn release(&mut self, id: ObjectId) -> bool {
    if !self.records.contains(id) {
      return false;
    }

    self.set_parent(id, None);

    let mut pending: Vec<ObjectId> = vec![id];

    while let Some(id) = pending.pop() {
      if let Some(children) = self.children.remove(id) {
        pending.extend(children);
      }

      self.leave_group(id);

      if let Some(group) = self.groups.remove(id) {
        for member in group.members {
          self.members.remove(member);
        }
      }

      self.place_on(id, None);
      self.parents.remove(id);
      self.records.remove(id);
      self.creatures.remove(id);
      self.ammo.remove(id);
      self.schedules.remove(id);
      self.online.remove(id);
      self.ids.release(id);
    }

    true
  }

  pub fn get_records(&self) -> &IdMap<ObjectRecord> {
    &self.records
  }

  /// The records, their values to change: nothing indexes a record's fields.
  pub fn get_records_mut(&mut self) -> &mut IdMap<ObjectRecord> {
    &mut self.records
  }

  /// The graph level `id` stands on, none for one on no level or that does not exist.
  pub fn get_level(&self, id: ObjectId) -> Option<u8> {
    self.levels.get(id).copied()
  }

  /// The objects standing on `level`, in id order.
  pub fn get_level_objects(&self, level: u8) -> impl Iterator<Item = ObjectId> + '_ {
    self.on_level.get(&level).into_iter().flatten().copied()
  }

  /// How many objects stand on `level`.
  pub fn count_on_level(&self, level: u8) -> usize {
    self.on_level.get(&level).map_or(0, BTreeSet::len)
  }

  /// Moves object `id` to `place`, `false` for an object that does not exist.
  pub fn move_to(&mut self, id: ObjectId, place: ObjectPlace) -> bool {
    let Some(record) = self.records.get_mut(id) else {
      return false;
    };

    record.position = place.position;
    record.game_vertex = place.game_vertex;
    record.level_vertex = place.level_vertex;
    self.place_on(id, place.level);

    true
  }

  /// Who is online, by id.
  pub fn get_online(&self) -> &IdMap<ObjectOnline> {
    &self.online
  }

  pub fn is_online(&self, id: ObjectId) -> bool {
    self.online.contains(id)
  }

  /// What holds `id`, none for an object held by nothing or that does not exist.
  pub fn get_parent(&self, id: ObjectId) -> Option<ObjectId> {
    self.parents.get(id).copied()
  }

  /// The objects `id` holds, in the order they were put under it.
  pub fn get_children(&self, id: ObjectId) -> &[ObjectId] {
    self.children.get(id).map_or(&[], Vec::as_slice)
  }

  /// Puts `id` under `parent`, last of what it holds, or under nothing; `false`, and nothing changed, where either does
  /// not exist or `id` holds `parent` at any depth or is it.
  pub fn set_parent(&mut self, id: ObjectId, parent: Option<ObjectId>) -> bool {
    if !self.records.contains(id) {
      return false;
    }

    if let Some(parent) = parent
      && (!self.records.contains(parent) || self.is_holding(id, parent))
    {
      return false;
    }

    if let Some(previous) = self.parents.remove(id)
      && let Some(children) = self.children.get_mut(previous)
    {
      children.retain(|child| *child != id);

      if children.is_empty() {
        self.children.remove(previous);
      }
    }

    if let Some(parent) = parent {
      self.parents.insert(id, parent);

      match self.children.get_mut(parent) {
        Some(children) => children.push(id),
        None => {
          self.children.insert(parent, vec![id]);
        }
      }
    }

    true
  }

  /// Brings `id` online at `tick` with everything it holds and, for a squad, its members, as X-Ray's `add_online`
  /// brings an object's children (`CSE_ALifeDynamicObject::add_online`) and a squad's `switch_online` its members
  /// (`CSE_ALifeOnlineOfflineGroup::switch_online`). One already online keeps its tick. Answers the objects that came
  /// online.
  pub fn bring_online(&mut self, id: ObjectId, tick: u64) -> Vec<ObjectId> {
    let mut pending: Vec<ObjectId> = vec![id];
    let mut switched: Vec<ObjectId> = Vec::new();

    while let Some(id) = pending.pop() {
      if self.records.contains(id) && !self.online.contains(id) {
        self.online.insert(id, ObjectOnline { since: tick });
        switched.push(id);
      }

      pending.extend(self.get_followers(id));
    }

    switched
  }

  /// Takes `id` offline with everything it holds and, for a squad, its members. Answers the objects that went offline.
  pub fn take_offline(&mut self, id: ObjectId) -> Vec<ObjectId> {
    let mut pending: Vec<ObjectId> = vec![id];
    let mut switched: Vec<ObjectId> = Vec::new();

    while let Some(id) = pending.pop() {
      if self.online.remove(id).is_some() {
        switched.push(id);
      }

      pending.extend(self.get_followers(id));
    }

    switched
  }

  /// The groups table: a row for each squad.
  pub fn get_groups(&self) -> &IdMap<ObjectGroup> {
    &self.groups
  }

  /// The members table: a row for each squad member, naming its squad.
  pub fn get_members(&self) -> &IdMap<ObjectMember> {
    &self.members
  }

  /// The members of squad `id`, the commander first; none for an object that is no squad.
  pub fn get_group_members(&self, id: ObjectId) -> &[ObjectId] {
    self.groups.get(id).map_or(&[], |group| group.members.as_slice())
  }

  /// Makes object `id` a squad assigned to `smart`, with the members it has (none for one new to it); `false` for an
  /// object that does not exist.
  pub fn form_group(&mut self, id: ObjectId, smart: Option<ObjectId>) -> bool {
    if !self.records.contains(id) {
      return false;
    }

    match self.groups.get_mut(id) {
      Some(group) => group.smart = smart,
      None => {
        self.groups.insert(
          id,
          ObjectGroup {
            members: Vec::new(),
            smart,
          },
        );
      }
    }

    true
  }

  /// Makes `member` a member of squad `group`, last of its members (`CSE_ALifeOnlineOfflineGroup::register_member`),
  /// out of any squad it was in; the squad's row is made, assigned to no smart, if it has none. `false`, and nothing
  /// changed, where either does not exist or they are one.
  pub fn join_group(&mut self, group: ObjectId, member: ObjectId) -> bool {
    if group == member || !self.records.contains(group) || !self.records.contains(member) {
      return false;
    }

    self.leave_group(member);

    match self.groups.get_mut(group) {
      Some(row) => row.members.push(member),
      None => {
        self.groups.insert(
          group,
          ObjectGroup {
            members: vec![member],
            smart: None,
          },
        );
      }
    }

    self.members.insert(member, ObjectMember { group });

    true
  }

  /// Takes `member` out of the squad it is in, if any.
  fn leave_group(&mut self, member: ObjectId) {
    if let Some(row) = self.members.remove(member)
      && let Some(group) = self.groups.get_mut(row.group)
    {
      group.members.retain(|id| *id != member);
    }
  }

  /// Whether `holder` is `id` or holds it at any depth.
  fn is_holding(&self, holder: ObjectId, id: ObjectId) -> bool {
    let mut current: Option<ObjectId> = Some(id);

    while let Some(id) = current {
      if id == holder {
        return true;
      }

      current = self.parents.get(id).copied();
    }

    false
  }

  /// The objects that go online and offline with `id`: what it holds, and a squad's members.
  fn get_followers(&self, id: ObjectId) -> impl Iterator<Item = ObjectId> + '_ {
    self.get_children(id).iter().chain(self.get_group_members(id)).copied()
  }

  /// Stands `id` on `level`, or on none, keeping the level index.
  fn place_on(&mut self, id: ObjectId, level: Option<u8>) {
    if let Some(previous) = self.levels.remove(id)
      && let Some(objects) = self.on_level.get_mut(&previous)
    {
      objects.remove(&id);

      if objects.is_empty() {
        self.on_level.remove(&previous);
      }
    }

    if let Some(level) = level {
      self.levels.insert(id, level);
      self.on_level.entry(level).or_default().insert(id);
    }
  }

  /// Fills the concern tables of spawned object `id` from its packet.
  fn fill_concerns(&mut self, id: ObjectId, placed: &SpawnPlacedObject) {
    let inherited: &AlifeObjectInherited = &placed.object.inherited;

    if let Some(creature) = inherited.get_creature() {
      self.creatures.insert(
        id,
        ObjectCreature {
          health: creature.health,
        },
      );
    }

    if let AlifeObjectInherited::CseAlifeItemAmmo(ammo) = inherited {
      self.ammo.insert(
        id,
        ObjectAmmo {
          rounds: ammo.update_ammo_left,
        },
      );
    }

    if let Some(kind) = ObjectScheduleKind::of_class(inherited.get_class()) {
      let direct_control: bool = inherited
        .get_abstract()
        .is_some_and(|abstract_object| abstract_object.direct_control != 0);

      self.schedules.insert(
        id,
        ObjectSchedule {
          kind,
          direct_control,
          updated: None,
        },
      );
    }
  }
}

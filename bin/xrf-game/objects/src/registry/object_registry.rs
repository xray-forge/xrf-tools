use std::collections::BTreeMap;

use xrf_spawn::{SpawnPlacedObject, SpawnPlacedObjects};

use crate::id::id_map::IdMap;
use crate::id::object_id::ObjectId;
use crate::record::object_level::ObjectLevel;
use crate::record::object_record::ObjectRecord;

/// The game's objects, X-Ray's object registry as id-keyed tables: the one store systems share. It is data, not a
/// system: the game's state owns it and hands a system the tables it reads or writes as that system's context, so the
/// compiler sees who writes what. Tables a system adds (online state, schedules) sit beside the records, keyed alike.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjectRegistry {
  levels: Vec<ObjectLevel>,
  records: IdMap<ObjectRecord>,
}

impl ObjectRegistry {
  /// A new game's objects: every object the spawn places, given ids in the spawn's order as a new game creates them
  /// (a spawn packet's own id is not the object's: X-Ray hands ids out as it creates each, `xrServer::PerformIDgen`),
  /// with the game graph's levels. A parent is found by its spawn id; objects past the sixteen-bit id space are left
  /// out, as X-Ray could not create them either.
  pub fn of_spawn(placed: &SpawnPlacedObjects) -> Self {
    // Numbered once, so a parent's id and its record's id are the same number.
    let numbered: Vec<(ObjectId, &SpawnPlacedObject)> =
      (0..ObjectId::NONE).map(ObjectId).zip(&placed.objects).collect();
    let ids: BTreeMap<u16, ObjectId> = numbered
      .iter()
      .map(|(id, object)| (object.object.header.spawn_id, *id))
      .collect();
    let mut records: IdMap<ObjectRecord> = IdMap::default();

    for (id, object) in &numbered {
      records.insert(*id, Self::create_record(object, &ids));
    }

    Self {
      levels: placed
        .levels
        .iter()
        .map(|level| ObjectLevel {
          id: level.id,
          name: level.name.clone(),
        })
        .collect(),
      records,
    }
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

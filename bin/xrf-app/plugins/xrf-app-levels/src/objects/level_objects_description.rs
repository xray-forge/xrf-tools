use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use xrf_app_core::TauriResult;
use xrf_app_core::session::SessionSnapshot;
use xrf_math::Vector3d;
use xrf_spawn::{Patrol, SpawnLevelGraph};
use xrf_vfs::XraySearch;
use xrf_visual::VisualTransform;

use crate::level::SelectedLevel;
use crate::objects::{
  LevelObject, LevelObjectId, LevelObjectKind, LevelObjectPatrol, LevelObjectPatrolPoint, LevelObjectSpawn,
  LevelObjectVolume,
};
use crate::parts::{LevelGraphPart, LevelSpawn, LevelSpawnPart};
use crate::report::LevelReport;

/// Every level object of a level, in the order its sources keep them, and the visuals they are drawn as, each named
/// once.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelObjectsDescription {
  pub visuals: Vec<String>,
  pub objects: Vec<LevelObject>,
}

impl LevelObjectsDescription {
  /// The open level's objects, read through `search`, with how long that took: the spawn's, then the graph's where it
  /// can be read, its patrols and points left out with a warning where it cannot.
  ///
  /// # Errors
  ///
  /// Returns an error when the spawn cannot be read.
  pub fn read(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> TauriResult<Self> {
    let started: Instant = Instant::now();
    let spawn: Arc<LevelSpawn> = LevelSpawnPart::get(current, search)
      .inspect_err(|error| LevelReport::missing_level_objects(&current.source, error))?;
    let graph: Option<Arc<SpawnLevelGraph>> = LevelGraphPart::get(current, search)
      .inspect_err(|error| LevelReport::missing_level_graph(&current.source, error))
      .ok();
    let described: Self = Self::describe(&spawn, graph.as_deref());

    LevelReport::level_objects(&current.source, &described, started);

    Ok(described)
  }

  /// Every object of a level's spawn the level holds, in the spawn's order, each visual named once; then, of its
  /// `graph`, every patrol path starting on it followed by its points, and its graph points.
  pub fn describe(spawn: &LevelSpawn, graph: Option<&SpawnLevelGraph>) -> Self {
    let mut visuals: Vec<String> = Vec::new();
    let mut indices: HashMap<&str, u32> = HashMap::new();
    let mut objects: Vec<LevelObject> = Vec::new();

    for (index, object) in spawn.objects.iter().enumerate() {
      let Some(kind) = LevelObjectKind::of(object) else {
        continue;
      };
      let visual: Option<u32> = kind
        .get_entry()
        .is_drawn
        .then(|| object.inherited.get_visual())
        .flatten()
        .map(|name| {
          *indices.entry(name).or_insert_with(|| {
            visuals.push(name.to_owned());

            (visuals.len() - 1) as u32
          })
        });

      objects.push(LevelObject {
        spawn: Some(LevelObjectSpawn {
          section: object.section.clone(),
          clsid: object.clsid.clone(),
          story_id: object.inherited.get_abstract().and_then(|it| it.get_story_id()),
          release: spawn.get_release(index),
          visual,
        }),
        volume: LevelObjectVolume::of(&object.inherited),
        ..LevelObject::new(
          LevelObjectId::Spawn { index: index as u32 },
          kind,
          object.name.clone(),
          VisualTransform::of_spawn(&object.position, &object.direction),
        )
      });
    }

    if let Some(graph) = graph {
      for (path, patrol) in graph.patrols.iter().enumerate() {
        Self::add_patrol(&mut objects, path as u32, patrol);
      }

      objects.extend(graph.points.iter().map(|(vertex, position)| {
        LevelObject::new(
          LevelObjectId::GraphPoint { vertex: *vertex },
          LevelObjectKind::GraphPoints,
          format!("vertex {vertex}"),
          Self::to_place(position),
        )
      }));
    }

    Self { visuals, objects }
  }

  /// A patrol path, standing at its first point, then each of its points as a part of it.
  fn add_patrol(objects: &mut Vec<LevelObject>, path: u32, patrol: &Patrol) {
    let id: LevelObjectId = LevelObjectId::Patrol { path };
    let place: VisualTransform = patrol.points.first().map_or_else(
      || Self::to_place(&Vector3d::new(0.0, 0.0, 0.0)),
      |first| Self::to_place(&first.position),
    );

    objects.push(LevelObject {
      patrol: Some(LevelObjectPatrol::from(patrol)),
      ..LevelObject::new(id, LevelObjectKind::Patrols, patrol.name.clone(), place)
    });
    objects.extend(patrol.points.iter().enumerate().map(|(point, at)| LevelObject {
      parent: Some(id),
      patrol_point: Some(LevelObjectPatrolPoint::from(at)),
      ..LevelObject::new(
        LevelObjectId::PatrolPoint {
          path,
          point: point as u32,
        },
        LevelObjectKind::Patrols,
        at.name.clone(),
        Self::to_place(&at.position),
      )
    }));
  }

  /// Unturned at a point of engine space, in renderer space.
  fn to_place(position: &Vector3d) -> VisualTransform {
    VisualTransform {
      i: Vector3d::new(1.0, 0.0, 0.0),
      j: Vector3d::new(0.0, 1.0, 0.0),
      k: Vector3d::new(0.0, 0.0, 1.0),
      c: LevelObject::to_renderer_space(position),
    }
  }
}

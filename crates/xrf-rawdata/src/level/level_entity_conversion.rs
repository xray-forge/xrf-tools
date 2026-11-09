use byteorder::ByteOrder;
use xrf_chunk::{ChunkFieldRecorder, ChunkFieldReplay};
use xrf_editable_level::{
  EditableCustomObject, EditableObjectPart, EditableObjectTransform, EditableShape, EditableShapeElement,
  EditableShapeObject, EditableSpawnPointKind, EditableSpawnPointObject,
};
use xrf_error::{XrfError, XrfResult};
use xrf_ini::IniSection;
use xrf_math::{Sphere, Vector3d};
use xrf_spawn::{AlifeObject, LevelSpawnFile, LevelSpawnObject, Shape};

use crate::level::level_decompile_context::LevelDecompileContext;
use crate::level::level_spawn_data_fields::LevelSpawnDataFields;
use crate::provenance::level_scene_provenance::LevelSceneRefusal;

/// A level's server entities, between the scene's `spawn.part` and the spawn lists `CSpawnPoint::SSpawnData::
/// ExportGame` (`SpawnPoint.cpp:374`) fills: `level.spawn`, and `level_rs.spawn` for entities marked to respawn in
/// multiplayer.
///
/// The export takes an entity's name, position and angles from its spawn point, and its volume from the shape attached
/// to it, over what its packet holds; the decompile therefore gives each spawn point the packet's name and place, and a
/// class with a volume the packet's shapes as an attached shape.
pub struct LevelEntityConversion;

/// A level's entities decompiled: the spawn points of those typed, and those refused, each named with its section in
/// the reason.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelEntityDecompile {
  pub entities: Vec<EditableSpawnPointObject>,
  pub refused: Vec<LevelSceneRefusal>,
}

/// The spawn lists a scene's entities export to, each `None` where no entity goes there, as the build then writes none.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelEntityExport {
  pub spawn: Option<LevelSpawnFile>,
  pub respawn: Option<LevelSpawnFile>,
}

impl LevelEntityConversion {
  /// The spawn lists the build writes, `BuilderGame.cpp:30` and `:36`.
  pub const SPAWN_FILE_NAME: &'static str = "level.spawn";
  pub const RESPAWN_FILE_NAME: &'static str = "level_rs.spawn";

  /// The prefix the decompile names an entity's attached shape after.
  pub const SHAPE_PREFIX: &'static str = "shape";

  /// The spawn points of a level's entities: those of `spawn` first, then those of `respawn` marked to respawn, each
  /// in its list's order. Every packet's name is reserved before any other is generated, as the export keeps it.
  ///
  /// # Errors
  ///
  /// Returns an error when a typed packet's fields cannot be saved, a string holding a double quote; a packet whose
  /// class is not known is refused one by one instead.
  pub fn decompile<T: ByteOrder>(
    spawn: Option<&LevelSpawnFile>,
    respawn: Option<&LevelSpawnFile>,
    context: &mut LevelDecompileContext,
  ) -> XrfResult<LevelEntityDecompile> {
    let lists: [(Option<&LevelSpawnFile>, &'static str, u8); 2] = [
      (spawn, Self::SPAWN_FILE_NAME, 0),
      (
        respawn,
        Self::RESPAWN_FILE_NAME,
        EditableSpawnPointObject::RESPAWN_ENTITY,
      ),
    ];
    let mut decompile: LevelEntityDecompile = LevelEntityDecompile {
      entities: Vec::new(),
      refused: Vec::new(),
    };

    for object in lists
      .iter()
      .filter_map(|(list, _, _)| *list)
      .flat_map(|list| &list.objects)
    {
      context.names.reserve(&object.name);
    }

    for (list, file, flags) in lists {
      for object in list.map(|list| list.objects.as_slice()).unwrap_or_default() {
        let entity: AlifeObject = match object.read_alife_object::<T>() {
          Ok(entity) => entity,
          Err(error) => {
            decompile.refused.push(LevelSceneRefusal {
              file: file.to_owned(),
              object: object.name.clone(),
              reason: format!("section '{}': {error}", object.section),
            });
            continue;
          }
        };

        decompile
          .entities
          .push(Self::decompile_entity::<T>(&entity, flags, context)?);
      }
    }

    Ok(decompile)
  }

  /// The spawn lists the part's entities export to, each entity at the place, under the name and with the shape its
  /// spawn point gives, numbered from zero in the part's order within its list.
  ///
  /// # Errors
  ///
  /// Returns an error naming the spawn point whose spawn data does not read as its section's class, whose class has a
  /// volume but no shape attached (as the build refuses it), or whose packet the writer refuses.
  pub fn export<T: ByteOrder>(part: &EditableObjectPart<EditableSpawnPointObject>) -> XrfResult<LevelEntityExport> {
    let mut spawn: Vec<LevelSpawnObject> = Vec::new();
    let mut respawn: Vec<LevelSpawnObject> = Vec::new();

    for point in &part.objects {
      let EditableSpawnPointKind::Entity { section, flags, data } = &point.kind else {
        continue;
      };
      let list: &mut Vec<LevelSpawnObject> = if flags & EditableSpawnPointObject::RESPAWN_ENTITY == 0 {
        &mut spawn
      } else {
        &mut respawn
      };
      let id: u32 = u32::try_from(list.len())
        .map_err(|_| XrfError::new_invalid_error("A spawn list holds more entities than its chunk ids number"))?;

      list.push(
        Self::export_entity::<T>(point, section, data, id)
          .map_err(|error| XrfError::new_invalid_error(format!("Spawn point '{}': {error}", point.object.name)))?,
      );
    }

    let into_file = |objects: Vec<LevelSpawnObject>| (!objects.is_empty()).then_some(LevelSpawnFile { objects });

    Ok(LevelEntityExport {
      spawn: into_file(spawn),
      respawn: into_file(respawn),
    })
  }

  /// The spawn point of a typed packet: at its position and angles, unscaled, under its name, its packet as spawn data
  /// and a class's volume as an attached shape standing at the entity, unturned.
  fn decompile_entity<T: ByteOrder>(
    entity: &AlifeObject,
    flags: u8,
    context: &mut LevelDecompileContext,
  ) -> XrfResult<EditableSpawnPointObject> {
    let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();
    let mut data: IniSection = IniSection::new("spawndata");

    entity.write_spawn_packet::<T, _>(&mut recorder)?;
    LevelSpawnDataFields::write_section(&recorder.into_fields(), &mut data)?;

    let position: Vector3d = entity.header.position;
    let attached: Option<EditableShapeObject> = entity.inherited.get_shapes().map(|shapes| EditableShapeObject {
      object: EditableCustomObject::new(
        context.names.get_next(Self::SHAPE_PREFIX),
        EditableObjectTransform {
          position,
          ..EditableObjectTransform::default()
        },
      ),
      shape: EditableShape {
        usage: 0,
        elements: shapes.iter().map(Self::get_element).collect(),
      },
    });

    Ok(EditableSpawnPointObject {
      object: EditableCustomObject::new(
        entity.header.name.clone(),
        EditableObjectTransform {
          position,
          rotation: entity.header.direction,
          scale: Vector3d::ONE,
        },
      ),
      kind: EditableSpawnPointKind::Entity {
        section: entity.header.section.clone(),
        flags,
        data,
      },
      attached,
    })
  }

  /// One entity's packet as `ExportGame` writes it: the spawn data read as the section's class, then the spawn point's
  /// name, position and angles over the packet's, and a class's volume from the attached shape with its scale applied.
  fn export_entity<T: ByteOrder>(
    point: &EditableSpawnPointObject,
    section: &str,
    data: &IniSection,
    id: u32,
  ) -> XrfResult<LevelSpawnObject> {
    let mut fields: LevelSpawnDataFields<'_> = LevelSpawnDataFields::new(data);
    let mut entity: AlifeObject = AlifeObject::read_spawn_packet::<T, _>(&mut ChunkFieldReplay::new(&mut fields))?;

    fields.assert_read()?;

    if entity.header.section != section {
      return Err(XrfError::new_invalid_error(format!(
        "its spawn data is of section '{}', the entity of '{section}'",
        entity.header.section
      )));
    }

    entity.header.name.clone_from(&point.object.name);
    entity.header.position = point.object.transform.position;
    entity.header.direction = point.object.transform.rotation;

    if let Some(shapes) = entity.inherited.get_shapes_mut() {
      let attached: &EditableShapeObject = point
        .attached
        .as_ref()
        .ok_or_else(|| XrfError::new_invalid_error("its entity has a volume, so it must hold a shape attached"))?;

      *shapes = attached
        .get_scaled_elements()
        .into_iter()
        .map(Self::get_shape)
        .collect();
    }

    LevelSpawnObject::from_alife_object::<T>(id, &entity)
  }

  fn get_element(shape: &Shape) -> EditableShapeElement {
    match shape {
      Shape::Sphere(sphere) => EditableShapeElement::Sphere {
        center: sphere.center,
        radius: sphere.radius,
      },
      Shape::Box(matrix) => EditableShapeElement::Box(*matrix),
    }
  }

  fn get_shape(element: EditableShapeElement) -> Shape {
    match element {
      EditableShapeElement::Sphere { center, radius } => Shape::Sphere(Sphere { center, radius }),
      EditableShapeElement::Box(matrix) => Shape::Box(matrix),
    }
  }
}

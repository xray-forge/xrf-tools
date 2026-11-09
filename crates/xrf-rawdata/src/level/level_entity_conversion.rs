use std::collections::HashMap;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkFieldRecorder, ChunkFieldReplay, ChunkWriter};
use xrf_editable_level::{
  EditableCustomObject, EditableObjectPart, EditableObjectTransform, EditableShape, EditableShapeElement,
  EditableShapeObject, EditableSpawnPointKind, EditableSpawnPointObject,
};
use xrf_error::{XrfError, XrfResult};
use xrf_ini::IniSection;
use xrf_math::{Sphere, Vector3d};
use xrf_spawn::{AlifeBuiltFields, AlifeObject, AlifeObjectInherited, LevelSpawnFile, LevelSpawnObject, Shape};

use crate::level::level_decompile_context::LevelDecompileContext;
use crate::level::level_spawn_data_fields::LevelSpawnDataFields;
use crate::provenance::level_scene_provenance::{LevelSceneDisagreement, LevelSceneRefusal, LevelSceneUpdatePacket};
use crate::provenance::sidecar::Sidecar;

/// A level's server entities, between the scene's `spawn.part` and the spawn lists `CSpawnPoint::SSpawnData::
/// ExportGame` (`SpawnPoint.cpp:374`) fills: `level.spawn`, and `level_rs.spawn` for entities marked to respawn in
/// multiplayer.
///
/// The export takes an entity's name, position and angles from its spawn point, and its volume from the shape attached
/// to it, over what its packet holds; the decompile therefore gives each spawn point the packet's name and place, and a
/// class with a volume the packet's shapes as an attached shape.
pub struct LevelEntityConversion;

/// What a level's entities are decompiled from.
#[derive(Clone, Copy, Debug, Default)]
pub struct LevelEntitySources<'a> {
  pub spawn: Option<&'a LevelSpawnFile>,
  pub respawn: Option<&'a LevelSpawnFile>,
  /// The root spawn's objects standing on the level (decision 10). Where it holds any, they stand in for `spawn`'s
  /// entities but its graph points, which a root spawn keeps as graph vertices instead.
  pub root: &'a [AlifeObject],
}

/// A level's entities decompiled: the spawn points of those typed, those refused (each named with its section in the
/// reason), and, from a root spawn, what it and `level.spawn` disagree on and the update packets the scene cannot keep.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelEntityDecompile {
  pub entities: Vec<EditableSpawnPointObject>,
  pub refused: Vec<LevelSceneRefusal>,
  pub disagreements: Vec<LevelSceneDisagreement>,
  pub update_packets: Vec<LevelSceneUpdatePacket>,
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

  /// The spawn points of a level's entities: those of `spawn` (or the root spawn's) first, then those of `respawn`
  /// marked to respawn. Every name is reserved before any other is generated, as the export keeps it.
  ///
  /// From a root spawn, each object takes the place of its `level.spawn` packet of the same name, the fields xrAI fills
  /// in (`AlifeBuiltFields`) taken from that packet, and the objects only the root spawn holds follow in its order with
  /// a fresh entity's (`AlifeBuiltFields::UNBUILT`), as does an object whose `level.spawn` packet does not read. A disagreement is recorded for each of those, for each packet only
  /// `level.spawn` holds (left out), and for each object whose packet then is not `level.spawn`'s; an update packet is
  /// kept where it differs from a fresh entity's.
  ///
  /// # Errors
  ///
  /// Returns an error when a typed packet's fields cannot be saved, a string holding a double quote, or a root spawn
  /// object does not write; a packet whose class is not known is refused one by one instead.
  pub fn decompile<T: ByteOrder>(
    sources: &LevelEntitySources<'_>,
    context: &mut LevelDecompileContext,
  ) -> XrfResult<LevelEntityDecompile> {
    let spawn: &[LevelSpawnObject] = sources.spawn.map(|list| list.objects.as_slice()).unwrap_or_default();
    let respawn: &[LevelSpawnObject] = sources.respawn.map(|list| list.objects.as_slice()).unwrap_or_default();
    let mut decompile: LevelEntityDecompile = LevelEntityDecompile::default();

    for name in spawn
      .iter()
      .chain(respawn)
      .map(|object| object.name.as_str())
      .chain(sources.root.iter().map(|object| object.header.name.as_str()))
    {
      context.names.reserve(name);
    }

    if sources.root.is_empty() {
      Self::decompile_listed::<T>(spawn, Self::SPAWN_FILE_NAME, 0, &mut decompile, context)?;
    } else {
      Self::decompile_root::<T>(spawn, sources.root, &mut decompile, context)?;
    }

    Self::decompile_listed::<T>(
      respawn,
      Self::RESPAWN_FILE_NAME,
      EditableSpawnPointObject::RESPAWN_ENTITY,
      &mut decompile,
      context,
    )?;

    Ok(decompile)
  }

  /// A spawn list's packets as spawn points of `flags`, those of no known class refused.
  fn decompile_listed<T: ByteOrder>(
    objects: &[LevelSpawnObject],
    file: &str,
    flags: u8,
    decompile: &mut LevelEntityDecompile,
    context: &mut LevelDecompileContext,
  ) -> XrfResult {
    for object in objects {
      match object.read_alife_object::<T>() {
        Ok(entity) => decompile
          .entities
          .push(Self::decompile_entity::<T>(&entity, flags, context)?),
        Err(error) => decompile.refused.push(Self::refuse(object, file, &error)),
      }
    }

    Ok(())
  }

  /// `level.spawn`'s packets in its order, each replaced by the root spawn's object of its name, then the root spawn's
  /// objects `level.spawn` lacks; its graph points kept, its other packets the root spawn lacks left out.
  fn decompile_root<T: ByteOrder>(
    listed: &[LevelSpawnObject],
    root: &[AlifeObject],
    decompile: &mut LevelEntityDecompile,
    context: &mut LevelDecompileContext,
  ) -> XrfResult {
    let mut by_name: HashMap<&str, usize> = HashMap::with_capacity(root.len());
    let mut taken: Vec<bool> = vec![false; root.len()];

    for (index, object) in root.iter().enumerate() {
      by_name.entry(object.header.name.as_str()).or_insert(index);
    }

    for object in listed {
      if let Some(index) = by_name.remove(object.name.as_str()) {
        taken[index] = true;
        Self::decompile_built::<T>(&root[index], Some(object), decompile, context)?;
        continue;
      }

      match object.read_alife_object::<T>() {
        Ok(entity) if matches!(entity.inherited, AlifeObjectInherited::CseAlifeGraphPoint(_)) => {
          decompile
            .entities
            .push(Self::decompile_entity::<T>(&entity, 0, context)?);
        }
        Ok(_) => decompile.disagreements.push(LevelSceneDisagreement {
          object: object.name.clone(),
          reason: String::from("only in level.spawn, so left out: the root spawn holds the level"),
        }),
        Err(error) => decompile
          .refused
          .push(Self::refuse(object, Self::SPAWN_FILE_NAME, &error)),
      }
    }

    for (object, _) in root.iter().zip(taken).filter(|(_, taken)| !taken) {
      Self::decompile_built::<T>(object, None, decompile, context)?;
    }

    Ok(())
  }

  /// One root spawn object as a spawn point, compared with its `level.spawn` packet where there is one.
  fn decompile_built<T: ByteOrder>(
    built: &AlifeObject,
    listed: Option<&LevelSpawnObject>,
    decompile: &mut LevelEntityDecompile,
    context: &mut LevelDecompileContext,
  ) -> XrfResult {
    let name: &str = &built.header.name;
    let mut entity: AlifeObject = built.clone();
    // xrAI overwrites these whatever the editor wrote, so they come from the editor's own packet where `level.spawn`
    // holds one that reads, and are a fresh entity's otherwise.
    let editor_fields: AlifeBuiltFields = listed
      .and_then(|listed| listed.read_alife_object::<T>().ok())
      .map_or(AlifeBuiltFields::UNBUILT, |listed| listed.get_built_fields());

    entity.set_built_fields(&editor_fields);

    let disagreement: Option<&str> = match listed {
      None => Some("only in the root spawn"),
      // Compared as bytes: as floats, a negative zero would equal a positive one.
      Some(listed) if Self::get_spawn_packet::<T>(&entity)? != Self::get_listed_packet::<T>(listed)? => {
        Some("its spawn packet differs from level.spawn's past the fields xrAI fills in")
      }
      Some(_) => None,
    };

    if let Some(reason) = disagreement {
      decompile.disagreements.push(LevelSceneDisagreement {
        object: name.to_owned(),
        reason: reason.to_owned(),
      });
    }

    let stored: Vec<u8> = Self::get_update_packet::<T>(built)?;

    if stored != Self::get_update_packet::<T>(&built.get_fresh_entity::<T>()?)? {
      decompile.update_packets.push(LevelSceneUpdatePacket {
        object: name.to_owned(),
        bytes: Sidecar::format_hex(&stored),
      });
    }

    decompile
      .entities
      .push(Self::decompile_entity::<T>(&entity, 0, context)?);

    Ok(())
  }

  fn get_update_packet<T: ByteOrder>(object: &AlifeObject) -> XrfResult<Vec<u8>> {
    let mut bytes: ChunkWriter = ChunkWriter::new();

    object.write_update_packet::<T, _>(&mut bytes)?;

    Ok(bytes.buffer)
  }

  fn get_spawn_packet<T: ByteOrder>(object: &AlifeObject) -> XrfResult<Vec<u8>> {
    let mut bytes: ChunkWriter = ChunkWriter::new();

    object.write_spawn_packet::<T, _>(&mut bytes)?;

    Ok(bytes.buffer)
  }

  fn get_listed_packet<T: ByteOrder>(object: &LevelSpawnObject) -> XrfResult<Vec<u8>> {
    let mut bytes: ChunkWriter = ChunkWriter::new();

    object.write::<T>(&mut bytes)?;

    Ok(bytes.buffer)
  }

  fn refuse(object: &LevelSpawnObject, file: &str, error: &XrfError) -> LevelSceneRefusal {
    LevelSceneRefusal {
      file: file.to_owned(),
      object: object.name.clone(),
      reason: format!("section '{}': {error}", object.section),
    }
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

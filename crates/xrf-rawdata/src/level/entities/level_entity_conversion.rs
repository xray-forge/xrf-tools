use std::collections::HashMap;

use xrf_chunk::{ChunkFieldRecorder, ChunkWriter};
use xrf_editable_level::{
  EditableCustomObject, EditableObjectPart, EditableObjectTransform, EditableShape, EditableShapeElement,
  EditableShapeObject, EditableSpawnPointKind, EditableSpawnPointObject,
};
use xrf_error::{XrfError, XrfResult};
use xrf_ini::IniSection;
use xrf_math::{Sphere, Vector3d};
use xrf_spawn::{AlifeBuiltFields, AlifeObject, AlifeObjectInherited, LevelSpawnFile, LevelSpawnObject, Shape};

use crate::level::entities::level_entity_decompile::LevelEntityDecompile;
use crate::level::entities::level_entity_export::LevelEntityExport;
use crate::level::entities::level_entity_sources::LevelEntitySources;
use crate::level::entities::level_spawn_data_fields::LevelSpawnDataFields;
use crate::level::level_decompile_context::LevelDecompileContext;
use crate::provenance::level_scene_disagreement::LevelSceneDisagreement;
use crate::provenance::level_scene_refusal::LevelSceneRefusal;
use crate::provenance::level_scene_update_packet::LevelSceneUpdatePacket;
use crate::provenance::sidecar::Sidecar;

/// A level's server entities, between the scene's `spawn.part` and the spawn lists
/// `CSpawnPoint::SSpawnData::ExportGame` fills: `level.spawn`, and `level_rs.spawn` for multiplayer respawns.
///
/// The export takes an entity's name, position and angles from its spawn point, and its volume from its attached
/// shape, over its packet's; the decompile puts the packet's back there.
pub struct LevelEntityConversion;

impl LevelEntityConversion {
  /// The spawn lists the build writes.
  pub const SPAWN_FILE_NAME: &'static str = "level.spawn";
  pub const RESPAWN_FILE_NAME: &'static str = "level_rs.spawn";

  /// The prefix the decompile names an entity's attached shape after.
  pub const SHAPE_PREFIX: &'static str = "shape";

  /// Reserves every entity's name in `sources`, which the export keeps, against generated names. A scene decompile
  /// calls it before any step names an object; [`Self::decompile`] calls it too, for the entities alone.
  pub fn reserve_names<'a>(sources: &LevelEntitySources<'a>, context: &mut LevelDecompileContext) {
    let listed = |file: Option<&'a LevelSpawnFile>| file.into_iter().flat_map(|list| &list.objects);

    for name in listed(sources.spawn)
      .chain(listed(sources.respawn))
      .map(|object| object.name.as_str())
      .chain(sources.root.iter().map(|object| object.header.name.as_str()))
    {
      context.names.reserve(name);
    }
  }

  /// The spawn points of a level's entities: those of `spawn` (or the root spawn's) first, then those of `respawn`
  /// marked to respawn.
  ///
  /// From a root spawn, each object replaces its `level.spawn` packet of the same name, taking that packet's fields
  /// xrAI fills in (`AlifeBuiltFields`), or a fresh entity's where none reads; objects only the root spawn holds
  /// follow. Disagreements are recorded for those, for packets only `level.spawn` holds (left out), and for packets
  /// that then differ. An update packet is kept where it differs from a fresh entity's.
  ///
  /// # Errors
  ///
  /// Returns an error when a packet's fields cannot be saved (a string holding a double quote) or a root spawn object
  /// does not write. A packet of an unknown class is refused on its own instead.
  pub fn decompile<'a>(
    sources: &LevelEntitySources<'a>,
    context: &mut LevelDecompileContext,
  ) -> XrfResult<LevelEntityDecompile> {
    let is_kept = |name: &String| !sources.folded.contains(name);
    let list = |file: Option<&'a LevelSpawnFile>| -> Vec<&'a LevelSpawnObject> {
      file
        .map(|list| list.objects.iter().filter(|object| is_kept(&object.name)).collect())
        .unwrap_or_default()
    };
    let spawn: Vec<&LevelSpawnObject> = list(sources.spawn);
    let respawn: Vec<&LevelSpawnObject> = list(sources.respawn);
    let root: Vec<&AlifeObject> = sources
      .root
      .iter()
      .filter(|object| is_kept(&object.header.name))
      .collect();
    let mut decompile: LevelEntityDecompile = LevelEntityDecompile::default();

    Self::reserve_names(sources, context);

    if sources.root.is_empty() {
      Self::decompile_listed(&spawn, Self::SPAWN_FILE_NAME, 0, &mut decompile, context)?;
    } else {
      Self::decompile_root(&spawn, &root, &mut decompile, context)?;
    }

    Self::decompile_listed(
      &respawn,
      Self::RESPAWN_FILE_NAME,
      EditableSpawnPointObject::RESPAWN_ENTITY,
      &mut decompile,
      context,
    )?;

    Ok(decompile)
  }

  /// A spawn list's packets as spawn points of `flags`, those of no known class refused.
  fn decompile_listed(
    objects: &[&LevelSpawnObject],
    file: &str,
    flags: u8,
    decompile: &mut LevelEntityDecompile,
    context: &mut LevelDecompileContext,
  ) -> XrfResult {
    for object in objects {
      match object.read_alife_object() {
        Ok(entity) => decompile
          .entities
          .push(Self::decompile_entity(&entity, flags, context)?),
        Err(error) => decompile.refused.push(Self::refuse(object, file, &error)),
      }
    }

    Ok(())
  }

  /// `level.spawn`'s packets in order, each replaced by the root spawn's object of its name, then the root spawn's
  /// other objects. Of the packets the root spawn lacks, only graph points are kept.
  fn decompile_root(
    listed: &[&LevelSpawnObject],
    root: &[&AlifeObject],
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
        Self::decompile_built(root[index], Some(*object), decompile, context)?;
        continue;
      }

      match object.read_alife_object() {
        Ok(entity) if matches!(entity.inherited, AlifeObjectInherited::CseAlifeGraphPoint(_)) => {
          decompile.entities.push(Self::decompile_entity(&entity, 0, context)?);
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
      Self::decompile_built(object, None, decompile, context)?;
    }

    Ok(())
  }

  /// One root spawn object as a spawn point, compared with its `level.spawn` packet where there is one.
  fn decompile_built(
    built: &AlifeObject,
    listed: Option<&LevelSpawnObject>,
    decompile: &mut LevelEntityDecompile,
    context: &mut LevelDecompileContext,
  ) -> XrfResult {
    let name: &str = &built.header.name;
    let mut entity: AlifeObject = built.clone();
    // xrAI overwrites these, so they come from the editor's packet in `level.spawn` where one reads, else a fresh
    // entity's.
    let editor_fields: AlifeBuiltFields = listed
      .and_then(|listed| listed.read_alife_object().ok())
      .map_or(AlifeBuiltFields::UNBUILT, |listed| listed.get_built_fields());

    entity.set_built_fields(&editor_fields);

    let disagreement: Option<&str> = match listed {
      None => Some("only in the root spawn"),
      // Compared as bytes: as floats, a negative zero would equal a positive one.
      Some(listed) if Self::get_spawn_packet(&entity)? != Self::get_listed_packet(listed)? => {
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

    let stored: Vec<u8> = Self::get_update_packet(built)?;

    if stored != Self::get_update_packet(&built.get_fresh_entity()?)? {
      decompile.update_packets.push(LevelSceneUpdatePacket {
        object: name.to_owned(),
        bytes: Sidecar::format_hex(&stored),
      });
    }

    decompile.entities.push(Self::decompile_entity(&entity, 0, context)?);

    Ok(())
  }

  fn get_update_packet(object: &AlifeObject) -> XrfResult<Vec<u8>> {
    let mut bytes: ChunkWriter = ChunkWriter::new();

    object.write_update_packet(&mut bytes)?;

    Ok(bytes.buffer)
  }

  fn get_spawn_packet(object: &AlifeObject) -> XrfResult<Vec<u8>> {
    let mut bytes: ChunkWriter = ChunkWriter::new();

    object.write_spawn_packet(&mut bytes)?;

    Ok(bytes.buffer)
  }

  fn get_listed_packet(object: &LevelSpawnObject) -> XrfResult<Vec<u8>> {
    let mut bytes: ChunkWriter = ChunkWriter::new();

    object.write(&mut bytes)?;

    Ok(bytes.buffer)
  }

  fn refuse(object: &LevelSpawnObject, file: &str, error: &XrfError) -> LevelSceneRefusal {
    LevelSceneRefusal {
      file: file.to_owned(),
      object: object.name.clone(),
      reason: format!("section '{}': {error}", object.section),
    }
  }

  /// The spawn lists the part's entities export to, numbered from zero in the part's order within each list.
  ///
  /// # Errors
  ///
  /// Returns an error naming a spawn point whose data does not read as its section's class, whose class has a volume
  /// but no attached shape (as the build refuses it), or whose packet does not write.
  pub fn export(part: &EditableObjectPart<EditableSpawnPointObject>) -> XrfResult<LevelEntityExport> {
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
        Self::export_entity(point, section, data, id)
          .map_err(|error| XrfError::new_invalid_error(format!("Spawn point '{}': {error}", point.object.name)))?,
      );
    }

    let into_file = |objects: Vec<LevelSpawnObject>| (!objects.is_empty()).then_some(LevelSpawnFile { objects });

    Ok(LevelEntityExport {
      spawn: into_file(spawn),
      respawn: into_file(respawn),
    })
  }

  /// The spawn point of a typed packet, at its place under its name, a class's volume attached as an unturned shape.
  fn decompile_entity(
    entity: &AlifeObject,
    flags: u8,
    context: &mut LevelDecompileContext,
  ) -> XrfResult<EditableSpawnPointObject> {
    let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();
    let mut data: IniSection = IniSection::new("spawndata");

    entity.write_spawn_packet(&mut recorder)?;
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

  /// One entity's packet as `ExportGame` writes it: the spawn data with the spawn point's name, position and angles,
  /// and a class's volume from the attached shape, scaled.
  fn export_entity(
    point: &EditableSpawnPointObject,
    section: &str,
    data: &IniSection,
    id: u32,
  ) -> XrfResult<LevelSpawnObject> {
    let mut entity: AlifeObject = LevelSpawnDataFields::read_entity(data)?;

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

    LevelSpawnObject::from_alife_object(id, &entity)
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

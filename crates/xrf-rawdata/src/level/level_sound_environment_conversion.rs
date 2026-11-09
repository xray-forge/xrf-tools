use xrf_editable_level::{
  EditableCustomObject, EditableLevelGuid, EditableLevelValue, EditableObjectPart, EditableObjectTransform,
  EditableShape, EditableShapeElement, EditableSoundEnvironmentObject,
};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelSndEnvFace, LevelSndEnvFile};
use xrf_math::{Aabb, Affine3d, EPS, Matrix4x4, Vector3d};

use crate::level::level_object_names::LevelObjectNames;

/// A level's reverb volumes, between the scene's `sound_env.part` and the compiled `level.snd_env`.
///
/// The CoP SDK's Build writes no `level.snd_env`: its exporter, `CLevelSoundManager::MakeEnvGeometry`, is commented
/// out (`SoundManager_LE.cpp:51`, `BuilderGame.cpp:72`), and IX-Ray's editor runs the same code. This ports it, and
/// every shipped file is laid out as it writes: per exported volume, its box's 8 corners and 12 faces naming the
/// volume's outer environment in front and its inner one behind.
pub struct LevelSoundEnvironmentConversion;

impl LevelSoundEnvironmentConversion {
  /// `SOUND_ENV_TOOLS_VERSION`, the sound environment tool's `[main]` version.
  pub const TOOL_VERSION: u32 = 0;

  /// `CFORM_CURRENT_VERSION`, the mesh header's version.
  pub const MESH_VERSION: u32 = 4;

  /// The editor's prefix for a new volume's name, the tool's class name.
  pub const NAME_PREFIX: &'static str = "sound_env";

  /// `du_box_faces`, the box's 12 triangles by corner of [`EditableShape::UNIT_BOX_CORNERS`].
  const BOX_FACES: [[usize; 3]; 12] = [
    [0, 1, 2],
    [2, 3, 0],
    [3, 2, 6],
    [6, 7, 3],
    [6, 4, 5],
    [6, 5, 7],
    [4, 1, 5],
    [1, 0, 5],
    [3, 5, 0],
    [3, 7, 5],
    [1, 4, 6],
    [1, 6, 2],
  ];

  /// The sound environment part a `level.snd_env` was exported from, a volume per box named as the editor names a new
  /// one. A box comes back as the editor's own volume, its unit box placed by the object's transform, when that
  /// transform saved as text composes into the same corners; otherwise, as for a box sheared by a scaled turn, the
  /// object stands at the origin and the box's own matrix holds it, chosen so its saved text gives the same corners.
  /// The mesh header's bounds are not kept: the export computes them, over volumes it skips too.
  ///
  /// # Errors
  ///
  /// Returns an error when the mesh is not whole boxes in the exporter's layout, or a box's faces name different
  /// environments.
  pub fn decompile(
    file: &LevelSndEnvFile,
    guid: EditableLevelGuid,
    names: &mut LevelObjectNames,
  ) -> XrfResult<EditableObjectPart<EditableSoundEnvironmentObject>> {
    let corner_count: usize = EditableShape::UNIT_BOX_CORNERS.len();

    if !file.faces.len().is_multiple_of(Self::BOX_FACES.len())
      || file.vertices.len() != file.faces.len() / Self::BOX_FACES.len() * corner_count
    {
      return Err(XrfError::new_invalid_error(format!(
        "Sound environment mesh of {} vertices and {} faces is not whole boxes",
        file.vertices.len(),
        file.faces.len()
      )));
    }

    let slots: [usize; 8] = Self::get_corner_slots();
    let objects: Vec<EditableSoundEnvironmentObject> = file
      .faces
      .chunks(Self::BOX_FACES.len())
      .enumerate()
      .map(|(index, faces)| {
        let base: usize = index * corner_count;
        let first: &LevelSndEnvFace = &faces[0];
        let in_layout: bool = faces.iter().zip(Self::BOX_FACES).all(|(face, corners)| {
          face.front == first.front
            && face.back == first.back
            && face
              .vertices
              .iter()
              .zip(corners)
              .all(|(vertex, corner)| *vertex as usize == base + slots[corner])
        });

        if !in_layout {
          return Err(XrfError::new_invalid_error(format!(
            "Sound environment box {index} is not one the editor's export lays out"
          )));
        }

        let corners: [Vector3d; 8] = std::array::from_fn(|corner| file.vertices[base + slots[corner]]);
        let (transform, shape) = Self::get_placement(&corners);

        Ok(EditableSoundEnvironmentObject {
          object: EditableCustomObject::new(names.get_next(Self::NAME_PREFIX), transform),
          shape,
          inner: file.environments[usize::from(first.back)].clone(),
          outer: file.environments[usize::from(first.front)].clone(),
        })
      })
      .collect::<XrfResult<Vec<EditableSoundEnvironmentObject>>>()?;

    Ok(EditableObjectPart::new(guid, Self::TOOL_VERSION, None, objects))
  }

  /// The `level.snd_env` the export writes from the part, `MakeEnvGeometry`: the environments in the order volumes
  /// first name them, inner before outer; each exported volume's box corners, merged with an earlier one within `EPS`
  /// as `CDB::Collector::VPack` does, and faces; and bounds over every volume, exported or not. None when no volume is
  /// exported, for which it writes no file.
  ///
  /// # Errors
  ///
  /// Returns an error, where the editor asserts, naming a volume of no shape, or an exported one whose shape does not
  /// start with a box.
  pub fn export(part: &EditableObjectPart<EditableSoundEnvironmentObject>) -> XrfResult<Option<LevelSndEnvFile>> {
    let mut environments: Vec<String> = Vec::new();
    let mut vertices: Vec<Vector3d> = Vec::new();
    let mut faces: Vec<LevelSndEnvFace> = Vec::new();
    let mut bounds: Option<Aabb> = None;

    for volume in &part.objects {
      let volume_bounds: Aabb = volume.get_bounds().ok_or_else(|| {
        XrfError::new_invalid_error(format!("Sound environment '{}' has no shape", volume.object.name))
      })?;

      match &mut bounds {
        Some(bounds) => bounds.include_box(&volume_bounds),
        None => bounds = Some(volume_bounds),
      }

      if !volume.is_exported() {
        continue;
      }

      let matrix: Matrix4x4 = volume.get_box().ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Sound environment '{}' has no box to export",
          volume.object.name
        ))
      })?;
      let back: u16 = Self::get_environment_index(&mut environments, &volume.inner)?;
      let front: u16 = Self::get_environment_index(&mut environments, &volume.outer)?;
      let corners: [Vector3d; 8] = EditableShape::place_unit_box(&matrix);

      for face in Self::BOX_FACES {
        faces.push(LevelSndEnvFace {
          vertices: face.map(|corner| Self::pack_vertex(&mut vertices, &corners[corner])),
          front,
          back,
        });
      }
    }

    Ok(
      bounds
        .filter(|_| !environments.is_empty())
        .map(|bounds| LevelSndEnvFile {
          environments,
          version: Self::MESH_VERSION,
          bounds,
          vertices,
          faces,
        }),
    )
  }

  /// Each box corner's slot among a box's 8 vertices, numbered as `VPack` first meets them over the faces.
  fn get_corner_slots() -> [usize; 8] {
    let mut slots: [usize; 8] = [usize::MAX; 8];
    let mut next: usize = 0;

    for corner in Self::BOX_FACES.as_flattened() {
      if slots[*corner] == usize::MAX {
        slots[*corner] = next;
        next += 1;
      }
    }

    slots
  }

  /// The transform and shape that place a box's corners exactly once saved, the editor's own form first.
  fn get_placement(corners: &[Vector3d; 8]) -> (EditableObjectTransform, EditableShape) {
    let affine: Affine3d = Affine3d {
      x_axis: &corners[3] - &corners[0],
      y_axis: &corners[1] - &corners[0],
      z_axis: &corners[5] - &corners[0],
      translation: corners[0].midpoint(&corners[6]),
    };

    let natural: Option<EditableObjectTransform> = EditableObjectTransform::from_matrix(&Matrix4x4::from(&affine))
      .ok()
      .and_then(|transform| transform.find_saved(|transform| Self::is_placing(&transform.to_matrix(), corners)));

    if let Some(transform) = natural {
      return (transform, EditableShape::new_unit_box());
    }

    let rows: [Vector3d; 4] = [affine.x_axis, affine.y_axis, affine.z_axis, affine.translation];
    let mut saved: [[f32; 3]; 4] = rows.map(|row| EditableLevelValue::round_vector3_as_saved(&row).to_array());

    // Each world axis of a corner sums only that axis of the four rows, so each is searched on its own.
    for axis in 0..3 {
      if let Some(found) = Self::find_axis(&rows, corners, axis) {
        for (row, value) in saved.iter_mut().zip(found) {
          row[axis] = value;
        }
      }
    }

    let [x_axis, y_axis, z_axis, translation] = saved.map(Vector3d::from);

    (
      EditableObjectTransform::default(),
      EditableShape {
        usage: 0,
        elements: vec![EditableShapeElement::Box(Affine3d {
          x_axis,
          y_axis,
          z_axis,
          translation,
        })],
      },
    )
  }

  /// The saved values of the four rows' `axis` components that give every corner's `axis` exactly, as
  /// `Fmatrix::transform_tiny` sums them.
  fn find_axis(rows: &[Vector3d; 4], corners: &[Vector3d; 8], axis: usize) -> Option<[f32; 4]> {
    let candidates: [[f32; 5]; 4] = rows.map(|row| EditableLevelValue::get_saved_neighbours(row.to_array()[axis]));
    let fits = |[i, j, k, c]: [f32; 4]| {
      EditableShape::UNIT_BOX_CORNERS
        .iter()
        .zip(corners)
        .all(|([x, y, z], placed)| (x * i + y * j + z * k + c).to_bits() == placed.to_array()[axis].to_bits())
    };

    candidates[0].iter().find_map(|&i| {
      candidates[1].iter().find_map(|&j| {
        candidates[2]
          .iter()
          .find_map(|&k| candidates[3].iter().map(|&c| [i, j, k, c]).find(|values| fits(*values)))
      })
    })
  }

  fn is_placing(matrix: &Matrix4x4, corners: &[Vector3d; 8]) -> bool {
    EditableShape::place_unit_box(matrix)
      .iter()
      .zip(corners)
      .all(|(ours, placed)| ours.to_array().map(f32::to_bits) == placed.to_array().map(f32::to_bits))
  }

  /// `VPack`: the index of an earlier vertex within `EPS` on every axis, else of the vertex appended.
  fn pack_vertex(vertices: &mut Vec<Vector3d>, vertex: &Vector3d) -> u32 {
    let index: usize = vertices
      .iter()
      .position(|it| (it.x - vertex.x).abs() < EPS && (it.y - vertex.y).abs() < EPS && (it.z - vertex.z).abs() < EPS)
      .unwrap_or_else(|| {
        vertices.push(*vertex);
        vertices.len() - 1
      });

    index as u32
  }

  /// An environment's index among those named so far, appended when new.
  fn get_environment_index(environments: &mut Vec<String>, environment: &str) -> XrfResult<u16> {
    let index: usize = environments.iter().position(|it| it == environment).unwrap_or_else(|| {
      environments.push(environment.to_owned());
      environments.len() - 1
    });

    u16::try_from(index)
      .map_err(|_| XrfError::new_invalid_error(format!("Sound environments past {} cannot be named", u16::MAX)))
  }
}

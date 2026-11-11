use byteorder::ByteOrder;
use xrf_chunk::ChunkLayout;
use xrf_editable_level::EditableSceneObject;
use xrf_editable_object::{
  EditableBone, EditableBoneBindPose, EditableBoneBreak, EditableBoneDefinition, EditableBoneJoint, EditableBoneMass,
  EditableObjectFile,
};
use xrf_error::XrfResult;
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_math::{Aabb, Affine3d, Cylinder, Obb, Sphere, Vector3d};
use xrf_skeleton::{SkeletonBoneShape, SkeletonBoneShapeKind, SkeletonJointKind, SkeletonJointLimit};
use xrf_spawn::{
  AlifeObject, AlifeObjectAbstract, AlifeObjectBreakable, AlifeObjectDynamicVisual, AlifeObjectInherited, ClsId,
  SpawnPacketHeader,
};

use crate::level::build::level_build_parts::{LevelBuildPartBox, LevelBuildParts};
use crate::level::build::level_build_vector::LevelBuildVector;
use crate::level::level_breakable_conversion::LevelBreakableConversion;
use crate::object::compile::bounds::bone_boxes::BoneBoxes;
use crate::object::compile::geometry::export_vertex_bone::ExportVertexBone;
use crate::object::compile::object_compile_options::TangentMethod;
use crate::object::compile::object_compile_source::ObjectCompileSource;
use crate::object::compile::skeleton::skeleton_geometry::SkeletonGeometry;
use crate::object::compile::skeleton::skeleton_geometry_builder::SkeletonGeometryBuilder;
use crate::object::compile::skeleton::skeleton_ogf_writer::SkeletonOgfWriter;
use crate::object::compile::skeleton::skin_corner::SkinCorner;

/// The breakables a build cuts out of the geometry (`ESceneObjectTool::ExportBreakableObjects`,
/// `ESceneObjectToolsExportBreakable.cpp:105`): the parts of the static objects' faces of a breakable game material
/// ([`LevelBuildParts`]), each moved into the frame of its box (`ComputeOBB_RAPID`) and cut into fragments grown at
/// random (`SBPart::prepare`, `GeometryPartExtractor.cpp:51`), written as a rigid skeleton of a bone per fragment,
/// `meshes\brkbl#<n>.ogf` (`SBPart::Export`), and placed by a `breakable_object` standing at the box.
pub(crate) struct LevelBuildBreakables {
  parts: LevelBuildParts,
  random: EditorRandom,
}

/// The breakables a build made: each part's skeleton, the entities placing them, and the parts it could not make one
/// of.
pub(crate) struct LevelBuildBreakableExport {
  pub(crate) parts: Vec<LevelBuildBreakablePart>,
  pub(crate) entities: Vec<AlifeObject>,
  /// Each part refused, by its name, with why.
  pub(crate) refused: Vec<(String, String)>,
}

/// One part's rigid skeleton, kept unwritten so that it takes the byte order of the level's other files.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LevelBuildBreakablePart {
  /// Its path in the level's directory, `meshes/brkbl#<n>.ogf`.
  pub(crate) path: String,
  object: EditableObjectFile,
  geometry: SkeletonGeometry,
}

impl LevelBuildBreakablePart {
  /// The part's OGF in `T`'s byte order, with a description of nothing (`ogf_desc`'s defaults).
  ///
  /// # Errors
  ///
  /// Returns an error when the skeleton does not write.
  pub(crate) fn write<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    SkeletonOgfWriter::write::<T>(
      &self.object,
      &self.geometry,
      &ObjectCompileSource {
        source_file: String::new(),
        convertor: String::new(),
        built_at: 0,
      },
    )
  }
}

/// One fragment of a part, `SBBone`: the game material of the face it grew from, how many faces it grew and their area,
/// and its offset from the first fragment.
struct Fragment {
  game_material: String,
  faces: usize,
  area: f32,
  offset: Vector3d,
}

/// A part moved into its box's frame, `SBPart` after `prepare`: its faces' corners and normals there, each face's
/// fragment, and the fragments.
struct PreparedPart {
  frame: Affine3d,
  corners: Vec<[Vector3d; 3]>,
  normals: Vec<[Vector3d; 3]>,
  bounds: Aabb,
  bone_of_face: Vec<usize>,
  fragments: Vec<Fragment>,
}

impl Default for LevelBuildBreakables {
  fn default() -> Self {
    Self {
      parts: LevelBuildParts::new(GameMtlMaterial::FLAG_BREAKABLE),
      random: EditorRandom::default(),
    }
  }
}

impl LevelBuildBreakables {
  /// The class every breakable is of.
  const SECTION: &'static str = "breakable_object";

  /// `per_bone_face_count_min`, the fewest faces a fragment grows (`Initialize(bb, EPS_L, 2)`).
  const FRAGMENT_FACES_MIN: usize = 2;

  /// Fragments a part is cut into at most, which keeps its skeleton within the 63 bones the editor's export takes.
  const FRAGMENTS_MAX: usize = 62;

  /// `SJointIKData::flBreakable`.
  const BREAKABLE_JOINT_FLAG: u32 = 1 << 0;

  /// Adds every face of a breakable game material of `object`, placed by `placed` ([`LevelBuildParts::add_object`]).
  ///
  /// # Errors
  ///
  /// As [`LevelBuildParts::add_object`].
  pub(crate) fn add_object(
    &mut self,
    placed: &EditableSceneObject,
    object: &EditableObjectFile,
    game_materials: &GameMtlFile,
  ) -> XrfResult {
    self.parts.add_object(placed, object, game_materials)
  }

  /// A breakable of each part, numbered by part; a part some face of which shares no edge with another refused as the
  /// Build refuses it.
  ///
  /// # Errors
  ///
  /// Returns an error where a part's skeleton does not build.
  pub(crate) fn into_export(mut self) -> XrfResult<LevelBuildBreakableExport> {
    let mut export: LevelBuildBreakableExport = LevelBuildBreakableExport {
      parts: Vec::new(),
      entities: Vec::new(),
      refused: Vec::new(),
    };

    for (index, part) in self.parts.get_parts().iter().enumerate() {
      let (name, path) = LevelBreakableConversion::get_part_names(index);

      match self.prepare(part) {
        Ok(prepared) => {
          export.parts.push(self.build_part(path, part, &prepared)?);
          export.entities.push(Self::get_entity(&name, &prepared.frame));
        }
        Err(reason) => export.refused.push((name, reason)),
      }
    }

    Ok(export)
  }

  /// `SBPart::prepare`: `part` in the frame of its box, and its fragments, each grown from a face with fewer than two
  /// neighbours free or with none, to a size drawn at random, faces left over joining a neighbour's fragment. As the
  /// editor does, the first pass over the leftovers gives every one of them to the first fragment, and an offset sums
  /// every face of its fragment but divides by the faces it grew.
  fn prepare(&mut self, part: &[usize]) -> Result<PreparedPart, String> {
    let neighbours: Vec<Vec<usize>> = self.part_neighbours(part);

    if neighbours.iter().any(Vec::is_empty) {
      return Err(String::from(
        "a face of it shares no edge with another (\"Error face found\")",
      ));
    }

    let fitted: LevelBuildPartBox = self
      .parts
      .fit_box(part)
      .ok_or_else(|| String::from("its corners hold no box"))?;
    let to_vector = |value: [f64; 3]| Vector3d::new(value[0] as f32, value[1] as f32, value[2] as f32);
    let frame: Affine3d = Affine3d {
      x_axis: to_vector(fitted.axes[0]),
      y_axis: to_vector(fitted.axes[1]),
      z_axis: to_vector(fitted.axes[2]),
      translation: to_vector(fitted.centre),
    };
    let into_frame = |point: &Vector3d, is_point: bool| -> Vector3d {
      let value: [f64; 3] = [f64::from(point.x), f64::from(point.y), f64::from(point.z)];
      let local: [f64; 3] = if is_point {
        LevelBuildVector::sub(&value, &fitted.centre)
      } else {
        value
      };

      to_vector(fitted.axes.map(|axis| LevelBuildVector::dot(&axis, &local)))
    };
    let corners: Vec<[Vector3d; 3]> = part
      .iter()
      .map(|face| self.parts.faces[*face].corners.map(|corner| into_frame(&corner, true)))
      .collect();
    let normals: Vec<[Vector3d; 3]> = part
      .iter()
      .map(|face| self.parts.faces[*face].normals.map(|normal| into_frame(&normal, false)))
      .collect();
    let bounds: Aabb =
      Aabb::from_points(corners.iter().flatten().copied()).ok_or_else(|| String::from("it has no corner"))?;
    let (bone_of_face, fragments) = self.grow_fragments(part, &neighbours, &corners);

    // A part whose every face has a neighbour grows a first fragment of two faces or more.
    if fragments.is_empty() {
      return Err(String::from("it grows no fragment"));
    }

    Ok(PreparedPart {
      frame,
      corners,
      normals,
      bounds,
      bone_of_face,
      fragments,
    })
  }

  /// Each face of `part`'s neighbours across an edge, as positions in `part`.
  fn part_neighbours(&self, part: &[usize]) -> Vec<Vec<usize>> {
    self
      .parts
      .get_edge_neighbours(part)
      .into_iter()
      .map(|faces| {
        faces
          .into_iter()
          .filter_map(|face| part.iter().position(|it| *it == face))
          .collect()
      })
      .collect()
  }

  /// The fragments of a part of `corners` and `neighbours`, and each face's fragment.
  fn grow_fragments(
    &mut self,
    part: &[usize],
    neighbours: &[Vec<usize>],
    corners: &[[Vector3d; 3]],
  ) -> (Vec<usize>, Vec<Fragment>) {
    let count: usize = part.len();
    let fragments_max: usize = (count / Self::FRAGMENT_FACES_MIN).clamp(1, Self::FRAGMENTS_MAX);
    let fragment_faces_max: usize = ((count as f32 / fragments_max as f32 + 0.5).floor() as usize) * 4;
    let mut growth: Growth<'_> = Growth {
      marked: vec![false; count],
      bone: vec![None; count],
      neighbours,
      corners,
    };
    let mut fragments: Vec<Fragment> = Vec::new();
    let mut taken: usize = 0;

    while fragments.len() < fragments_max {
      // The first free face with no free neighbour or two, else the last free one.
      let Some(seed) = (0..count)
        .filter(|face| !growth.marked[*face])
        .find(|face| {
          let free: usize = neighbours[*face].iter().filter(|it| !growth.marked[**it]).count();

          free == 0 || free >= 2
        })
        .or_else(|| (0..count).rev().find(|face| !growth.marked[*face]))
      else {
        break;
      };
      let limit: usize = self
        .random
        .get_between(Self::FRAGMENT_FACES_MIN, fragment_faces_max + 1);
      let (mut grown, mut area) = (0, 0.0);

      growth.grow(seed, fragments.len(), limit, &mut grown, &mut area);

      if grown == 1 {
        growth.bone[seed] = None;
      } else {
        fragments.push(Fragment {
          game_material: self.parts.surfaces[self.parts.faces[part[seed]].surface]
            .game_material
            .clone(),
          faces: grown,
          area,
          offset: Vector3d::ZERO,
        });
        taken += grown;
      }
    }

    // `face_accum_total_saved` starts equal, so the first pass hands every face left to the first fragment.
    if taken < count {
      for bone in &mut growth.bone {
        if bone.is_none() {
          *bone = Some(0);
        }
      }
    }

    let bone_of_face: Vec<usize> = growth.bone.iter().map(|bone| bone.unwrap_or(0)).collect();
    let mut sums: Vec<Vector3d> = vec![Vector3d::ZERO; fragments.len()];

    for (face, bone) in bone_of_face.iter().enumerate() {
      if let Some(sum) = sums.get_mut(*bone) {
        for corner in &corners[face] {
          *sum = &*sum + corner;
        }
      }
    }

    for (fragment, sum) in fragments.iter_mut().zip(&sums) {
      fragment.offset = sum * (1.0 / (fragment.faces * 3) as f32);
    }

    if let Some(first) = fragments.first().map(|it| it.offset) {
      for fragment in &mut fragments {
        fragment.offset = &fragment.offset - &first;
      }
    }

    (bone_of_face, fragments)
  }

  /// `SBPart::Export`: a rigid skeleton of a bone per fragment, the first the root and every other its child, each face
  /// following its fragment's bone alone, into a child per engine shader and texture; each bone's box the one around its
  /// vertices, which its physics shape repeats, breaking off when struck, its mass the area it grew.
  fn build_part(&self, path: String, part: &[usize], prepared: &PreparedPart) -> XrfResult<LevelBuildBreakablePart> {
    let mut builder: SkeletonGeometryBuilder = SkeletonGeometryBuilder::of_faces(prepared.bounds);

    for (index, face) in part.iter().enumerate() {
      let held = &self.parts.faces[*face];
      let corners: [SkinCorner; 3] = std::array::from_fn(|corner| SkinCorner {
        position: prepared.corners[index][corner],
        normal: prepared.normals[index][corner],
        texture_coordinate: held.texture_coordinates[corner],
        weights: vec![ExportVertexBone {
          bone: prepared.bone_of_face[index] as u16,
          weight: 1.0,
        }],
      });

      builder.add_face(&self.parts.surfaces[held.surface], &corners)?;
    }

    let geometry: SkeletonGeometry = builder.finish(TangentMethod::ActorEditor)?;
    let mut bones: Vec<EditableBone> = prepared
      .fragments
      .iter()
      .enumerate()
      .map(|(index, fragment)| Self::get_bone(index, fragment))
      .collect();

    let boxes: Vec<Obb> = BoneBoxes::of(&bones, &geometry.splits);

    for (bone, obb) in bones.iter_mut().zip(boxes) {
      bone.mass = Some(EditableBoneMass {
        mass: bone.mass.as_ref().map_or(0.0, |it| it.mass),
        center: obb.center,
      });
      bone.shape.obb = obb;
    }

    Ok(LevelBuildBreakablePart {
      path,
      object: EditableObjectFile {
        flags: 0,
        flags_size: 4,
        class_script: None,
        lods: None,
        meshes: None,
        surfaces: None,
        bones: Some(bones),
        embedded_motions: None,
        motion_references: None,
        bone_parts: None,
        actor_transform: None,
        description: None,
        layout: ChunkLayout::default(),
      },
      geometry,
    })
  }

  /// A fragment's bone: named by its number, the first the root, its joint none and breakable, its mass its area.
  fn get_bone(index: usize, fragment: &Fragment) -> EditableBone {
    let limit: SkeletonJointLimit = SkeletonJointLimit {
      min: 0.0,
      max: 0.0,
      spring_factor: 1.0,
      damping_factor: 1.0,
    };

    EditableBone {
      version: 2,
      definition: EditableBoneDefinition {
        name: index.to_string(),
        parent: if index == 0 { String::new() } else { String::from("0") },
        weight_map: String::new(),
      },
      bind_pose: EditableBoneBindPose {
        offset: fragment.offset,
        rotation: Vector3d::ZERO,
        length: 0.0,
      },
      data_name: None,
      game_material: fragment.game_material.clone(),
      shape: SkeletonBoneShape {
        kind: SkeletonBoneShapeKind::Box,
        flags: SkeletonBoneShape::REMOVE_AFTER_BREAK_FLAG,
        obb: Obb {
          rotation: [Vector3d::X, Vector3d::Y, Vector3d::Z],
          center: Vector3d::ZERO,
          half_extents: Vector3d::ZERO,
        },
        sphere: Sphere {
          center: Vector3d::ZERO,
          radius: 0.0,
        },
        cylinder: Cylinder {
          center: Vector3d::ZERO,
          direction: Vector3d::ZERO,
          height: 0.0,
          radius: 0.0,
        },
      },
      joint_flags: Some(Self::BREAKABLE_JOINT_FLAG),
      joint: EditableBoneJoint {
        kind: SkeletonJointKind::None,
        limits: [limit.clone(), limit.clone(), limit],
        spring_factor: 1.0,
        damping_factor: 1.0,
      },
      joint_break: Some(EditableBoneBreak {
        force: 0.0,
        torque: 0.0,
      }),
      friction: Some(0.0),
      mass: Some(EditableBoneMass {
        mass: fragment.area,
        center: Vector3d::ZERO,
      }),
      layout: ChunkLayout::default(),
    }
  }

  /// The `breakable_object` placing the part `name` at `frame`: named by its file, its visual the same without the
  /// extension (`CSE_Visual::set_visual`), standing at the box with the angles `getXYZ` reads off its axes; the rest as
  /// GSC's build left it in vanilla's packets.
  fn get_entity(name: &str, frame: &Affine3d) -> AlifeObject {
    AlifeObject {
      header: SpawnPacketHeader {
        id: u16::MAX,
        net_action: 1,
        section: String::from(Self::SECTION),
        name: format!("{name}.ogf"),
        script_game_id: 0,
        script_rp: 0xFE,
        position: frame.translation,
        direction: frame.get_xyz(),
        respawn_time: 0,
        parent_id: u16::MAX,
        phantom_id: u16::MAX,
        script_flags: 33,
        version: 128,
        game_type: u16::MAX,
        script_version: 12,
        client_data: Vec::new(),
        spawn_id: u16::MAX,
      },
      clsid: ClsId::OBrkbl,
      inherited: AlifeObjectInherited::CseAlifeObjectBreakable(Box::new(AlifeObjectBreakable {
        base: AlifeObjectDynamicVisual {
          base: AlifeObjectAbstract {
            game_vertex_id: u16::MAX,
            distance: 0.0,
            direct_control: 1,
            level_vertex_id: u32::MAX,
            flags: 0xFFFF_FFBA,
            custom_data: String::new(),
            story_id: u32::MAX,
            spawn_story_id: u32::MAX,
          },
          visual_name: name.to_lowercase(),
          visual_flags: 0,
        },
        health: 1.0,
      })),
    }
  }
}

/// A part's fragments as they grow: which faces are taken, by which fragment, and what grows them.
struct Growth<'a> {
  marked: Vec<bool>,
  bone: Vec<Option<usize>>,
  neighbours: &'a [Vec<usize>],
  corners: &'a [[Vector3d; 3]],
}

impl Growth<'_> {
  /// `recurse_fragment`: takes `face`, then its free neighbours while the fragment is under `limit`, then grows on from
  /// each of those it took.
  fn grow(&mut self, face: usize, bone: usize, limit: usize, grown: &mut usize, area: &mut f32) {
    if !self.marked[face] {
      self.take(face, bone, grown, area);
    }

    let mut taken: Vec<usize> = Vec::new();

    for neighbour in &self.neighbours[face] {
      if *grown >= limit {
        break;
      }

      if self.marked[*neighbour] {
        continue;
      }

      self.take(*neighbour, bone, grown, area);
      taken.push(*neighbour);
    }

    for neighbour in taken {
      if *grown >= limit {
        break;
      }

      if self.bone[neighbour] == Some(bone) {
        self.grow(neighbour, bone, limit, grown, area);
      }
    }
  }

  /// `use_face`: the face taken by fragment `bone`, its area (`SBFace::CalcArea`) added.
  fn take(&mut self, face: usize, bone: usize, grown: &mut usize, area: &mut f32) {
    let [first, second, third] = &self.corners[face];
    let (along, across) = (second - first, third - first);
    let (length, other) = (along.dot(&along), across.dot(&across));

    self.marked[face] = true;
    self.bone[face] = Some(bone);
    *area += 0.5 * (length * other - along.dot(&across).powi(2)).max(0.0).sqrt();
    *grown += 1;
  }
}

/// The editor's `CRandom` (`xrCore/_random.h`), seeded at zero: a fragment's size is drawn from it, so a part is cut
/// the same way every build of ours, though not as GSC's builds cut it.
#[derive(Default)]
struct EditorRandom {
  state: u32,
}

impl EditorRandom {
  /// `randI(min, max)`: from `min` up to `max`, `max` itself excluded.
  fn get_between(&mut self, min: usize, max: usize) -> usize {
    self.state = self.state.wrapping_mul(214_013).wrapping_add(2_531_011);

    let drawn: usize = ((self.state >> 16) & 0x7FFF) as usize;

    min + drawn % max.saturating_sub(min).max(1)
  }
}

use std::collections::BTreeMap;
use std::fmt;

use oxijolt::{CompoundChild, DroppedTriangles, MeshSettings, Quat, Shape, Vec3};
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_level::{LevelCformFace, LevelCformGeometry};
use xrf_math::Vector3d;

/// A level's collision form as the physics engine collides with it, built off the game loop (it takes a second or two
/// for a large level) and then added to a [`crate::PhysicsWorld`]: a static compound of one triangle mesh a game
/// material, each child carrying its material's id, so what touches the level learns what it is made of.
pub struct PhysicsLevel {
  pub(crate) shape: Shape,
  /// How many faces were left out as passable: bushes and the like, which X-Ray's actor walks through.
  passable: usize,
  /// How many faces the engine dropped as too small or thin to collide with reliably.
  dropped: usize,
}

impl PhysicsLevel {
  /// The collision of `geometry`, a level's `level.cform`, in the engine's space, its faces' game materials read in
  /// `materials` (the tree's `gamemtl.xr`).
  ///
  /// A face of a passable material is left out, as X-Ray's physics makes no contact with one (`Physics.cpp`,
  /// `flPassable`), unless the material is also an actor obstacle, which its actor always collides with
  /// (`PHActorCharacter.cpp`, `flActorObstacle`). A face of a material the library does not hold is kept.
  ///
  /// # Errors
  ///
  /// Returns an error when no face is left or the engine refuses a mesh.
  pub fn of_cform(geometry: &LevelCformGeometry, materials: &GameMtlFile) -> XrfResult<Self> {
    let passable: Vec<bool> = Self::find_passable(materials);
    let mut by_material: BTreeMap<u16, Vec<&LevelCformFace>> = BTreeMap::new();

    for face in geometry.get_faces() {
      if !passable.get(usize::from(face.material)).copied().unwrap_or(false) {
        by_material.entry(face.material).or_default().push(face);
      }
    }

    let kept: usize = by_material.values().map(Vec::len).sum();
    let mut remap: Vec<u32> = vec![u32::MAX; geometry.get_vertices().len()];
    let mut dropped: usize = 0;
    // One mesh a material, in the materials' order.
    let mut meshes: Vec<Shape> = Vec::with_capacity(by_material.len());

    for (material, faces) in &by_material {
      let (mesh, lost): (Shape, DroppedTriangles) = Self::build_mesh(geometry.get_vertices(), faces, &mut remap)
        .map_err(|error| {
          XrfError::new_unexpected_error(format!(
            "the level's collision of material {material} does not build: {error}"
          ))
        })?;

      dropped += lost.indices().len();
      meshes.push(mesh);
    }

    let children: Vec<CompoundChild<'_>> = by_material
      .keys()
      .zip(&meshes)
      .map(|(material, mesh)| CompoundChild {
        shape: mesh,
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        user_data: u32::from(*material),
      })
      .collect();
    let shape: Shape = Shape::new_compound(&children)
      .map_err(|error| XrfError::new_unexpected_error(format!("the level's collision does not build: {error}")))?;

    Ok(Self {
      shape,
      passable: geometry.get_faces().len() - kept,
      dropped,
    })
  }

  /// How many faces of the form were left out as passable.
  pub fn get_passable(&self) -> usize {
    self.passable
  }

  /// How many faces of the form the engine dropped as too small or thin to collide with reliably.
  pub fn get_dropped(&self) -> usize {
    self.dropped
  }

  /// One material's mesh of `faces`, holding only the vertices they use, renumbered through `remap` (one entry a
  /// vertex of the form, unset as `u32::MAX`, and left unset again for the next material).
  fn build_mesh(
    vertices: &[Vector3d<f32>],
    faces: &[&LevelCformFace],
    remap: &mut [u32],
  ) -> Result<(Shape, DroppedTriangles), oxijolt::ShapeError> {
    let mut used: Vec<Vec3> = Vec::new();
    let mut taken: Vec<usize> = Vec::new();
    let mut take = |index: u32| -> u32 {
      let index: usize = index as usize;

      if remap[index] == u32::MAX {
        let vertex: Vector3d<f32> = vertices[index];

        remap[index] = u32::try_from(used.len()).unwrap_or(u32::MAX);
        used.push(Vec3::new(vertex.x, vertex.y, vertex.z));
        taken.push(index);
      }

      remap[index]
    };
    // X-Ray winds its faces clockwise seen from the front; Jolt's front faces run counter-clockwise.
    let triangles: Vec<[u32; 3]> = faces
      .iter()
      .map(|face| {
        let [a, b, c] = face.vertices;

        [take(a), take(c), take(b)]
      })
      .collect();
    let built = Shape::new_mesh_with_settings(&used, &triangles, &MeshSettings::default());

    for index in taken {
      remap[index] = u32::MAX;
    }

    built
  }

  /// Whether the actor passes each material, by its id: a face holds one below `1 << 14`.
  fn find_passable(materials: &GameMtlFile) -> Vec<bool> {
    let mut passable: Vec<bool> = vec![false; usize::from(u16::MAX) + 1];

    for material in &materials.materials {
      let is_passed: bool = material.is_passable() && material.flags & GameMtlMaterial::FLAG_ACTOR_OBSTACLE == 0;

      if let Some(slot) = usize::try_from(material.id).ok().and_then(|id| passable.get_mut(id)) {
        *slot = is_passed;
      }
    }

    passable
  }
}

impl fmt::Debug for PhysicsLevel {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("PhysicsLevel")
      .field("passable", &self.passable)
      .field("dropped", &self.dropped)
      .finish_non_exhaustive()
  }
}

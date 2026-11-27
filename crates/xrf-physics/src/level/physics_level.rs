use std::fmt;

use oxijolt::{DroppedTriangles, MeshSettings, Shape, Vec3};
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_level::{LevelCformFace, LevelCformGeometry};

/// A level's collision form as the physics engine collides with it: one static triangle mesh of the faces X-Ray's
/// physics collides with, built off the game loop (it takes a second or two for a large level) and then added to a
/// [`crate::PhysicsWorld`].
pub struct PhysicsLevel {
  pub(crate) shape: Shape,
  /// How many faces were left out as passable: bushes and the like, which X-Ray's actor walks through.
  passable: usize,
  /// How many faces the engine dropped as too small or thin to collide with reliably.
  dropped: usize,
}

impl PhysicsLevel {
  /// The mesh of `geometry`, a level's `level.cform`, in the engine's space, its faces' game materials read in
  /// `materials` (the tree's `gamemtl.xr`).
  ///
  /// A face of a passable material is left out, as X-Ray's physics makes no contact with one (`Physics.cpp`,
  /// `flPassable`), unless the material is also an actor obstacle, which its actor always collides with
  /// (`PHActorCharacter.cpp`, `flActorObstacle`). A face of a material the library does not hold is kept.
  ///
  /// # Errors
  ///
  /// Returns an error when no face is left or the engine refuses the mesh.
  pub fn of_cform(geometry: &LevelCformGeometry, materials: &GameMtlFile) -> XrfResult<Self> {
    let passable: Vec<bool> = Self::find_passable(materials);
    let vertices: Vec<Vec3> = geometry
      .get_vertices()
      .iter()
      .map(|vertex| Vec3::new(vertex.x, vertex.y, vertex.z))
      .collect();
    // X-Ray winds its faces clockwise seen from the front; Jolt's front faces run counter-clockwise.
    let triangles: Vec<[u32; 3]> = geometry
      .get_faces()
      .iter()
      .filter(|face| !passable.get(usize::from(face.material)).copied().unwrap_or(false))
      .map(
        |LevelCformFace {
           vertices: [a, b, c], ..
         }| [*a, *c, *b],
      )
      .collect();
    let (shape, dropped): (Shape, DroppedTriangles) =
      Shape::new_mesh_with_settings(&vertices, &triangles, &MeshSettings::default())
        .map_err(|error| XrfError::new_unexpected_error(format!("the level's collision does not build: {error}")))?;

    Ok(Self {
      shape,
      passable: geometry.get_faces().len() - triangles.len(),
      dropped: dropped.indices().len(),
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

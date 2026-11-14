use xrf_level::{BuildPrjFile, BuildPrjName, BuildPrjShader};
use xrf_math::{Aabb, Vector2d, Vector3d};
use xrf_shaders::{ShaderCompilerFile, ShaderCompilerShader};

use crate::geometry::geometry_visual::GeometryVisual;
use crate::tests::geometry::project;
use crate::{
  CompilerShaders, GeometryBasis, GeometryFace, GeometryMesh, GeometrySplits, GeometryVisualVertex, LightColor,
};

/// A face of `material` over `vertices`, mapped across the texture.
fn face(vertices: [u32; 3], material: u16) -> GeometryFace {
  GeometryFace {
    vertices,
    texture_coordinates: [
      Vector2d::new(0.0, 0.0),
      Vector2d::new(1.0, 0.0),
      Vector2d::new(0.0, 1.0),
    ],
    material,
    game_material: 0,
    smoothing: 0,
    source: 0,
  }
}

/// A mesh of `count` unit squares in a row along x, `spacing` metres apart, each two faces of `material`.
fn row(count: u32, spacing: f32, material: u16) -> GeometryMesh {
  let mut vertices: Vec<Vector3d> = Vec::new();
  let mut faces: Vec<GeometryFace> = Vec::new();

  for square in 0..count {
    let x: f32 = square as f32 * spacing;
    let base: u32 = vertices.len() as u32;

    vertices.extend([
      Vector3d::new(x, 0.0, 0.0),
      Vector3d::new(x + 1.0, 0.0, 0.0),
      Vector3d::new(x + 1.0, 0.0, 1.0),
      Vector3d::new(x, 0.0, 1.0),
    ]);
    faces.push(face([base, base + 2, base + 1], material));
    faces.push(face([base, base + 3, base + 2], material));
  }

  GeometryMesh {
    bounds: Aabb::from_points(&vertices).expect("a mesh of corners"),
    vertices,
    faces,
  }
}

/// Compiler shaders for the geometry tests' project with materials 0 and 2 rendering and material 1 only colliding.
fn shaders_rendering_material_zero() -> CompilerShaders {
  let shader = |name: &str, flags: u32| ShaderCompilerShader {
    name: String::from(name),
    name_trailing: vec![0; ShaderCompilerShader::NAME_SIZE - name.len()],
    flags,
    vertex_translucency: 0.5,
    vertex_ambient: 0.0,
    lightmap_density: 1.0,
  };
  let library: ShaderCompilerFile = ShaderCompilerFile {
    shaders: vec![
      shader("default", ShaderCompilerShader::RENDERING),
      shader("hidden", ShaderCompilerShader::COLLISION),
    ],
  };
  let named = |name: &str| BuildPrjShader {
    name: BuildPrjName::new(name, BuildPrjName::LONG_SIZE).expect("a name"),
  };
  let mut project: BuildPrjFile = project(&[[0.0; 3]], &[]).expect("a project");

  project.compiler_shaders = vec![named("default"), named("hidden")];
  project.materials[0].compiler_shader = 0;
  project.materials[1].compiler_shader = 1;
  project.materials[2].compiler_shader = 0;

  CompilerShaders::resolve(&project, &library).expect("resolve")
}

#[test]
fn groups_rendered_faces_by_material_in_the_order_faces_first_name_it() {
  let mut mesh: GeometryMesh = row(3, 2.0, 2);

  // Material 1 does not render; material 0 is named after material 2.
  mesh.faces[1].material = 1;
  mesh.faces[4].material = 0;

  let splits: GeometrySplits = GeometrySplits::of(&mesh, &shaders_rendering_material_zero());

  assert_eq!(splits.splits, vec![vec![0, 2, 3, 5], vec![4]]);
}

#[test]
fn cuts_a_split_wider_than_the_limit_at_its_middle_and_keeps_halves_too_wide_to_join() {
  // 80 squares side by side from x = 0 to 80: 160 faces over 80 m, cut once at 40 m into two of 80 faces, which hold
  // too few faces to be cut again. Joined they would span 80 m, past four thirds of the limit and a metre past each.
  let wide: GeometrySplits = GeometrySplits::of(&row(80, 1.0, 0), &shaders_rendering_material_zero());

  assert_eq!(wide.splits.len(), 2);
  assert!(wide.splits.iter().all(|split| split.len() == 80));
  assert_eq!(wide.splits[0][0], 0, "the lower half holds the first square");

  // Splits of different materials never join.
  let mut mesh: GeometryMesh = row(2, 2.0, 0);

  mesh.faces[2].material = 2;
  mesh.faces[3].material = 2;

  let joined: GeometrySplits = GeometrySplits::of(&mesh, &shaders_rendering_material_zero());

  assert_eq!(
    joined.splits,
    vec![vec![0, 1], vec![2, 3]],
    "different materials stay apart"
  );
}

fn corner(position: [f32; 3], coordinate: [f32; 2], tangent: u8) -> GeometryVisualVertex {
  GeometryVisualVertex {
    position: Vector3d::from(position),
    normal: Vector3d::new(0.0, 1.0, 0.0),
    basis: GeometryBasis {
      tangent: [tangent, 127, 127],
      binormal: [127, 127, 255],
    },
    texture_coordinate: Vector2d::new(coordinate[0], coordinate[1]),
    lightmap_coordinate: None,
    light: LightColor::default(),
  }
}

#[test]
fn welds_a_visuals_corners_drops_closed_and_repeated_faces_and_keeps_an_opaque_fast_path() {
  let texel: [Vector2d; 1] = [Vector2d::new(1.0 / 256.0, 1.0 / 256.0)];
  let mut visual: GeometryVisual = GeometryVisual::new(0, 0, vec![String::from("ground")], String::new(), true);
  let a = corner([0.0, 0.0, 0.0], [0.0, 0.0], 255);
  let b = corner([1.0, 0.0, 0.0], [1.0, 0.0], 255);
  let c = corner([1.0, 0.0, 1.0], [1.0, 1.0], 255);
  let d = corner([0.0, 0.0, 1.0], [0.0, 1.0], 255);

  visual.add_face([a, c, b], &texel, true);
  // Shares `a` and `c`, one within a basis step and a coordinate texel of the first `a`.
  visual.add_face([corner([0.0005, 0.0, 0.0], [0.003, 0.0], 254), d, c], &texel, true);

  assert_eq!(visual.vertices.len(), 4);
  assert_eq!(visual.faces, vec![[0, 1, 2], [0, 3, 1]]);

  // A repeat in another order is left out; a face closing on itself takes no vertex.
  visual.add_face([b, a, c], &texel, true);
  visual.add_face([a, corner([3.0, 0.0, 0.0], [5.0, 5.0], 0), a], &texel, true);

  assert_eq!((visual.vertices.len(), visual.faces.len()), (4, 2));
  assert_eq!((visual.fast_vertices.len(), visual.fast_faces.len()), (4, 2));

  // A corner a basis step and a half off, or a texel off, is a vertex of its own; a face with alpha has no fast path.
  visual.add_face([corner([0.0, 0.0, 0.0], [0.0, 0.0], 252), b, c], &texel, false);
  visual.add_face([corner([0.0, 0.0, 0.0], [0.004, 0.0], 255), d, c], &texel, false);

  assert_eq!(visual.vertices.len(), 6);
  assert_eq!(visual.fast_faces.len(), 2);
}

#[test]
fn shifts_each_joined_groups_coordinates_by_the_whole_tiles_at_its_middle() {
  let texel: [Vector2d; 1] = [Vector2d::new(1.0 / 256.0, 1.0 / 256.0)];
  let mut visual: GeometryVisual = GeometryVisual::new(0, 0, vec![String::from("ground")], String::new(), true);

  // One face mapped from 3 to 4 tiles along u, another apart from it from -2 to -1.
  visual.add_face(
    [
      corner([0.0, 0.0, 0.0], [3.0, 0.0], 255),
      corner([1.0, 0.0, 1.0], [4.0, 1.0], 255),
      corner([1.0, 0.0, 0.0], [4.0, 0.0], 255),
    ],
    &texel,
    true,
  );
  visual.add_face(
    [
      corner([5.0, 0.0, 0.0], [-2.0, 0.0], 255),
      corner([6.0, 0.0, 1.0], [-1.0, 1.0], 255),
      corner([6.0, 0.0, 0.0], [-1.0, 0.0], 255),
    ],
    &texel,
    true,
  );
  visual.shift_texture_coordinates();

  // The first group's middle is 3.5 along u, and it moves 3 tiles. The second's span reaches up from the smallest
  // positive float, not from its own -1, so its middle is -1 and it moves 1 tile.
  assert_eq!(visual.vertices[0].texture_coordinate, Vector2d::new(0.0, 0.0));
  assert_eq!(visual.vertices[3].texture_coordinate, Vector2d::new(-1.0, 0.0));
  assert_eq!(visual.vertices[4].texture_coordinate, Vector2d::new(0.0, 1.0));
}

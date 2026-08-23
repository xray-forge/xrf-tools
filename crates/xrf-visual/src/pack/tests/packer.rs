//! Holds what a packed visual promises: the right triangles, in the right space, with a reason for
//! every piece that produced none.

use xrf_db::{OgfFile, OgfGeometry};

use crate::data::visual_section::VisualDrawRange;
use crate::data::visual_submesh::{VisualGeometry, VisualSkipCause, VisualSubmesh};
use crate::pack::tests::fixtures::{
  MODEL_TYPE_GEOMDEF_PM, MODEL_TYPE_GEOMDEF_ST, PROGRESSIVE_FINE_OFFSET, PROGRESSIVE_FINE_TRIANGLES,
  PROGRESSIVE_INDICES, bones, description, embedded_motions, geometry, geometry_of_unknown_format, kinematics,
  progressive_child, progressive_child_with_windows, skeleton, static_triangle_child, swi, textured, vector, vertex,
  visual, window,
};
use crate::pack::tests::reader::{read_f32_section, read_u16_section};
use crate::pack::visual_package::VisualPackage;
use crate::pack::visual_packer::VisualPacker;

fn only_submesh(package: &VisualPackage) -> &VisualSubmesh {
  assert_eq!(package.description.submeshes.len(), 1, "expect exactly one submesh");

  &package.description.submeshes[0]
}

fn only_geometry(package: &VisualPackage) -> &VisualGeometry {
  only_submesh(package)
    .geometry()
    .expect("expect the only submesh to have packed")
}

fn skipped_reason(child: OgfFile) -> String {
  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![child]));

  String::from(
    only_submesh(&package)
      .skipped_reason()
      .expect("expect the child to have been skipped"),
  )
}

#[test]
fn draws_the_whole_index_buffer_of_a_static_submesh() {
  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![static_triangle_child()]));
  let geometry: &VisualGeometry = only_geometry(&package);

  assert_eq!(geometry.vertex_count, 3);
  assert_eq!(geometry.index_count, 3);
  assert_eq!(
    geometry.detail_levels,
    vec![VisualDrawRange { start: 0, count: 3 }],
    "expect a static submesh to offer its whole buffer as its one level"
  );
}

#[test]
fn draws_only_detail_level_zero_of_a_progressive_submesh() {
  // The whole point of the crate. Level zero is the tail of the buffer here, mirroring the measured
  // case in `gamedata`, so drawing everything would render four triangles instead of two and stack the
  // coarse shell over the fine mesh.
  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![progressive_child()]));
  let geometry: &VisualGeometry = only_geometry(&package);

  assert_eq!(geometry.index_count, PROGRESSIVE_INDICES.len() as u32);
  assert_eq!(
    geometry.get_default_level(),
    VisualDrawRange {
      start: PROGRESSIVE_FINE_OFFSET,
      count: u32::from(PROGRESSIVE_FINE_TRIANGLES) * 3,
    }
  );
  assert!(
    geometry.get_default_level().count < geometry.index_count,
    "expect the drawn range to be a slice of the buffer, not all of it"
  );
}

#[test]
fn ships_the_whole_index_buffer_and_every_usable_level_of_a_progressive_submesh() {
  // Nothing is discarded at the boundary: the detail level picker changes a draw range rather than
  // re-reading the file, so the coarse levels have to survive packing.
  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![progressive_child()]));
  let geometry: &VisualGeometry = only_geometry(&package);

  assert_eq!(
    geometry.detail_levels,
    vec![
      VisualDrawRange {
        start: PROGRESSIVE_FINE_OFFSET,
        count: u32::from(PROGRESSIVE_FINE_TRIANGLES) * 3,
      },
      VisualDrawRange { start: 0, count: 12 },
    ]
  );
  assert_eq!(
    read_u16_section(&package.buffer, geometry.indices).len(),
    PROGRESSIVE_INDICES.len()
  );
}

#[test]
fn drops_an_unusable_coarse_level_without_failing_the_submesh() {
  // A bad coarse level costs the option, not the mesh: the finest level is what the model is, and the
  // viewer can still draw it. Reaching past the index buffer is the shape this takes in the wild.
  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![progressive_child_with_windows(vec![
    window(PROGRESSIVE_FINE_OFFSET, PROGRESSIVE_FINE_TRIANGLES, 6),
    window(0, 64, 4),
  ])]));
  let geometry: &VisualGeometry = only_geometry(&package);

  assert_eq!(
    geometry.detail_levels,
    vec![VisualDrawRange {
      start: PROGRESSIVE_FINE_OFFSET,
      count: u32::from(PROGRESSIVE_FINE_TRIANGLES) * 3,
    }]
  );
}

#[test]
fn winds_each_triangle_in_place_so_detail_offsets_stay_valid() {
  // Read back from the buffer rather than from the fixture: the drawn range must still name the two
  // fine triangles after winding, which is only true if winding never moved a triangle.
  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![progressive_child()]));
  let geometry: &VisualGeometry = only_geometry(&package);
  let indices: Vec<u16> = read_u16_section(&package.buffer, geometry.indices);

  let start: usize = geometry.get_default_level().start as usize;
  let drawn: &[u16] = &indices[start..start + geometry.get_default_level().count as usize];

  assert_eq!(
    drawn,
    &[0, 4, 1, 1, 5, 2],
    "expect the fine triangles, wound the other way"
  );
  assert_eq!(
    &indices[0..6],
    &[0, 2, 1, 0, 3, 2],
    "expect the coarse level to keep its place at the head"
  );
}

#[test]
fn mirrors_z_on_positions_and_normals() {
  let child: OgfFile = OgfFile {
    geometry: Some(geometry(
      vec![vertex(vector(1.0, 2.0, 3.0), vector(0.0, 0.0, 1.0), 0.0, 0.0)],
      vec![0, 0, 0],
    )),
    ..visual(MODEL_TYPE_GEOMDEF_ST)
  };

  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![child]));
  let geometry: &VisualGeometry = only_geometry(&package);

  assert_eq!(
    read_f32_section(&package.buffer, geometry.positions),
    vec![1.0, 2.0, -3.0]
  );
  assert_eq!(
    read_f32_section(&package.buffer, geometry.normals),
    vec![0.0, 0.0, -1.0]
  );
}

#[test]
fn keeps_the_texture_coordinates_of_every_vertex() {
  let child: OgfFile = OgfFile {
    geometry: Some(geometry(
      vec![
        vertex(vector(0.0, 0.0, 0.0), vector(0.0, 0.0, 1.0), 0.25, 0.75),
        vertex(vector(1.0, 0.0, 0.0), vector(0.0, 0.0, 1.0), 1.0, 0.0),
      ],
      vec![0, 1, 0],
    )),
    ..visual(MODEL_TYPE_GEOMDEF_ST)
  };

  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![child]));
  let geometry: &VisualGeometry = only_geometry(&package);

  assert_eq!(
    read_f32_section(&package.buffer, geometry.uvs),
    vec![0.25, 0.75, 1.0, 0.0]
  );
}

#[test]
fn measures_bounds_over_the_drawn_range_only() {
  // A progressive buffer also holds vertices only the coarse levels reach. Vertex 3 here is one of
  // them, and it sits furthest out, so measuring the whole buffer would report a box a third larger
  // than the mesh actually on screen.
  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![progressive_child()]));
  let geometry: &VisualGeometry = only_geometry(&package);

  assert_eq!(geometry.vertex_count, 6, "expect every vertex to still ship");
  assert_eq!(
    geometry.bounds.bounding_box.max,
    vector(5.0, 10.0, 0.0),
    "expect the box to reach vertex 5, which the drawn range uses"
  );
  assert_eq!(
    geometry.bounds.bounding_box.min,
    vector(0.0, 0.0, -15.0),
    "expect the near face to come from vertex 5 rather than from unused vertex 3"
  );
}

#[test]
fn measures_bounds_from_converted_geometry() {
  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![static_triangle_child()]));
  let geometry: &VisualGeometry = only_geometry(&package);

  assert_eq!(geometry.bounds.bounding_box.min, vector(0.0, 0.0, 0.0));
  assert_eq!(geometry.bounds.bounding_box.max, vector(1.0, 1.0, 0.0));
  assert_eq!(geometry.bounds.bounding_sphere.center, vector(0.5, 0.5, 0.0));
}

#[test]
fn keeps_declared_and_computed_bounds_apart() {
  // Unreconciled on purpose: a file whose header disagrees with its vertices should show the
  // disagreement rather than have one silently stand in for the other.
  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![static_triangle_child()]));

  assert_eq!(
    package.description.declared_bounds.bounding_box.min,
    vector(-1.0, -2.0, -6.0)
  );
  assert_eq!(
    package
      .description
      .computed_bounds
      .as_ref()
      .expect("expect measured bounds")
      .bounding_box
      .max,
    vector(1.0, 1.0, 0.0)
  );
}

#[test]
fn merges_computed_bounds_across_packed_submeshes_only() {
  let far_child: OgfFile = OgfFile {
    geometry: Some(geometry(
      vec![
        vertex(vector(10.0, 0.0, 0.0), vector(0.0, 0.0, 1.0), 0.0, 0.0),
        vertex(vector(10.0, 4.0, 0.0), vector(0.0, 0.0, 1.0), 0.0, 0.0),
        vertex(vector(10.0, 0.0, 4.0), vector(0.0, 0.0, 1.0), 0.0, 0.0),
      ],
      vec![0, 1, 2],
    )),
    ..visual(MODEL_TYPE_GEOMDEF_ST)
  };

  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![
    static_triangle_child(),
    visual(MODEL_TYPE_GEOMDEF_ST),
    far_child,
  ]));

  let bounds = package.description.computed_bounds.expect("expect measured bounds");

  assert_eq!(bounds.bounding_box.min, vector(0.0, 0.0, -4.0));
  assert_eq!(bounds.bounding_box.max, vector(10.0, 4.0, 0.0));
}

#[test]
fn has_no_computed_bounds_when_nothing_packed() {
  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![visual(MODEL_TYPE_GEOMDEF_ST)]));

  assert!(package.description.computed_bounds.is_none());
  assert_eq!(package.description.buffer_length, 0);
  assert!(package.buffer.is_empty());
}

#[test]
fn keeps_packing_after_a_skipped_child() {
  // One broken piece must not hide the rest of a model, which is the reason a failure is a value here.
  let package: VisualPackage =
    VisualPacker::pack(&skeleton(vec![visual(MODEL_TYPE_GEOMDEF_ST), static_triangle_child()]));

  assert_eq!(package.description.submeshes.len(), 2);
  assert!(package.description.submeshes[0].skipped_reason().is_some());
  assert!(package.description.submeshes[1].geometry().is_some());
  assert_eq!(package.description.submeshes[1].index, 1);
}

#[test]
fn reports_the_buffer_length_the_description_covers() {
  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![progressive_child(), static_triangle_child()]));

  assert_eq!(package.description.buffer_length as usize, package.buffer.len());
}

#[test]
fn packs_a_single_level_visual_as_its_own_submesh() {
  let package: VisualPackage = VisualPacker::pack(&static_triangle_child());

  assert_eq!(only_geometry(&package).index_count, 3);
}

#[test]
fn carries_texture_and_shader_references_of_each_child() {
  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![textured(
    "wpn\\wpn_ak74",
    "models\\weapons",
    static_triangle_child(),
  )]));

  let submesh: &VisualSubmesh = only_submesh(&package);

  assert_eq!(submesh.texture_name.as_deref(), Some("wpn\\wpn_ak74"));
  assert_eq!(submesh.shader_name.as_deref(), Some("models\\weapons"));
  assert_eq!(submesh.model_type_label, "MT_SKELETON_GEOMDEF_ST");
}

#[test]
fn labels_a_model_type_the_engine_has_no_name_for() {
  let package: VisualPackage = VisualPacker::pack(&visual(37));

  assert_eq!(package.description.model_type_label, "MT_UNKNOWN(37)");
}

#[test]
fn reports_bones_motion_refs_and_embedded_motions() {
  let file: OgfFile = OgfFile {
    bones: Some(bones(&[("wpn_body", ""), ("magazin", "wpn_body")])),
    kinematics: Some(kinematics(&["dynamics\\weapons\\wpn_ak74\\wpn_ak74_hud_animation"])),
    motions: Some(embedded_motions(&["idle", "reload"])),
    description: Some(description("x:\\rawdata\\objects\\wpn_ak74.object")),
    ..skeleton(vec![static_triangle_child()])
  };

  let package: VisualPackage = VisualPacker::pack(&file);

  assert_eq!(
    package.description.bones,
    vec![
      crate::data::visual_description::VisualBone {
        name: String::from("wpn_body"),
        parent: String::new(),
      },
      crate::data::visual_description::VisualBone {
        name: String::from("magazin"),
        parent: String::from("wpn_body"),
      },
    ]
  );
  assert_eq!(
    package.description.motion_refs,
    vec![String::from("dynamics\\weapons\\wpn_ak74\\wpn_ak74_hud_animation")]
  );
  assert_eq!(
    package.description.embedded_motions,
    vec![String::from("idle"), String::from("reload")]
  );
  assert_eq!(
    package.description.source_file.as_deref(),
    Some("x:\\rawdata\\objects\\wpn_ak74.object")
  );
}

#[test]
fn treats_an_empty_source_file_as_absent() {
  let file: OgfFile = OgfFile {
    description: Some(description("")),
    ..skeleton(vec![static_triangle_child()])
  };

  assert!(VisualPacker::pack(&file).description.source_file.is_none());
}

#[test]
fn grades_a_coverage_gap_apart_from_a_broken_file() {
  // A sweep fails on the second and only notes the first, so the two must be distinguishable without
  // reading the message.
  let unsupported: OgfFile = OgfFile {
    geometry: Some(geometry_of_unknown_format(0xdead_beef, vec![0, 1, 2])),
    ..visual(MODEL_TYPE_GEOMDEF_ST)
  };
  let malformed: OgfFile = OgfFile {
    swi_data: Some(swi(vec![window(10, 2, 6)])),
    ..progressive_child()
  };

  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![unsupported, malformed]));

  assert_eq!(
    package.description.submeshes[0].skipped().map(|(cause, _)| cause),
    Some(VisualSkipCause::Unsupported)
  );
  assert_eq!(
    package.description.submeshes[1].skipped().map(|(cause, _)| cause),
    Some(VisualSkipCause::Malformed)
  );
}

#[test]
fn skips_a_child_with_no_geometry_chunk() {
  assert_eq!(
    skipped_reason(visual(MODEL_TYPE_GEOMDEF_ST)),
    "Carries no geometry chunk"
  );
}

#[test]
fn skips_a_child_whose_vertex_format_has_no_known_layout() {
  let child: OgfFile = OgfFile {
    geometry: Some(geometry_of_unknown_format(0xdead_beef, vec![0, 1, 2])),
    ..visual(MODEL_TYPE_GEOMDEF_ST)
  };

  assert_eq!(skipped_reason(child), "Vertex format 0xdeadbeef has no known layout");
}

#[test]
fn skips_a_child_with_no_index_chunk() {
  let child: OgfFile = OgfFile {
    geometry: Some(OgfGeometry {
      indices: None,
      ..geometry(
        vec![vertex(vector(0.0, 0.0, 0.0), vector(0.0, 0.0, 1.0), 0.0, 0.0)],
        vec![],
      )
    }),
    ..visual(MODEL_TYPE_GEOMDEF_ST)
  };

  assert_eq!(skipped_reason(child), "Carries no index chunk");
}

#[test]
fn skips_a_child_with_an_empty_index_chunk() {
  let child: OgfFile = OgfFile {
    geometry: Some(geometry(
      vec![vertex(vector(0.0, 0.0, 0.0), vector(0.0, 0.0, 1.0), 0.0, 0.0)],
      vec![],
    )),
    ..visual(MODEL_TYPE_GEOMDEF_ST)
  };

  assert_eq!(skipped_reason(child), "Index chunk is empty");
}

#[test]
fn skips_a_child_whose_index_count_is_not_whole_triangles() {
  let child: OgfFile = OgfFile {
    geometry: Some(geometry(
      vec![vertex(vector(0.0, 0.0, 0.0), vector(0.0, 0.0, 1.0), 0.0, 0.0)],
      vec![0, 0, 0, 0],
    )),
    ..visual(MODEL_TYPE_GEOMDEF_ST)
  };

  assert_eq!(
    skipped_reason(child),
    "Index count 4 is not a whole number of triangles"
  );
}

#[test]
fn skips_a_progressive_child_with_no_detail_table() {
  // Falling back to the whole buffer would draw every detail level at once, which looks like a fatter
  // model rather than an error. Refusing is louder, and the reference trees say this does not occur.
  let child: OgfFile = OgfFile {
    swi_data: None,
    ..progressive_child()
  };

  assert_eq!(
    skipped_reason(child),
    "Progressive geometry carries no detail table, so its full detail range is unknown"
  );
}

#[test]
fn skips_a_progressive_child_whose_detail_level_leaves_the_index_buffer() {
  let child: OgfFile = OgfFile {
    swi_data: Some(swi(vec![window(10, 2, 6)])),
    ..progressive_child()
  };

  assert_eq!(
    skipped_reason(child),
    "Detail level 0 draws 6 indices from offset 10, past the 12 the index chunk holds"
  );
}

#[test]
fn skips_a_child_whose_drawn_range_addresses_a_missing_vertex() {
  let child: OgfFile = OgfFile {
    geometry: Some(geometry(
      vec![
        vertex(vector(0.0, 0.0, 0.0), vector(0.0, 0.0, 1.0), 0.0, 0.0),
        vertex(vector(1.0, 0.0, 0.0), vector(0.0, 0.0, 1.0), 0.0, 0.0),
        vertex(vector(0.0, 1.0, 0.0), vector(0.0, 0.0, 1.0), 0.0, 0.0),
      ],
      vec![0, 1, 9],
    )),
    ..visual(MODEL_TYPE_GEOMDEF_ST)
  };

  assert_eq!(
    skipped_reason(child),
    "Drawn range references vertex 9, past the 3 the vertex chunk holds"
  );
}

#[test]
fn ignores_an_out_of_range_index_outside_the_drawn_range() {
  // Coarse levels are shipped untouched and never dereferenced, so a bad index in one must not cost the
  // model its full detail mesh.
  let child: OgfFile = OgfFile {
    geometry: Some(geometry(
      (0..6)
        .map(|index| vertex(vector(index as f32, 0.0, 0.0), vector(0.0, 0.0, 1.0), 0.0, 0.0))
        .collect(),
      vec![99, 98, 97, 0, 2, 3, 0, 1, 4, 1, 2, 5],
    )),
    swi_data: Some(swi(vec![window(
      PROGRESSIVE_FINE_OFFSET,
      PROGRESSIVE_FINE_TRIANGLES,
      6,
    )])),
    ..visual(MODEL_TYPE_GEOMDEF_PM)
  };

  let package: VisualPackage = VisualPacker::pack(&skeleton(vec![child]));

  assert!(only_submesh(&package).geometry().is_some());
}

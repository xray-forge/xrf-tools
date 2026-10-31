use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use image::RgbaImage;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_math::Vector3d;
use xrf_spawn::{
  AlifeLevelChanger, AlifeObject, AlifeObjectAbstract, AlifeObjectInherited, AlifeObjectSpaceRestrictor, ClsId,
  GraphLevel, Patrol, PatrolLink, PatrolPoint, Shape, SpawnLevelGraph,
};
use xrf_test_utils::GeneratedResources;
use xrf_visual::VisualTransform;

use crate::objects::{
  LEVEL_OBJECT_KINDS, LevelObject, LevelObjectId, LevelObjectKind, LevelObjectPlacement, LevelObjectShape,
  LevelObjectSources, LevelObjectsDescription, LevelPlacementDetails, LevelPlacementFile,
};
use crate::parts::{LevelMapArea, LevelSpawn};
use crate::tests::fixtures;
use crate::viewer::map::annotations::{
  AnnotationCanvas, AnnotationMarker, AnnotationOutline, AnnotationVolume, LevelAnnotations, MapAnnotationsWriteRequest,
};

fn new_restrictor(shape: Vec<Shape>, restrictor_type: u8) -> AlifeObjectSpaceRestrictor {
  AlifeObjectSpaceRestrictor {
    base: fixtures::new_abstract(AlifeObjectAbstract::INVALID_STORY_ID),
    shape,
    restrictor_type,
  }
}

fn new_object(name: &str, position: Vector3d, heading: f32, inherited: AlifeObjectInherited) -> AlifeObject {
  AlifeObject {
    position,
    direction: Vector3d::new(0.0, heading, 0.0),
    ..fixtures::new_object(name, "space_restrictor", ClsId::SpcRsS, inherited)
  }
}

fn new_level(id: u8, name: &str, offset: Vector3d) -> GraphLevel {
  GraphLevel {
    id,
    name: String::from(name),
    section: String::from(name),
    guid: Uuid::nil(),
    offset,
  }
}

/// A box four metres square and two high about the origin.
fn new_box() -> Shape {
  Shape::Box((
    Vector3d::new(4.0, 0.0, 0.0),
    Vector3d::new(0.0, 2.0, 0.0),
    Vector3d::new(0.0, 0.0, 4.0),
    Vector3d::new(0.0, 0.0, 0.0),
  ))
}

fn new_spawn(objects: Vec<AlifeObject>) -> LevelSpawn {
  LevelSpawn {
    objects,
    arrivals: Vec::new(),
    releases: HashMap::new(),
  }
}

/// Zaton, and Pripyat 500 metres north of it, a graph point and a patrol of two points.
fn new_graph() -> SpawnLevelGraph {
  SpawnLevelGraph {
    levels: vec![
      new_level(1, "zaton", Vector3d::new(0.0, 0.0, 0.0)),
      new_level(2, "pripyat", Vector3d::new(0.0, 0.0, 500.0)),
    ],
    points: vec![(321, Vector3d::new(5.0, 0.0, 6.0))],
    patrols: vec![Patrol {
      name: String::from("zat_walk"),
      points: ["wp00", "wp01"]
        .iter()
        .enumerate()
        .map(|(index, name)| PatrolPoint {
          name: String::from(*name),
          position: Vector3d::new(index as f32 * 10.0, 0.0, 0.0),
          flags: 0,
          level_vertex_id: 0,
          game_vertex_id: 0,
        })
        .collect(),
      links: vec![PatrolLink {
        index: 0,
        links: vec![(1, 1.0), (7, 1.0)],
      }],
    }],
  }
}

fn zaton_area() -> LevelMapArea {
  LevelMapArea {
    min_x: -600.0,
    min_z: -615.0,
    max_x: 600.0,
    max_z: 585.0,
  }
}

fn every_kind() -> Vec<LevelObjectKind> {
  LEVEL_OBJECT_KINDS.iter().map(|entry| entry.kind).collect()
}

/// Graph point 321 from above at `point`, tagged with its vertex.
fn vertex_marker(point: [f32; 2]) -> AnnotationMarker {
  AnnotationMarker {
    kind: LevelObjectKind::GraphPoints,
    name: String::from("vertex 321"),
    point,
    tag: Some(String::from("321")),
  }
}

/// Zaton's map at 1200 pixels a side drawing `kinds`, none of their objects named beyond what is always named.
fn zaton_canvas(kinds: Vec<LevelObjectKind>) -> AnnotationCanvas {
  AnnotationCanvas {
    area: zaton_area(),
    width: 1200,
    height: 1200,
    kinds,
    is_labelled: false,
  }
}

fn zaton_request(root: &Path) -> MapAnnotationsWriteRequest {
  MapAnnotationsWriteRequest {
    path: root.join("map.png"),
    area: zaton_area(),
    kinds: every_kind(),
    is_labelled: false,
  }
}

#[test]
fn outlines_a_box_from_above_at_its_place() {
  let spawn: LevelSpawn = new_spawn(vec![new_object(
    "zat_space",
    Vector3d::new(10.0, 0.0, 20.0),
    0.0,
    AlifeObjectInherited::CseAlifeSpaceRestrictor(Box::new(new_restrictor(vec![new_box()], 3))),
  )]);
  let annotations: LevelAnnotations = LevelAnnotations::of(
    &LevelObjectsDescription::describe(&LevelObjectSources {
      graph: Some(&new_graph()),
      ..LevelObjectSources::of_spawn(&spawn)
    }),
    &new_graph(),
    "zaton",
  );
  let [volume] = annotations.volumes.as_slice() else {
    panic!("one volume expected, not {}", annotations.volumes.len());
  };
  let AnnotationOutline::Polygon(points) = &volume.outlines[0] else {
    panic!("a box is a polygon from above");
  };
  let mut corners: Vec<[i32; 2]> = points
    .iter()
    .map(|[x, z]| [x.round() as i32, z.round() as i32])
    .collect();

  corners.sort_unstable();

  // Its eight corners from above are four, the box's footprint about its place.
  assert_eq!(corners, vec![[8, 18], [8, 22], [12, 18], [12, 22]]);
  assert_eq!(volume.kind, LevelObjectKind::Restrictors);
  assert!(!volume.is_in);
}

#[test]
fn turns_a_sphere_with_its_object_and_keeps_its_radius() {
  let sphere: Shape = Shape::Sphere((Vector3d::new(5.0, 0.0, 0.0), 3.0));
  let spawn: LevelSpawn = new_spawn(vec![new_object(
    "zat_in",
    Vector3d::new(100.0, 0.0, 0.0),
    std::f32::consts::FRAC_PI_2,
    AlifeObjectInherited::CseAlifeSpaceRestrictor(Box::new(new_restrictor(vec![sphere], 4))),
  )]);
  let volume: &AnnotationVolume = &LevelAnnotations::of(
    &LevelObjectsDescription::describe(&LevelObjectSources {
      graph: Some(&new_graph()),
      ..LevelObjectSources::of_spawn(&spawn)
    }),
    &new_graph(),
    "zaton",
  )
  .volumes[0];
  let AnnotationOutline::Circle { center, radius } = volume.outlines[0] else {
    panic!("a sphere is a circle from above");
  };

  // Five metres off the object's place, turned a quarter away from `+x`.
  assert!(
    (center[0] - 100.0).abs() < 1e-3 && (center[1].abs() - 5.0).abs() < 1e-3,
    "{center:?}"
  );
  assert_eq!(radius, 3.0);
  assert!(volume.is_in);
}

#[test]
fn points_a_level_changer_toward_its_destination_across_the_world() {
  let changer: AlifeLevelChanger = AlifeLevelChanger {
    base: new_restrictor(vec![new_box()], 3),
    dest_game_vertex_id: 0,
    dest_level_vertex_id: 0,
    dest_position: Vector3d::new(0.0, 0.0, 0.0),
    dest_direction: Vector3d::new(0.0, 0.0, 0.0),
    dest_level_name: String::from("pripyat"),
    dest_graph_point: String::new(),
    silent_mode: 0,
    enabled: 1,
    hint: String::new(),
  };
  let spawn: LevelSpawn = new_spawn(vec![new_object(
    "zat_to_pripyat",
    Vector3d::new(0.0, 0.0, 0.0),
    0.0,
    AlifeObjectInherited::SeLevelChanger(Box::new(changer)),
  )]);
  let annotations: LevelAnnotations = LevelAnnotations::of(
    &LevelObjectsDescription::describe(&LevelObjectSources {
      graph: Some(&new_graph()),
      ..LevelObjectSources::of_spawn(&spawn)
    }),
    &new_graph(),
    "zaton",
  );

  assert_eq!(
    annotations.volumes[0].destination,
    Some((String::from("pripyat"), Some([0.0, 1.0])))
  );
  // A link to a point the patrol does not hold is left out, and its points are drawn with it rather than as dots.
  assert_eq!(annotations.patrols[0].links, vec![(0, 1)]);
  assert_eq!(annotations.markers, vec![vertex_marker([5.0, 6.0])]);
}

#[test]
fn outlines_a_placement_covering_a_space_and_dots_what_stands_at_a_point() {
  let at = |x: f32, z: f32| VisualTransform::of_spawn(&Vector3d::new(x, 0.0, z), &Vector3d::new(0.0, 0.0, 0.0));
  let mut environment: LevelObject = LevelObject::new(
    LevelObjectId::Placement {
      file: LevelPlacementFile::SoundEnvironments,
      index: 0,
    },
    LevelObjectKind::SoundEnvironments,
    String::from("cave"),
    at(30.0, 40.0),
  );
  let mut particles: LevelObject = LevelObject::new(
    LevelObjectId::Placement {
      file: LevelPlacementFile::Particles,
      index: 0,
    },
    LevelObjectKind::StaticParticles,
    String::from("ambient\\fire"),
    at(-5.0, 7.0),
  );

  environment.placement = Some(LevelObjectPlacement {
    shapes: vec![LevelObjectShape::Sphere {
      center: Vector3d::new(0.0, 0.0, 0.0),
      radius: 6.0,
    }],
    details: LevelPlacementDetails::SoundEnvironment {
      outer: String::from("outdoor"),
      inner: String::from("cave"),
    },
  });
  particles.placement = Some(LevelObjectPlacement {
    shapes: Vec::new(),
    details: LevelPlacementDetails::Particles {
      effect: String::from("ambient\\fire"),
    },
  });

  let annotations: LevelAnnotations = LevelAnnotations::of(
    &LevelObjectsDescription {
      visuals: Vec::new(),
      objects: vec![environment, particles],
    },
    &new_graph(),
    "zaton",
  );
  let [volume] = annotations.volumes.as_slice() else {
    panic!("one volume expected, not {}", annotations.volumes.len());
  };

  assert_eq!(volume.kind, LevelObjectKind::SoundEnvironments);
  assert_eq!(
    volume.outlines,
    vec![AnnotationOutline::Circle {
      center: [30.0, 40.0],
      radius: 6.0
    }]
  );
  assert_eq!(
    annotations.markers,
    vec![AnnotationMarker {
      kind: LevelObjectKind::StaticParticles,
      name: String::from("ambient\\fire"),
      point: [-5.0, 7.0],
      tag: None,
    }]
  );
}

#[test]
fn refuses_an_area_spanning_no_ground() {
  let area = |min_x: f32, max_x: f32| LevelMapArea {
    min_x,
    min_z: -10.0,
    max_x,
    max_z: 10.0,
  };

  assert!(area(-10.0, 10.0).require_ground().is_ok());
  assert!(area(10.0, 10.0).require_ground().is_err());
  assert!(area(10.0, -10.0).require_ground().is_err());
  assert!(area(f32::NAN, 10.0).require_ground().is_err());
}

#[test]
fn lays_the_area_over_the_map_north_up() {
  let canvas: AnnotationCanvas = zaton_canvas(every_kind());

  assert_eq!(canvas.to_pixel([-600.0, 585.0]), (0.0, 0.0));
  assert_eq!(canvas.to_pixel([600.0, -615.0]), (1200.0, 1200.0));
  assert_eq!(canvas.to_pixel([0.0, -15.0]), (600.0, 600.0));
}

#[test]
fn draws_a_group_a_kind_asked_and_names_only_what_it_names() {
  let annotations: LevelAnnotations = LevelAnnotations {
    volumes: vec![
      AnnotationVolume {
        kind: LevelObjectKind::SmartTerrains,
        name: String::from("zat_stalker_base_smart"),
        outlines: vec![AnnotationOutline::Circle {
          center: [0.0, 0.0],
          radius: 20.0,
        }],
        is_in: false,
        destination: None,
      },
      AnnotationVolume {
        kind: LevelObjectKind::Restrictors,
        name: String::from("zat_<restrictor>"),
        outlines: vec![AnnotationOutline::Circle {
          center: [50.0, 0.0],
          radius: 5.0,
        }],
        is_in: true,
        destination: None,
      },
    ],
    patrols: Vec::new(),
    markers: vec![AnnotationMarker {
      kind: LevelObjectKind::Stalkers,
      name: String::from("zat_guard"),
      point: [10.0, 0.0],
      tag: None,
    }],
  };
  let quiet: String = zaton_canvas(vec![
    LevelObjectKind::SmartTerrains,
    LevelObjectKind::Restrictors,
    LevelObjectKind::Stalkers,
  ])
  .draw(&annotations);
  let named: String = AnnotationCanvas {
    is_labelled: true,
    ..zaton_canvas(vec![LevelObjectKind::Restrictors, LevelObjectKind::Stalkers])
  }
  .draw(&annotations);
  let group = |svg: &str, id: &str| svg.find(&format!(r#"<g id="{id}""#));

  assert!(quiet.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 1200""#));
  assert!(group(&quiet, "smart-terrains").is_some() && group(&quiet, "restrictors").is_some());
  assert!(group(&quiet, "patrols").is_none());
  // The dots lowest, the outlines over them and the smart terrains, always named, on top.
  assert!(group(&quiet, "stalkers") < group(&quiet, "restrictors"));
  assert!(group(&quiet, "restrictors") < group(&quiet, "smart-terrains"));
  assert!(quiet.contains(r##"fill="#e040fb""##));
  assert!(quiet.contains(">zat_stalker_base_smart</text>"));
  assert!(!quiet.contains("zat_&lt;restrictor&gt;") && !quiet.contains("zat_guard"));
  // A restrictor keeping the actor in is a lighter blue.
  assert!(quiet.contains("#90caf9"));
  assert!(named.contains(">zat_&lt;restrictor&gt;</text>") && named.contains(">zat_guard</text>"));
  assert!(!named.contains("smart-terrains"));
}

#[test]
fn rasterises_the_svg_at_any_size_clear_where_it_draws_nothing() -> TauriResult<()> {
  let svg: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><rect x="0" y="0" width="50" height="100" fill="#ff0000"/></svg>"##;
  let image = AnnotationCanvas::rasterise(svg, (40, 20))?;

  assert_eq!(image.dimensions(), (40, 20));
  assert_eq!(image.get_pixel(5, 10).0, [255, 0, 0, 255]);
  assert_eq!(image.get_pixel(35, 10).0[3], 0);

  Ok(())
}

// A map's annotations are drawn at the size the map was captured at, whatever the request's area, and the SVG answered
// is the one written beside it.
#[test]
fn writes_the_annotations_over_a_map_at_the_size_it_was_captured() -> TauriResult<()> {
  let root: PathBuf = GeneratedResources::path("level_annotations/write");
  let map: PathBuf = root.join("map.png");
  let target: PathBuf = root.join("map.svg");

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(&root).expect("directory created");
  RgbaImage::new(40, 20).save(&map).expect("map written");

  let canvas: AnnotationCanvas = AnnotationCanvas::over_map(&MapAnnotationsWriteRequest {
    path: map,
    ..zaton_request(&root)
  })?;
  let svg: String = canvas.write(
    &LevelAnnotations::of(
      &LevelObjectsDescription::describe(&LevelObjectSources {
        graph: Some(&new_graph()),
        ..LevelObjectSources::of_spawn(&new_spawn(Vec::new()))
      }),
      &new_graph(),
      "zaton",
    ),
    &target,
  )?;

  assert_eq!((canvas.width, canvas.height), (40, 20));
  assert!(svg.contains(r#"viewBox="0 0 40 20""#));
  assert_eq!(fs::read_to_string(&target).expect("annotations written"), svg);

  Ok(())
}

#[test]
fn refuses_a_map_it_cannot_read() {
  let root: PathBuf = GeneratedResources::path("level_annotations/no_map");

  assert!(AnnotationCanvas::over_map(&zaton_request(&root)).is_err());
}

#[test]
fn names_each_graph_point_by_its_game_vertex() {
  let annotations: LevelAnnotations = LevelAnnotations {
    volumes: Vec::new(),
    patrols: Vec::new(),
    markers: vec![vertex_marker([0.0, -15.0])],
  };
  let svg: String = zaton_canvas(vec![LevelObjectKind::GraphPoints]).draw(&annotations);

  assert!(svg.contains(r#"<g id="graph-points""#));
  assert!(svg.contains(r#"<circle cx="600.0" cy="600.0""#));
  assert!(svg.contains(">321</text>"));
}

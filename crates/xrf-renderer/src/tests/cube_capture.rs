use glam::{Mat4, Vec3, Vec4};
use image::RgbaImage;
use xrf_error::XrfResult;

use crate::camera::camera_frame::CameraFrame;
use crate::camera::engine_space::EngineSpace;
use crate::capture::capture_plan::CapturePlan;
use crate::capture::capture_shot::CaptureShot;
use crate::capture::cube_assembly::CubeAssembly;
use crate::capture::cube_face::CubeFace;
use crate::capture::map_tile_plan::MapTilePlan;
use crate::contract::render_capture::RenderCapture;
use crate::contract::render_cube_capture::RenderCubeCapture;
use crate::contract::render_map_capture::RenderMapCapture;
use crate::contract::render_scale::RenderScale;
use crate::contract::render_view_options::RenderViewOptions;

/// A camera standing at engine `(10, 2, -30)`, its fog ending past its far plane.
fn camera() -> CameraFrame {
  CameraFrame {
    position: EngineSpace::mirror(Vec3::new(10.0, 2.0, -30.0)),
    near: 0.2,
    far: 500.0,
    ..CameraFrame::default()
  }
}

fn cube(size: u32, supersampling: u32) -> RenderCubeCapture {
  RenderCubeCapture {
    size,
    supersampling,
    settle_frames: 8,
  }
}

fn faces(plan: &CapturePlan) -> Vec<CubeFace> {
  plan
    .shots
    .iter()
    .map(|shot| match shot {
      CaptureShot::Face(face) => *face,
      CaptureShot::Tile(_) => panic!("a cubemap planned a map's tile"),
    })
    .collect()
}

/// The engine-space direction a face sees through the point `(u, v)` of its picture, each from -1 to 1, `u` right
/// and `v` down.
fn to_direction(face: &CubeFace, u: f32, v: f32) -> Vec3 {
  let inverse: Mat4 = face.get_view().get_view_projection().inverse();
  let point: Vec4 = inverse * Vec4::new(u, -v, 0.5, 1.0);

  EngineSpace::mirror(point.truncate() / point.w - face.position).normalize()
}

#[test]
fn faces_look_down_the_engine_axes_as_a_cube_texture_lays_them() -> XrfResult {
  let plan: CapturePlan = CapturePlan::new_cube(cube(256, 1), &camera(), f32::INFINITY, 8192)?;
  // Direct3D's cube faces: the direction through `(u, v)` of each, `+x -x +y -y +z -z`, which the engine's
  // `D3DXCreateCubeTextureFromFile` reads its faces by.
  let expected: [fn(f32, f32) -> Vec3; 6] = [
    |u, v| Vec3::new(1.0, -v, -u),
    |u, v| Vec3::new(-1.0, -v, u),
    |u, v| Vec3::new(u, 1.0, v),
    |u, v| Vec3::new(u, -1.0, -v),
    |u, v| Vec3::new(u, -v, 1.0),
    |u, v| Vec3::new(-u, -v, -1.0),
  ];

  for (face, expected) in faces(&plan).iter().zip(expected) {
    for (u, v) in [(0.0, 0.0), (0.9, 0.0), (0.0, 0.9), (-0.5, 0.7)] {
      let seen: Vec3 = to_direction(face, u, v);

      assert!(
        seen.abs_diff_eq(expected(u, v).normalize(), 1e-4),
        "face {} at ({u}, {v}) sees {seen}",
        face.index
      );
    }
  }

  Ok(())
}

#[test]
fn plans_six_faces_at_the_camera_after_metering_all_six_small() -> XrfResult {
  let plan: CapturePlan = CapturePlan::new_cube(cube(1024, 2), &camera(), 300.0, 8192)?;
  let shots: Vec<CubeFace> = faces(&plan);

  assert_eq!(shots.len(), CubeFace::COUNT);
  assert!(shots.iter().enumerate().all(|(index, face)| face.index == index
    && face.position == camera().position
    && face.get_drawn_size() == (2048, 2048)));
  // The fog's end cuts the far plane short.
  assert_eq!(shots[0].far, 300.0);
  assert_eq!(plan.meters.len(), CubeFace::COUNT);
  assert_eq!(plan.meter_frames % CubeFace::COUNT as u32, 0);
  assert_eq!(
    plan.get_meter(7).map(|shot| shot.get_drawn_size()),
    Some((CapturePlan::CUBE_METER, CapturePlan::CUBE_METER))
  );

  Ok(())
}

#[test]
fn refuses_cubemaps_it_cannot_draw() {
  assert!(CapturePlan::new_cube(cube(0, 1), &camera(), f32::INFINITY, 8192).is_err());
  assert!(CapturePlan::new_cube(cube(4096, 1), &camera(), f32::INFINITY, 8192).is_err());
  assert!(CapturePlan::new_cube(cube(512, 0), &camera(), f32::INFINITY, 8192).is_err());
  assert!(CapturePlan::new_cube(cube(512, 5), &camera(), f32::INFINITY, 8192).is_err());
  // Past the GPU's largest texture once drawn over.
  assert!(CapturePlan::new_cube(cube(2048, 4), &camera(), f32::INFINITY, 4096).is_err());
}

#[test]
fn lays_the_faces_out_as_a_cross_and_takes_them_back_in_order() -> XrfResult {
  let plan: CapturePlan = CapturePlan::new_cube(cube(8, 2), &camera(), f32::INFINITY, 8192)?;
  let mut assembly: CubeAssembly = CubeAssembly::new(8);

  // Each face drawn one flat colour with no alpha of its own, so its place in the cross shows which it was.
  for face in faces(&plan) {
    let (side, _): (u32, u32) = face.get_drawn_size();
    let shade: u8 = 40 * (face.index as u8 + 1);

    assembly.place(
      &face,
      RenderCapture {
        width: side,
        height: side,
        pixels: [shade, shade, shade, 0].repeat((side * side) as usize),
        frame: 0,
      },
    )?;
  }

  let cross: RgbaImage = assembly.finish();

  assert_eq!(cross.dimensions(), (32, 24));
  // +x right of the forward +z, -x left of it, +y above, -y below, -z at the far right.
  assert_eq!(cross.get_pixel(20, 12).0, [40, 40, 40, 255]);
  assert_eq!(cross.get_pixel(4, 12).0, [80, 80, 80, 255]);
  assert_eq!(cross.get_pixel(12, 4).0, [120, 120, 120, 255]);
  assert_eq!(cross.get_pixel(12, 20).0, [160, 160, 160, 255]);
  assert_eq!(cross.get_pixel(12, 12).0, [200, 200, 200, 255]);
  assert_eq!(cross.get_pixel(28, 12).0, [240, 240, 240, 255]);
  // No face in the corners.
  assert_eq!(cross.get_pixel(2, 2).0[3], 0);

  let split: Vec<RgbaImage> = CubeAssembly::split(&cross)?;

  assert_eq!(split.len(), CubeFace::COUNT);
  assert!(
    split
      .iter()
      .enumerate()
      .all(|(index, face)| face.dimensions() == (8, 8) && face.get_pixel(3, 3).0[0] == 40 * (index as u8 + 1))
  );
  assert!(CubeAssembly::split(&RgbaImage::new(30, 24)).is_err());

  Ok(())
}

#[test]
fn refuses_a_face_drawn_at_another_size() -> XrfResult {
  let plan: CapturePlan = CapturePlan::new_cube(cube(8, 1), &camera(), f32::INFINITY, 8192)?;

  assert!(
    CubeAssembly::new(8)
      .place(
        &faces(&plan)[0],
        RenderCapture {
          width: 16,
          height: 16,
          pixels: vec![0; 16 * 16 * 4],
          frame: 0,
        },
      )
      .is_err()
  );

  Ok(())
}

#[test]
fn draws_a_face_with_the_whole_look_but_its_lens_flare_and_a_tile_with_its_coverage() -> XrfResult {
  let face: CaptureShot = CapturePlan::new_cube(cube(256, 1), &camera(), f32::INFINITY, 8192)?.shots[0];
  let tile: CaptureShot = CaptureShot::Tile(
    MapTilePlan::new(
      RenderMapCapture {
        min_x: -10.0,
        min_z: -10.0,
        max_x: 10.0,
        max_z: 10.0,
        bottom: 0.0,
        top: 10.0,
        width: 64,
        height: 64,
        supersampling: 1,
        settle_frames: 1,
        is_grassed: false,
      },
      8192,
    )?
    .tiles[0],
  );
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.output.upscaling.scale = RenderScale::Quality;

  let look: RenderViewOptions = face.apply_look(&options, false);

  assert!(look.show.is_sky_visible && look.show.is_fogged && !look.show.is_lens_flared);
  assert_eq!(look.features.grass.is_enabled, options.features.grass.is_enabled);
  assert_eq!(look.output.upscaling.scale, RenderScale::Native);
  assert!(
    face.apply_look(&options, true).features.exposure.adaptation
      < tile.apply_look(&options, true).features.exposure.adaptation
  );
  assert!(!face.is_covered() && tile.is_covered());

  Ok(())
}

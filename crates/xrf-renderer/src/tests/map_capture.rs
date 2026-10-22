use glam::{Vec3, Vec4};
use xrf_error::XrfResult;

use crate::camera::camera_view::CameraView;
use crate::camera::engine_space::EngineSpace;
use crate::capture::map_assembly::MapAssembly;
use crate::capture::map_capture_look::MapCaptureLook;
use crate::capture::map_tile::MapTile;
use crate::capture::map_tile_plan::MapTilePlan;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_capture::RenderCapture;
use crate::contract::render_map_capture::{RENDER_MAP_TILE_MARGIN, RenderMapCapture};
use crate::contract::render_rect::RenderRect;
use crate::contract::render_scale::RenderScale;
use crate::contract::render_view_options::RenderViewOptions;
use crate::scene::level::view_lens::ViewLens;

/// Zaton's `bound_rect`, 1200 metres a side, drawn between its lowest and highest ground.
fn zaton(width: u32, height: u32, supersampling: u32) -> RenderMapCapture {
  RenderMapCapture {
    min_x: -600.0,
    min_z: -615.0,
    max_x: 600.0,
    max_z: 585.0,
    bottom: -20.0,
    top: 80.0,
    width,
    height,
    supersampling,
    settle_frames: 8,
    is_grassed: false,
  }
}

/// Where an engine-space point lands in a tile's drawing, in its pixels from the top left.
fn to_tile_pixel(tile: &MapTile, point: Vec3) -> (f32, f32) {
  let view: CameraView = tile.get_view();
  let clip: Vec4 = view.get_view_projection() * EngineSpace::mirror(point).extend(1.0);
  let (width, height): (u32, u32) = tile.get_drawn_size();

  (
    (clip.x / clip.w * 0.5 + 0.5) * width as f32,
    (0.5 - clip.y / clip.w * 0.5) * height as f32,
  )
}

#[test]
fn cuts_a_map_into_tiles_row_by_row_from_its_top_left() -> XrfResult {
  let plan: MapTilePlan = MapTilePlan::new(zaton(4096, 2732, 1), 8192)?;
  let placed: Vec<RenderRect> = plan.tiles.iter().map(|tile| tile.placed).collect();

  assert_eq!(
    placed,
    vec![
      RenderRect {
        x: 0,
        y: 0,
        width: 2048,
        height: 2048,
      },
      RenderRect {
        x: 2048,
        y: 0,
        width: 2048,
        height: 2048,
      },
      RenderRect {
        x: 0,
        y: 2048,
        width: 2048,
        height: 684,
      },
      RenderRect {
        x: 2048,
        y: 2048,
        width: 2048,
        height: 684,
      },
    ]
  );

  Ok(())
}

#[test]
fn draws_a_tile_past_its_pixels_by_the_margin_times_the_supersampling() -> XrfResult {
  let plan: MapTilePlan = MapTilePlan::new(zaton(1024, 1024, 2), 8192)?;

  assert_eq!(plan.tiles.len(), 1);
  assert_eq!(
    plan.tiles[0].get_drawn_size(),
    (
      (1024 + 2 * RENDER_MAP_TILE_MARGIN) * 2,
      (1024 + 2 * RENDER_MAP_TILE_MARGIN) * 2
    )
  );

  Ok(())
}

#[test]
fn draws_the_map_north_up_with_its_corners_on_the_area_corners() -> XrfResult {
  let plan: MapTilePlan = MapTilePlan::new(zaton(1200, 1200, 1), 8192)?;
  let tile: &MapTile = &plan.tiles[0];
  let margin: f32 = RENDER_MAP_TILE_MARGIN as f32;
  // One map pixel a metre, so the area's corners are the placed pixels' corners, past the margin.
  let (left, top): (f32, f32) = to_tile_pixel(tile, Vec3::new(-600.0, 0.0, 585.0));
  let (right, bottom): (f32, f32) = to_tile_pixel(tile, Vec3::new(600.0, 0.0, -615.0));

  assert!(
    (left - margin).abs() < 1e-2 && (top - margin).abs() < 1e-2,
    "{left}, {top}"
  );
  assert!(
    (right - margin - 1200.0).abs() < 1e-2 && (bottom - margin - 1200.0).abs() < 1e-2,
    "{right}, {bottom}"
  );

  Ok(())
}

#[test]
fn meets_neighbouring_tiles_on_the_same_ground() -> XrfResult {
  let plan: MapTilePlan = MapTilePlan::new(zaton(4096, 4096, 1), 8192)?;
  let (first, second): (&MapTile, &MapTile) = (&plan.tiles[0], &plan.tiles[1]);
  let margin: f32 = RENDER_MAP_TILE_MARGIN as f32;
  // The ground under the first tile's last column and the second's first, which must be one map pixel apart.
  let seam: Vec3 = Vec3::new(-600.0 + 2048.0 * 1200.0 / 4096.0, 0.0, 0.0);

  assert!((to_tile_pixel(first, seam).0 - margin - 2048.0).abs() < 1e-2);
  assert!((to_tile_pixel(second, seam).0 - margin).abs() < 1e-2);

  Ok(())
}

#[test]
fn cuts_the_level_between_the_heights_asked() -> XrfResult {
  let plan: MapTilePlan = MapTilePlan::new(zaton(1024, 1024, 1), 8192)?;
  let view: CameraView = plan.tiles[0].get_view();
  let depth = |height: f32| -> f32 {
    let clip: Vec4 = view.get_view_projection() * EngineSpace::mirror(Vec3::new(0.0, height, 0.0)).extend(1.0);

    clip.z / clip.w
  };

  assert!((depth(80.0) - 1.0).abs() < 1e-4);
  assert!(depth(-20.0).abs() < 1e-4);

  Ok(())
}

#[test]
fn refuses_maps_it_cannot_draw() {
  assert!(
    MapTilePlan::new(
      RenderMapCapture {
        max_x: -600.0,
        ..zaton(1024, 1024, 1)
      },
      8192
    )
    .is_err()
  );
  assert!(
    MapTilePlan::new(
      RenderMapCapture {
        top: -30.0,
        ..zaton(1024, 1024, 1)
      },
      8192
    )
    .is_err()
  );
  assert!(MapTilePlan::new(zaton(0, 1024, 1), 8192).is_err());
  assert!(MapTilePlan::new(zaton(1024, 1024, 0), 8192).is_err());
  // A full tile drawn four times over is past what an 8192 device draws.
  assert!(MapTilePlan::new(zaton(4096, 4096, 4), 8192).is_err());
}

#[test]
fn meters_the_whole_area_at_most_its_metering_size() -> XrfResult {
  let plan: MapTilePlan = MapTilePlan::new(zaton(4096, 2048, 2), 8192)?;
  let meter: MapTile = plan.get_meter();

  assert_eq!(meter.get_drawn_size(), (MapTilePlan::METER, MapTilePlan::METER / 2));
  assert_eq!(
    (meter.min_x, meter.max_x, meter.min_z, meter.max_z),
    (-600.0, 600.0, -615.0, 585.0)
  );

  Ok(())
}

#[test]
fn assembles_tiles_reduced_and_cut_to_their_places() -> XrfResult {
  let plan: MapTilePlan = MapTilePlan::new(zaton(2100, 30, 2), 16384)?;
  let mut assembly: MapAssembly = MapAssembly::new(2100, 30);

  // Each tile drawn one flat colour, so its place in the map shows which it was.
  for (index, tile) in plan.tiles.iter().enumerate() {
    let (width, height): (u32, u32) = tile.get_drawn_size();
    let shade: u8 = 100 + index as u8 * 100;

    assembly.place(
      tile,
      RenderCapture {
        width,
        height,
        pixels: [shade, shade, shade, 255].repeat((width * height) as usize),
        frame: 0,
      },
    )?;
  }

  let map = assembly.finish();

  assert_eq!(map.dimensions(), (2100, 30));
  assert_eq!(map.get_pixel(2047, 29).0, [100, 100, 100, 255]);
  assert_eq!(map.get_pixel(2048, 0).0, [200, 200, 200, 255]);

  Ok(())
}

#[test]
fn refuses_a_tile_drawn_at_another_size() -> XrfResult {
  let plan: MapTilePlan = MapTilePlan::new(zaton(64, 64, 1), 8192)?;

  assert!(
    MapAssembly::new(64, 64)
      .place(
        &plan.tiles[0],
        RenderCapture {
          width: 64,
          height: 64,
          pixels: vec![0; 64 * 64 * 4],
          frame: 0,
        },
      )
      .is_err()
  );

  Ok(())
}

#[test]
fn draws_a_tile_as_the_engine_draws_its_map_through_the_lens_of_its_view() -> XrfResult {
  let plan: MapTilePlan = MapTilePlan::new(zaton(1024, 1024, 1), 8192)?;
  let tile: &MapTile = &plan.tiles[0];
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.features.antialiasing = RenderAntialiasing::Dlss;
  options.output.upscaling.scale = RenderScale::Quality;
  options.output.output_scale = 0.5;

  let look: RenderViewOptions = MapCaptureLook::apply(&options, tile);

  // The map's own look: no fog, particles or bloom, drawn at the tile's own pixels.
  assert!(!look.show.is_fogged && !look.show.is_particled && !look.features.bloom.is_enabled);
  assert!(!look.features.grass.is_enabled);
  assert_eq!(look.output.upscaling.scale, RenderScale::Native);
  assert_eq!(look.output.output_scale, 1.0);
  // The view's own look stands otherwise.
  assert_eq!(look.mode, options.mode);
  assert_eq!(look.features.ambient_occlusion, options.features.ambient_occlusion);

  // Drawn as the render thread draws it, through the tile's lens: one cascade holding the tile's whole box.
  let drawn: RenderViewOptions = ViewLens::fit(&look, &tile.get_view()).into_owned();
  let box_diagonal: f32 = Vec3::new(tile.max_x - tile.min_x, tile.top - tile.bottom, tile.max_z - tile.min_z).length();

  assert_eq!(drawn.features.shadows.cascades.len(), 1);
  assert!((drawn.features.shadows.cascades[0] - box_diagonal).abs() < 1e-2);
  assert_eq!(drawn.features.antialiasing, RenderAntialiasing::Taa);

  Ok(())
}

#[test]
fn plants_grass_over_a_tile_to_its_corners_only_when_asked() -> XrfResult {
  let plan: MapTilePlan = MapTilePlan::new(
    RenderMapCapture {
      is_grassed: true,
      ..zaton(4096, 4096, 1)
    },
    8192,
  )?;
  let tile: &MapTile = &plan.tiles[0];
  let look: RenderViewOptions = ViewLens::fit(
    &MapCaptureLook::apply(&RenderViewOptions::default(), tile),
    &tile.get_view(),
  )
  .into_owned();
  let corner: f32 = (tile.max_x - tile.min_x).hypot(tile.max_z - tile.min_z) * 0.5;

  assert!(look.features.grass.is_enabled);
  assert!(look.features.grass.radius >= corner && look.features.grass.radius < corner + 1.0);
  // The metering view plants none, nor a map not asking for it.
  assert!(!plan.get_meter().is_grassed);
  assert!(
    !MapCaptureLook::apply(
      &RenderViewOptions::default(),
      &MapTilePlan::new(zaton(4096, 4096, 1), 8192)?.tiles[0]
    )
    .features
    .grass
    .is_enabled
  );

  Ok(())
}

use xrf_error::XrfResult;
use xrf_math::{Aabb, Sphere, Vector3d};
use xrf_ogf::{OgfHeaderChunk, OgfModelType, OgfSlideWindow, OgfSwiContainerChunk, OgfSwiDataChunk};

use crate::geom::level_geom_file::LevelGeomFile;
use crate::geom::window::level_geom_slide_window::LevelGeomSlideWindow;
use crate::geom::window::level_geom_slide_window_item::LevelGeomSlideWindowItem;
use crate::level::visuals::level_visual::LevelVisual;

/// A visual of `model_type` with neither windows nor a table, drawing nothing yet.
fn visual(model_type: OgfModelType) -> LevelVisual {
  LevelVisual {
    header: OgfHeaderChunk {
      version: 4,
      model_type: model_type.to_raw(),
      shader_id: 0,
      bounding_box: Aabb {
        min: Vector3d::ZERO,
        max: Vector3d::ZERO,
      },
      bounding_sphere: Sphere {
        center: Vector3d::ZERO,
        radius: 0.0,
      },
    },
    texture: None,
    geometry: None,
    fastpath: None,
    children: Vec::new(),
    tree: None,
    lod: None,
    swi: None,
    swi_container: None,
  }
}

/// Level geometry holding one window table: full detail at index 0, a coarser level after it.
fn geometry() -> LevelGeomFile {
  LevelGeomFile {
    vertex_buffers: Vec::new(),
    index_buffers: Vec::new(),
    slide_windows: vec![LevelGeomSlideWindowItem {
      reserved: [0; 4],
      windows: vec![
        LevelGeomSlideWindow {
          offset: 0,
          triangles: 120,
          vertices: 90,
        },
        LevelGeomSlideWindow {
          offset: 360,
          triangles: 60,
          vertices: 45,
        },
      ],
    }],
  }
}

#[test]
fn takes_a_static_progressive_visuals_own_first_window() -> XrfResult {
  let mut progressive: LevelVisual = visual(OgfModelType::Progressive);

  progressive.swi = Some(Ok(OgfSwiDataChunk {
    reserved: [0; 4],
    windows: vec![
      OgfSlideWindow {
        offset: 6,
        num_tris: 40,
        num_verts: 30,
      },
      OgfSlideWindow {
        offset: 126,
        num_tris: 20,
        num_verts: 15,
      },
    ],
  }));

  assert_eq!(
    progressive.get_full_detail_window(&geometry())?,
    Some(LevelGeomSlideWindow {
      offset: 6,
      triangles: 40,
      vertices: 30,
    })
  );

  Ok(())
}

#[test]
fn takes_a_progressive_trees_first_window_from_the_table_it_names() -> XrfResult {
  let mut tree: LevelVisual = visual(OgfModelType::TreePm);

  tree.swi_container = Some(Ok(OgfSwiContainerChunk { ext_swib_index: 0 }));

  assert_eq!(
    tree.get_full_detail_window(&geometry())?,
    Some(LevelGeomSlideWindow {
      offset: 0,
      triangles: 120,
      vertices: 90,
    })
  );

  // A table the level does not hold is refused, not read past.
  tree.swi_container = Some(Ok(OgfSwiContainerChunk { ext_swib_index: 3 }));

  assert!(tree.get_full_detail_window(&geometry()).is_err());

  Ok(())
}

#[test]
fn refuses_a_progressive_visual_without_a_window_and_takes_none_for_a_plain_one() -> XrfResult {
  // Its whole run lays every level of detail end to end, so drawing it whole draws them all at once.
  assert!(
    visual(OgfModelType::Progressive)
      .get_full_detail_window(&geometry())
      .is_err()
  );
  assert_eq!(visual(OgfModelType::Normal).get_full_detail_window(&geometry())?, None);
  assert_eq!(visual(OgfModelType::TreeSt).get_full_detail_window(&geometry())?, None);

  Ok(())
}

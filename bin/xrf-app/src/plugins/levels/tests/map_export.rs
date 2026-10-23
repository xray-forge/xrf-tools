use std::fs;
use std::path::PathBuf;

use image::{ImageFormat, Rgba, RgbaImage};
use xrf_dds::DdsFile;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::map_export::{export_map, flatten_onto, get_export_size};
use crate::plugins::levels::viewer::map_export_format::MapExportFormat;
use crate::plugins::levels::viewer::map_export_request::{MapExportAnnotations, MapExportRequest};
use xrf_texture::{TextureEncodingFormat, TextureEncodingQuality};

/// A 30 by 20 map, its left half covered in red and its right clear, and beside it annotations drawing a white square
/// over its right half, in a folder of its own.
fn write_map(name: &str) -> TauriResult<(PathBuf, PathBuf)> {
  let folder: PathBuf = build_absolute_generated_test_resource_path(&format!("map_export/{name}"));
  let map: RgbaImage = RgbaImage::from_fn(30, 20, |x, _| {
    if x < 15 {
      Rgba([255, 0, 0, 255])
    } else {
      Rgba([0, 0, 0, 0])
    }
  });

  let _ = fs::remove_dir_all(&folder);
  fs::create_dir_all(&folder).map_err(|error| error.to_string())?;

  let path: PathBuf = folder.join("map_zaton.png");

  map
    .save_with_format(&path, ImageFormat::Png)
    .map_err(|error| error.to_string())?;
  fs::write(
    path.with_extension("svg"),
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 30 20"><rect x="15" y="0" width="15" height="20" fill="#ffffff"/></svg>"##,
  )
  .map_err(|error| error.to_string())?;

  Ok((folder, path))
}

fn new_request(path: PathBuf, target: PathBuf, format: MapExportFormat) -> MapExportRequest {
  MapExportRequest {
    path,
    target,
    format,
    long_side: 30,
    backdrop: None,
    annotations: None,
  }
}

#[test]
fn exports_at_most_the_capture_keeping_its_shape() {
  assert_eq!(get_export_size((4096, 2732), 2048, false), (2048, 1366));
  assert_eq!(get_export_size((4096, 2732), 8192, false), (4096, 2732));
  assert_eq!(get_export_size((1000, 3000), 1500, false), (500, 1500));
}

#[test]
fn rounds_a_dds_to_whole_blocks() {
  // 2732 / 2 is 1366, between the blocks 1364 and 1368, nearer the second.
  assert_eq!(get_export_size((4096, 2732), 2048, true), (2048, 1368));
  assert_eq!(get_export_size((5, 3), 5, true), (4, 4));
}

#[test]
fn lays_a_map_over_its_backdrop_opaque() {
  let mut image: RgbaImage = RgbaImage::from_pixel(2, 1, Rgba([200, 100, 0, 255]));

  image.put_pixel(1, 0, Rgba([200, 100, 0, 0]));
  flatten_onto(&mut image, [10, 20, 30]);

  assert_eq!(image.get_pixel(0, 0).0, [200, 100, 0, 255]);
  assert_eq!(image.get_pixel(1, 0).0, [10, 20, 30, 255]);
}

#[test]
fn writes_a_dds_of_whole_blocks_still_covering_the_map() -> TauriResult<()> {
  let (folder, path) = write_map("dds")?;
  let target: PathBuf = folder.join("map_zaton.dds");
  let format: MapExportFormat = MapExportFormat::Dds {
    format: TextureEncodingFormat::Bc3,
    quality: TextureEncodingQuality::Fast,
    is_mipmapped: true,
  };

  // 30 pixels across are seven blocks and a half, resampled to eight rather than padded.
  assert_eq!(export_map(&new_request(path, target.clone(), format))?, (32, 20));

  let metadata = DdsFile::read_from_path(&target)
    .map_err(|error| error.to_string())?
    .metadata();

  assert_eq!((metadata.width, metadata.height), (32, 20));
  assert!(metadata.mipmap_levels > 1);

  Ok(())
}

#[test]
fn lays_the_annotations_over_the_map_or_alone() -> TauriResult<()> {
  let (folder, path) = write_map("annotated")?;
  let over: PathBuf = folder.join("over.png");
  let alone: PathBuf = folder.join("alone.png");

  export_map(&MapExportRequest {
    annotations: Some(MapExportAnnotations::Over),
    ..new_request(path.clone(), over.clone(), MapExportFormat::Png)
  })?;
  export_map(&MapExportRequest {
    annotations: Some(MapExportAnnotations::Alone),
    backdrop: Some([0, 0, 255]),
    ..new_request(path, alone.clone(), MapExportFormat::Png)
  })?;

  let over: RgbaImage = image::open(&over).map_err(|error| error.to_string())?.to_rgba8();
  let alone: RgbaImage = image::open(&alone).map_err(|error| error.to_string())?.to_rgba8();

  assert_eq!(over.get_pixel(5, 10).0, [255, 0, 0, 255]);
  assert_eq!(over.get_pixel(25, 10).0, [255, 255, 255, 255]);
  // Alone, the map's red is gone from under the backdrop.
  assert_eq!(alone.get_pixel(5, 10).0, [0, 0, 255, 255]);
  assert_eq!(alone.get_pixel(25, 10).0, [255, 255, 255, 255]);

  Ok(())
}

#[test]
fn writes_an_svg_only_of_the_annotations_alone() -> TauriResult<()> {
  let (folder, path) = write_map("svg")?;
  let target: PathBuf = folder.join("annotations.svg");

  assert!(export_map(&new_request(path.clone(), target.clone(), MapExportFormat::Svg)).is_err());
  assert!(!target.exists(), "a refused export writes nothing");
  assert_eq!(
    export_map(&MapExportRequest {
      annotations: Some(MapExportAnnotations::Alone),
      ..new_request(path.clone(), target.clone(), MapExportFormat::Svg)
    })?,
    (30, 20)
  );
  assert_eq!(
    fs::read_to_string(&target).map_err(|error| error.to_string())?,
    fs::read_to_string(path.with_extension("svg")).map_err(|error| error.to_string())?
  );

  Ok(())
}

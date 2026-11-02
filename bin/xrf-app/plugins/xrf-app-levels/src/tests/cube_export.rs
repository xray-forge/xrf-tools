use std::fs;
use std::path::PathBuf;

use image::{ImageFormat, Rgba, RgbaImage};
use xrf_app_core::TauriResult;
use xrf_dds::DdsFile;
use xrf_test_utils::GeneratedResources;
use xrf_texture::{TextureEncodingFormat, TextureEncodingQuality};

use crate::viewer::ViewerDdsExport;
use crate::viewer::cube::{CubeExportFormat, CubeExportRequest, CubeExporter};

/// A cross of 16 pixel faces in a folder of its own, each face one flat shade by its place in the engine's order.
fn write_cross(name: &str) -> TauriResult<(PathBuf, PathBuf)> {
  let folder: PathBuf = GeneratedResources::path(&format!("cube_export/{name}"));
  let places: [(u32, u32); 6] = [(2, 1), (0, 1), (1, 0), (1, 2), (1, 1), (3, 1)];
  let mut cross: RgbaImage = RgbaImage::new(64, 48);

  for (face, (x, y)) in places.iter().enumerate() {
    for (px, py) in (0..16).flat_map(|px| (0..16).map(move |py| (px, py))) {
      cross.put_pixel(x * 16 + px, y * 16 + py, Rgba([40 * (face as u8 + 1), 0, 0, 255]));
    }
  }

  let _ = fs::remove_dir_all(&folder);
  fs::create_dir_all(&folder).map_err(|error| error.to_string())?;

  let path: PathBuf = folder.join("cube_zaton.png");

  cross
    .save_with_format(&path, ImageFormat::Png)
    .map_err(|error| error.to_string())?;

  Ok((folder, path))
}

#[test]
fn names_six_faces_as_the_engine_and_by_face() {
  let request = |format: CubeExportFormat| CubeExportRequest {
    path: PathBuf::from("shots/cube_zaton.png"),
    target: PathBuf::from("out"),
    format,
    size: 16,
  };

  assert_eq!(
    CubeExporter::list_paths(&request(CubeExportFormat::Tga)),
    (1..=6)
      .map(|face| PathBuf::from(format!("out/{face}.tga")))
      .collect::<Vec<_>>()
  );
  assert_eq!(
    CubeExporter::list_paths(&request(CubeExportFormat::Png))[3],
    PathBuf::from("out/cube_zaton_ny.png")
  );
}

#[test]
fn writes_the_faces_as_one_cube_dds_at_the_size_asked() -> TauriResult<()> {
  let (folder, path) = write_cross("dds")?;
  let target: PathBuf = folder.join("cube.dds");
  let written: Vec<PathBuf> = CubeExporter::export(&CubeExportRequest {
    path,
    target: target.clone(),
    format: CubeExportFormat::Dds(ViewerDdsExport {
      format: TextureEncodingFormat::Bc3,
      quality: TextureEncodingQuality::Fast,
      is_mipmapped: true,
    }),
    size: 8,
  })?;
  let metadata = DdsFile::read_from_path(&target)
    .map_err(|error| error.to_string())?
    .metadata();

  assert_eq!(written, vec![target]);
  assert!(metadata.is_cubemap);
  assert_eq!((metadata.width, metadata.height, metadata.mipmap_levels), (8, 8, 4));

  Ok(())
}

#[test]
fn writes_six_faces_as_tgas_and_pngs() -> TauriResult<()> {
  let (folder, path) = write_cross("faces")?;

  for format in [CubeExportFormat::Tga, CubeExportFormat::Png] {
    let written: Vec<PathBuf> = CubeExporter::export(&CubeExportRequest {
      path: path.clone(),
      target: folder.clone(),
      format,
      size: 16,
    })?;

    assert_eq!(written.len(), 6);

    for (face, file) in written.iter().enumerate() {
      let image: RgbaImage = image::open(file).map_err(|error| error.to_string())?.to_rgba8();

      assert_eq!(image.dimensions(), (16, 16));
      assert_eq!(
        image.get_pixel(8, 8).0,
        [40 * (face as u8 + 1), 0, 0, 255],
        "{}",
        file.display()
      );
    }
  }

  Ok(())
}

#[test]
fn refuses_a_picture_that_is_no_cross() -> TauriResult<()> {
  let (folder, _) = write_cross("refused")?;
  let path: PathBuf = folder.join("square.png");

  RgbaImage::new(32, 32)
    .save_with_format(&path, ImageFormat::Png)
    .map_err(|error| error.to_string())?;

  assert!(
    CubeExporter::export(&CubeExportRequest {
      path,
      target: folder,
      format: CubeExportFormat::Png,
      size: 8,
    })
    .is_err()
  );

  Ok(())
}

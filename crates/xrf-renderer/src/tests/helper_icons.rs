use std::path::PathBuf;

use image::{Rgba, RgbaImage};
use resvg::{tiny_skia, usvg};
use serde_json::{Value, json};

use crate::contract::render_helper_icon::{RENDER_HELPER_ICONS, RenderHelperIcon};
use crate::host::render_bundle::RENDER_BUNDLE_SOURCE;
use crate::scene::level::helpers::gpu::helper_icon_atlas::HelperIconAtlas;

/// Where the icons' manifest and the pictures exported for it stand, which only the generator reads.
fn get_sources() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("helper-icons")
}

/// The manifest the icons make: each by the name it serialises to, the Material icon it is drawn from and the file its
/// picture is exported to, in the atlas's order, as the picture exporter reads it.
fn to_manifest() -> String {
  let icons: Vec<Value> = RENDER_HELPER_ICONS
    .iter()
    .map(|entry| {
      json!({
        "icon": entry.icon.get_name(),
        "material": entry.material,
        "picture": entry.icon.get_picture_file(),
      })
    })
    .collect();
  let manifest: Value = json!({
    "source": "@mui/icons-material (Material Icons, Apache License 2.0), as the application's UI draws them",
    "icons": icons,
  });

  format!(
    "{}\n",
    serde_json::to_string_pretty(&manifest).expect("a manifest serialises")
  )
}

#[test]
fn names_each_icon_as_it_serialises() {
  for icon in RENDER_HELPER_ICONS.iter().map(|entry| entry.icon) {
    assert_eq!(serde_json::to_value(icon).ok(), Some(json!(icon.get_name())));
    assert_eq!(RENDER_HELPER_ICONS[icon.get_index() as usize].icon, icon);
  }

  assert_eq!(RenderHelperIcon::ActorStart.get_picture_file(), "actor-start.svg");
  assert_eq!(RenderHelperIcon::Light.get_picture_file(), "light.svg");
}

/// The manifest is what the icons write: a stale one is rewritten and fails this once, its pictures and atlas then
/// made again (`cargo make generate-helper-icons`).
#[test]
fn the_manifest_names_every_icon_in_the_atlas_order() {
  let path: PathBuf = get_sources().join("manifest.json");
  let manifest: String = to_manifest();

  if std::fs::read_to_string(&path).ok().as_deref() != Some(manifest.as_str()) {
    std::fs::create_dir_all(get_sources()).expect("the icons' folder is made");
    std::fs::write(&path, manifest).expect("the manifest is written");
    panic!("rewritten: run `cargo make generate-helper-icons` and commit the manifest, its pictures and the atlas");
  }
}

#[test]
fn the_bundled_atlas_holds_a_cell_for_every_icon() {
  let atlas: RgbaImage = image::open(PathBuf::from(RENDER_BUNDLE_SOURCE).join(HelperIconAtlas::PATH))
    .expect("the atlas is bundled")
    .to_rgba8();

  assert_eq!(atlas.dimensions(), HelperIconAtlas::get_size(RENDER_HELPER_ICONS.len()));
  // Each cell draws its picture, in white within its margin.
  for icon in RENDER_HELPER_ICONS.iter().map(|entry| entry.icon) {
    let (column, row) = (
      icon.get_index() % HelperIconAtlas::COLUMNS,
      icon.get_index() / HelperIconAtlas::COLUMNS,
    );
    let drawn: usize = (0..HelperIconAtlas::CELL * HelperIconAtlas::CELL)
      .map(|texel| {
        atlas.get_pixel(
          column * HelperIconAtlas::CELL + texel % HelperIconAtlas::CELL,
          row * HelperIconAtlas::CELL + texel / HelperIconAtlas::CELL,
        )
      })
      .filter(|pixel| pixel.0 == [255, 255, 255, 255])
      .count();

    assert!(drawn > 100, "{icon:?} draws {drawn} white texels");
  }
}

/// Rasterises the exported pictures into the bundled atlas, each white outlined in black, centred in its cell within
/// its margin; run by `cargo make generate-helper-icons` after the pictures are exported.
#[test]
#[ignore]
fn generate_helper_icon_atlas() {
  let (width, height) = HelperIconAtlas::get_size(RENDER_HELPER_ICONS.len());
  let drawn: u32 = HelperIconAtlas::CELL - HelperIconAtlas::MARGIN * 2;
  let mut atlas: RgbaImage = RgbaImage::new(width, height);

  for icon in RENDER_HELPER_ICONS.iter().map(|entry| entry.icon) {
    let path: PathBuf = get_sources().join(icon.get_picture_file());
    let exported: String = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let inner: &str = exported
      .split_once('>')
      .and_then(|(_, rest)| rest.rsplit_once("</svg>"))
      .map_or("", |(inner, _)| inner);
    // The picture filled white over a black outline twice as wide as Material's own stroke, so it reads on any scene.
    let svg: String = format!(
      concat!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" color="#fff">"##,
        r##"<g fill="#fff" stroke="#000" stroke-width="2" stroke-linejoin="round" paint-order="stroke">{}</g></svg>"##
      ),
      inner
    );
    let tree: usvg::Tree = usvg::Tree::from_str(&svg, &usvg::Options::default())
      .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut pixmap: tiny_skia::Pixmap =
      tiny_skia::Pixmap::new(HelperIconAtlas::CELL, HelperIconAtlas::CELL).expect("a cell has pixels");

    resvg::render(
      &tree,
      tiny_skia::Transform::from_row(
        drawn as f32 / 24.0,
        0.0,
        0.0,
        drawn as f32 / 24.0,
        HelperIconAtlas::MARGIN as f32,
        HelperIconAtlas::MARGIN as f32,
      ),
      &mut pixmap.as_mut(),
    );

    let (left, top) = (
      icon.get_index() % HelperIconAtlas::COLUMNS * HelperIconAtlas::CELL,
      icon.get_index() / HelperIconAtlas::COLUMNS * HelperIconAtlas::CELL,
    );

    for (texel, pixel) in pixmap.pixels().iter().enumerate() {
      let colour: tiny_skia::ColorU8 = pixel.demultiply();

      atlas.put_pixel(
        left + texel as u32 % HelperIconAtlas::CELL,
        top + texel as u32 / HelperIconAtlas::CELL,
        Rgba([colour.red(), colour.green(), colour.blue(), colour.alpha()]),
      );
    }
  }

  let out: PathBuf = PathBuf::from(RENDER_BUNDLE_SOURCE).join(HelperIconAtlas::PATH);

  std::fs::create_dir_all(out.parent().expect("the atlas stands in a folder")).expect("the folder is made");
  atlas.save(&out).expect("the atlas is written");
}

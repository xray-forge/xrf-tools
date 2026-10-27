use std::fmt::Write;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use image::{Rgba, RgbaImage};
use resvg::usvg::fontdb::Database;
use resvg::{tiny_skia, usvg};
use xrf_app_core::TauriResult;
use xrf_utils::StagedFile;

use crate::parts::LevelMapArea;
use crate::viewer::map::annotations::{
  AnnotationOutline, AnnotationPoint, AnnotationVolume, LevelAnnotationKind, LevelAnnotations,
  MapAnnotationsWriteRequest,
};

/// How a map's annotations are drawn: the area the map covers onto its pixels, what of them is drawn, and whether
/// every volume and patrol is named rather than the smart terrains and level changers alone (a level has hundreds of
/// patrols, whose names would bury the map).
pub struct AnnotationCanvas {
  pub area: LevelMapArea,
  pub width: u32,
  pub height: u32,
  pub kinds: Vec<LevelAnnotationKind>,
  pub is_labelled: bool,
}

impl AnnotationCanvas {
  /// The map's longer side the sizes below are given at. A larger map is read zoomed in, so its marks grow by only the
  /// root of how much larger it is: twice as large at four times the side.
  const BASE_SIDE: f32 = 4096.0;

  /// Pixels a label stands at the base side, and the fewest it does on a smaller map.
  const LABEL_AT_BASE: f32 = 9.0;
  const LABEL_LEAST: f32 = 7.0;

  /// Pixels a line takes at the base side, and the fewest it does on a smaller map.
  const STROKE_AT_BASE: f32 = 0.6;
  const STROKE_LEAST: f32 = 0.35;

  /// A graph point's id against a label.
  const VERTEX_LABEL_SHARE: f32 = 0.75;

  /// The canvas a write request asks for, over its map's pixels.
  ///
  /// # Errors
  ///
  /// Returns why the map's size cannot be read.
  pub fn over_map(request: &MapAnnotationsWriteRequest) -> TauriResult<Self> {
    let (width, height): (u32, u32) = image::image_dimensions(&request.path)
      .map_err(|error| format!("Failed to read map '{}': {error}", request.path.display()))?;

    Ok(Self {
      area: request.area,
      width,
      height,
      kinds: request.kinds.clone(),
      is_labelled: request.is_labelled,
    })
  }

  /// Draws `annotations` and writes them to `target` whole; answers the SVG written.
  ///
  /// # Errors
  ///
  /// Returns why the SVG could not be written.
  pub fn write(&self, annotations: &LevelAnnotations, target: &Path) -> TauriResult<String> {
    let svg: String = self.draw(annotations);

    StagedFile::write(target, svg.as_bytes())
      .map_err(|error| format!("Failed to write annotations '{}': {error}", target.display()))?;

    Ok(svg)
  }

  /// The annotations as an SVG over the map: its view box the map's pixels, a group a kind, the lowest first.
  pub fn draw(&self, annotations: &LevelAnnotations) -> String {
    let growth: f32 = (self.width.max(self.height) as f32 / Self::BASE_SIDE).sqrt();
    let label: f32 = (Self::LABEL_AT_BASE * growth).max(Self::LABEL_LEAST);
    let stroke: f32 = (Self::STROKE_AT_BASE * growth).max(Self::STROKE_LEAST);
    let mut svg: String = String::new();

    let _ = write!(
      svg,
      r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" font-family="Segoe UI, Arial, sans-serif" font-weight="300" font-size="{label:.1}" text-anchor="middle" stroke-linejoin="round">"#,
      w = self.width,
      h = self.height,
    );

    for kind in LevelAnnotationKind::ALL
      .into_iter()
      .filter(|kind| self.kinds.contains(kind))
    {
      let colour: &str = kind.get_colour();

      let _ = write!(
        svg,
        r#"<g id="{}" fill="{colour}" stroke="{colour}" stroke-width="{stroke:.2}">"#,
        kind.get_id()
      );

      match kind {
        LevelAnnotationKind::GraphPoints => {
          let radius: f32 = (stroke * 2.0).max(0.75);
          let size: f32 = label * Self::VERTEX_LABEL_SHARE;

          // Each dot with its game vertex id beside it.
          for (id, point) in &annotations.graph_points {
            let (x, y): (f32, f32) = self.to_pixel(*point);
            let _ = write!(
              svg,
              r#"<circle cx="{x:.1}" cy="{y:.1}" r="{radius:.1}" stroke="none"/>"#
            );
            let _ = write!(
              svg,
              r##"<text x="{:.1}" y="{:.1}" font-size="{size:.1}" text-anchor="start" stroke="#101010" stroke-width="{:.2}" paint-order="stroke">{id}</text>"##,
              x + radius * 1.5,
              y + size * 0.35,
              size / 10.0,
            );
          }
        }
        LevelAnnotationKind::Patrols => self.draw_patrols(&mut svg, annotations, (stroke, label)),
        _ => {
          for volume in annotations.volumes.iter().filter(|volume| volume.kind == kind) {
            self.draw_volume(&mut svg, volume, (stroke, label));
          }
        }
      }

      svg.push_str("</g>");
    }

    svg.push_str("</svg>");

    svg
  }

  /// Where an engine-space point lands on the map, in its pixels from the top left: `+x` right and `+z` up, as
  /// `CDemoRecord` lays its map.
  pub fn to_pixel(&self, [x, z]: AnnotationPoint) -> (f32, f32) {
    let LevelMapArea {
      min_x,
      min_z,
      max_x,
      max_z,
    } = self.area;

    (
      (x - min_x) / (max_x - min_x) * self.width as f32,
      (max_z - z) / (max_z - min_z) * self.height as f32,
    )
  }

  /// Pixels a metre spans across.
  fn get_scale(&self) -> f32 {
    self.width as f32 / (self.area.max_x - self.area.min_x)
  }

  /// A volume's outlines, filled faintly, and its name where it is named: a level changer's with an arrow toward its
  /// destination.
  fn draw_volume(&self, svg: &mut String, volume: &AnnotationVolume, (stroke, label): (f32, f32)) {
    // A restrictor keeping the actor in is cyan, one keeping him out the kind's blue.
    let colour: Option<&str> = (volume.kind == LevelAnnotationKind::Restrictors && volume.is_in).then_some("#00bcd4");
    let paint: String = colour.map_or_else(String::new, |colour| format!(r#" fill="{colour}" stroke="{colour}""#));

    for outline in &volume.outlines {
      match outline {
        AnnotationOutline::Circle { center, radius } => {
          let (x, y): (f32, f32) = self.to_pixel(*center);
          let r: f32 = (radius * self.get_scale()).max(stroke);
          let _ = write!(
            svg,
            r#"<circle cx="{x:.1}" cy="{y:.1}" r="{r:.1}" fill-opacity="0.15"{paint}/>"#
          );
        }
        AnnotationOutline::Polygon(points) => {
          let points: Vec<String> = points
            .iter()
            .map(|point| {
              let (x, y): (f32, f32) = self.to_pixel(*point);

              format!("{x:.1},{y:.1}")
            })
            .collect();
          let _ = write!(
            svg,
            r#"<polygon points="{}" fill-opacity="0.15"{paint}/>"#,
            points.join(" ")
          );
        }
      }
    }

    let Some(center) = volume.outlines.first().map(AnnotationOutline::get_center) else {
      return;
    };
    let (x, y): (f32, f32) = self.to_pixel(center);

    match &volume.destination {
      Some((level, toward)) => {
        if let Some([dx, dz]) = toward {
          // The map's `y` runs down, against `z`.
          let (tx, ty): (f32, f32) = (x + dx * label * 3.0, y - dz * label * 3.0);
          let (side_x, side_y): (f32, f32) = (-dz * label * 0.6, -dx * label * 0.6);
          let (back_x, back_y): (f32, f32) = (tx - dx * label, ty + dz * label);

          let _ = write!(svg, r#"<line x1="{x:.1}" y1="{y:.1}" x2="{tx:.1}" y2="{ty:.1}"/>"#);
          let _ = write!(
            svg,
            r#"<polygon points="{tx:.1},{ty:.1} {:.1},{:.1} {:.1},{:.1}" stroke="none"/>"#,
            back_x + side_x,
            back_y + side_y,
            back_x - side_x,
            back_y - side_y,
          );
        }

        Self::draw_label(svg, (x, y - label), &format!("→ {level}"), label);
      }
      None if volume.kind == LevelAnnotationKind::SmartTerrains || self.is_labelled => {
        Self::draw_label(svg, (x, y), &volume.name, label);
      }
      None => {}
    }
  }

  /// The patrols' links as lines and points as dots, each named at its first point where every object is named.
  fn draw_patrols(&self, svg: &mut String, annotations: &LevelAnnotations, (stroke, label): (f32, f32)) {
    let radius: f32 = (stroke * 2.5).max(1.0);

    for patrol in &annotations.patrols {
      for (from, to) in &patrol.links {
        let (x1, y1): (f32, f32) = self.to_pixel(patrol.points[*from]);
        let (x2, y2): (f32, f32) = self.to_pixel(patrol.points[*to]);
        let _ = write!(svg, r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}"/>"#);
      }

      for point in &patrol.points {
        let (x, y): (f32, f32) = self.to_pixel(*point);
        let _ = write!(
          svg,
          r#"<circle cx="{x:.1}" cy="{y:.1}" r="{radius:.1}" stroke="none"/>"#
        );
      }

      if let Some(first) = patrol.points.first().filter(|_| self.is_labelled) {
        let (x, y): (f32, f32) = self.to_pixel(*first);

        Self::draw_label(svg, (x, y - label * 0.6), &patrol.name, label);
      }
    }
  }

  /// `text` at `at` in its group's colour, a thin dark outline under it so it reads over any ground.
  fn draw_label(svg: &mut String, (x, y): (f32, f32), text: &str, size: f32) {
    let _ = write!(
      svg,
      r##"<text x="{x:.1}" y="{y:.1}" stroke="#101010" stroke-width="{:.2}" paint-order="stroke" fill-opacity="1">{}</text>"##,
      size / 10.0,
      Self::escape(text)
    );
  }

  /// `svg` rasterised at `width` by `height` pixels, its view box stretched over them, its text set in the system's
  /// fonts; transparent where it draws nothing, its alpha straight.
  ///
  /// # Errors
  ///
  /// Returns an error for an SVG that cannot be read, or a size of no pixels.
  pub fn rasterise(svg: &str, (width, height): (u32, u32)) -> TauriResult<RgbaImage> {
    let options: usvg::Options<'_> = usvg::Options {
      fontdb: Self::get_system_fonts(),
      ..usvg::Options::default()
    };
    let tree: usvg::Tree =
      usvg::Tree::from_str(svg, &options).map_err(|error| format!("Failed to read the annotations: {error}"))?;
    let mut pixmap: tiny_skia::Pixmap = tiny_skia::Pixmap::new(width, height)
      .ok_or_else(|| format!("Cannot draw the annotations at {width} by {height} pixels"))?;
    let size: tiny_skia::Size = tree.size();

    resvg::render(
      &tree,
      tiny_skia::Transform::from_scale(width as f32 / size.width(), height as f32 / size.height()),
      &mut pixmap.as_mut(),
    );

    let mut image: RgbaImage = RgbaImage::new(width, height);

    for (pixel, drawn) in image.pixels_mut().zip(pixmap.pixels()) {
      let colour: tiny_skia::ColorU8 = drawn.demultiply();

      *pixel = Rgba([colour.red(), colour.green(), colour.blue(), colour.alpha()]);
    }

    Ok(image)
  }

  /// The system's fonts, read once a run: reading them takes a few hundred milliseconds.
  fn get_system_fonts() -> Arc<Database> {
    static SYSTEM_FONTS: OnceLock<Arc<Database>> = OnceLock::new();

    Arc::clone(SYSTEM_FONTS.get_or_init(|| {
      let mut fonts: Database = Database::new();

      fonts.load_system_fonts();

      Arc::new(fonts)
    }))
  }

  /// `text` as SVG character data.
  fn escape(text: &str) -> String {
    text
      .replace('&', "&amp;")
      .replace('<', "&lt;")
      .replace('>', "&gt;")
      .replace('"', "&quot;")
  }
}

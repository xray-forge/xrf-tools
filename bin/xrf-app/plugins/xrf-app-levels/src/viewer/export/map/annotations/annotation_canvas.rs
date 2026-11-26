use std::fmt::Write;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use image::{Rgba, RgbaImage};
use resvg::usvg::fontdb::Database;
use resvg::{tiny_skia, usvg};
use xrf_app_core::TauriResult;
use xrf_level_source::{LevelMapArea, LevelObjectKind, LevelObjectKindEntry};
use xrf_utils::StagedFile;

use crate::viewer::export::map::annotations::{
  AnnotationMarker, AnnotationOutline, AnnotationPatrol, AnnotationPoint, AnnotationVolume, LevelAnnotations,
  MapAnnotationsWriteRequest,
};

/// Draws a map's annotations: the covered area mapped onto the map's pixels, the object kinds drawn, and whether every
/// object is named rather than only smart terrains and level changers (a level's hundreds of patrol names would bury
/// the map).
pub(crate) struct AnnotationCanvas {
  pub area: LevelMapArea,
  pub width: u32,
  pub height: u32,
  pub kinds: Vec<LevelObjectKind>,
  pub is_labelled: bool,
}

impl AnnotationCanvas {
  /// The longer map side the sizes below apply at. A larger map is viewed zoomed in, so marks grow with the square
  /// root of its size: twice as large at four times the side.
  const BASE_SIDE: f32 = 4096.0;

  /// Label height in pixels at the base side, and its minimum on a smaller map.
  const LABEL_AT_BASE: f32 = 9.0;
  const LABEL_LEAST: f32 = 7.0;

  /// Line width in pixels at the base side, and its minimum on a smaller map.
  const STROKE_AT_BASE: f32 = 0.6;
  const STROKE_LEAST: f32 = 0.35;

  /// A dot's tag size relative to a label.
  const TAG_SHARE: f32 = 0.75;

  /// Colour of a restrictor keeping the actor in, distinct from the kind's blue for keeping him out.
  const RESTRICTOR_IN_COLOUR: &str = "#90caf9";

  /// The canvas `request` asks for, sized to its map's pixels.
  ///
  /// # Errors
  ///
  /// Returns why the map's size cannot be read.
  pub(crate) fn over_map(request: &MapAnnotationsWriteRequest) -> TauriResult<Self> {
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

  /// Draw `annotations` and write the SVG to `target` whole; answers the SVG.
  ///
  /// # Errors
  ///
  /// Returns why the SVG could not be written.
  pub(crate) fn write(&self, annotations: &LevelAnnotations, target: &Path) -> TauriResult<String> {
    let svg: String = self.draw(annotations);

    StagedFile::write(target, svg.as_bytes())
      .map_err(|error| format!("Failed to write annotations '{}': {error}", target.display()))?;

    Ok(svg)
  }

  /// A kind's layer, lowest first: dots, outlines, patrol lines, then the always-named volumes on top, so the names
  /// that matter stand over everything.
  fn get_layer(kind: LevelObjectKind) -> u8 {
    match kind {
      LevelObjectKind::SmartTerrains | LevelObjectKind::LevelChangers => 3,
      LevelObjectKind::Patrols => 2,
      LevelObjectKind::Restrictors
      | LevelObjectKind::Anomalies
      | LevelObjectKind::Climbables
      | LevelObjectKind::SoundEnvironments
      | LevelObjectKind::FogVolumes
      | LevelObjectKind::EnvModifiers => 1,
      LevelObjectKind::Props
      | LevelObjectKind::Items
      | LevelObjectKind::Weapons
      | LevelObjectKind::Lamps
      | LevelObjectKind::Actor
      | LevelObjectKind::Stalkers
      | LevelObjectKind::Creatures
      | LevelObjectKind::Traders
      | LevelObjectKind::Squads
      | LevelObjectKind::SmartCovers
      | LevelObjectKind::GraphPoints
      | LevelObjectKind::StaticParticles
      | LevelObjectKind::StaticSounds
      | LevelObjectKind::RespawnPoints
      | LevelObjectKind::Lights => 0,
    }
  }

  /// A kind's SVG group id: its label in kebab case.
  fn format_group_id(entry: &LevelObjectKindEntry) -> String {
    entry.label.to_lowercase().replace(' ', "-")
  }

  /// The annotations as an SVG whose view box is the map's pixels, one group per kind in its colour, lowest layer
  /// first.
  pub(crate) fn draw(&self, annotations: &LevelAnnotations) -> String {
    let growth: f32 = (self.width.max(self.height) as f32 / Self::BASE_SIDE).sqrt();
    let pen: AnnotationPen = AnnotationPen {
      stroke: (Self::STROKE_AT_BASE * growth).max(Self::STROKE_LEAST),
      label: (Self::LABEL_AT_BASE * growth).max(Self::LABEL_LEAST),
    };
    let AnnotationPen { stroke, label } = pen;
    let mut svg: String = String::new();

    let _ = write!(
      svg,
      r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" font-family="Segoe UI, Arial, sans-serif" font-weight="300" font-size="{label:.1}" text-anchor="middle" stroke-linejoin="round">"#,
      w = self.width,
      h = self.height,
    );

    let mut kinds: Vec<LevelObjectKind> = self.kinds.clone();

    kinds.sort_by_key(|kind| (Self::get_layer(*kind), *kind as usize));
    kinds.dedup();

    for kind in kinds {
      let entry: &LevelObjectKindEntry = kind.get_entry();
      let colour: &str = entry.color;

      let _ = write!(
        svg,
        r#"<g id="{}" fill="{colour}" stroke="{colour}" stroke-width="{stroke:.2}">"#,
        Self::format_group_id(entry)
      );

      for volume in annotations.volumes.iter().filter(|volume| volume.kind == kind) {
        self.draw_volume(&mut svg, volume, pen);
      }

      if kind == LevelObjectKind::Patrols {
        self.draw_patrols(&mut svg, &annotations.patrols, pen);
      }

      self.draw_markers(
        &mut svg,
        annotations.markers.iter().filter(|marker| marker.kind == kind),
        pen,
      );
      svg.push_str("</g>");
    }

    svg.push_str("</svg>");

    svg
  }

  /// An engine-space point in map pixels from the top left, `+x` right and `+z` up, as `CDemoRecord` lays its map.
  pub(crate) fn to_pixel(&self, [x, z]: AnnotationPoint) -> (f32, f32) {
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

  /// Pixels per metre.
  fn get_scale(&self) -> f32 {
    self.width as f32 / (self.area.max_x - self.area.min_x)
  }

  /// A volume's faintly filled outlines and, when named, its name; a level changer gets an arrow toward its
  /// destination.
  fn draw_volume(&self, svg: &mut String, volume: &AnnotationVolume, pen: AnnotationPen) {
    let AnnotationPen { stroke, label } = pen;
    let colour: Option<&str> =
      (volume.kind == LevelObjectKind::Restrictors && volume.is_in).then_some(Self::RESTRICTOR_IN_COLOUR);
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

        Self::draw_label(svg, x, y - label, &format!("→ {level}"), label);
      }
      None if volume.kind == LevelObjectKind::SmartTerrains || self.is_labelled => {
        Self::draw_label(svg, x, y, &volume.name, label);
      }
      None => {}
    }
  }

  /// Patrol links as lines and points as dots, each patrol named at its first point when every object is named.
  fn draw_patrols(&self, svg: &mut String, patrols: &[AnnotationPatrol], pen: AnnotationPen) {
    let AnnotationPen { stroke, label } = pen;
    let radius: f32 = (stroke * 2.5).max(1.0);

    for patrol in patrols {
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

        Self::draw_label(svg, x, y - label * 0.6, &patrol.name, label);
      }
    }
  }

  /// `markers` as dots, each beside its tag, or its name when every object is named.
  fn draw_markers<'a>(
    &self,
    svg: &mut String,
    markers: impl Iterator<Item = &'a AnnotationMarker>,
    pen: AnnotationPen,
  ) {
    let radius: f32 = (pen.stroke * 2.5).max(1.0);
    let size: f32 = pen.label * Self::TAG_SHARE;

    for marker in markers {
      let (x, y): (f32, f32) = self.to_pixel(marker.point);
      let _ = write!(
        svg,
        r#"<circle cx="{x:.1}" cy="{y:.1}" r="{radius:.1}" stroke="none"/>"#
      );

      if let Some(text) = marker
        .tag
        .as_deref()
        .or_else(|| self.is_labelled.then_some(marker.name.as_str()))
      {
        let _ = write!(
          svg,
          r##"<text x="{:.1}" y="{:.1}" font-size="{size:.1}" text-anchor="start" stroke="#101010" stroke-width="{:.2}" paint-order="stroke">{}</text>"##,
          x + radius * 1.5,
          y + size * 0.35,
          size / 10.0,
          Self::escape(text)
        );
      }
    }
  }

  /// `text` at (`x`, `y`) in its group's colour, over a thin dark outline so it reads on any ground.
  fn draw_label(svg: &mut String, x: f32, y: f32, text: &str, size: f32) {
    let _ = write!(
      svg,
      r##"<text x="{x:.1}" y="{y:.1}" stroke="#101010" stroke-width="{:.2}" paint-order="stroke" fill-opacity="1">{}</text>"##,
      size / 10.0,
      Self::escape(text)
    );
  }

  /// `svg` rasterised to `width` by `height` with its view box stretched over them and text in system fonts; straight
  /// alpha, transparent where nothing is drawn.
  ///
  /// # Errors
  ///
  /// Returns an error for an SVG that cannot be read, or a size of no pixels.
  pub(crate) fn rasterise(svg: &str, (width, height): (u32, u32)) -> TauriResult<RgbaImage> {
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

  /// The system fonts, read once per run since reading takes a few hundred milliseconds.
  fn get_system_fonts() -> Arc<Database> {
    static SYSTEM_FONTS: OnceLock<Arc<Database>> = OnceLock::new();

    Arc::clone(SYSTEM_FONTS.get_or_init(|| {
      let mut fonts: Database = Database::new();

      fonts.load_system_fonts();

      Arc::new(fonts)
    }))
  }

  fn escape(text: &str) -> String {
    text
      .replace('&', "&amp;")
      .replace('<', "&lt;")
      .replace('>', "&gt;")
      .replace('"', "&quot;")
  }
}

/// Annotation sizes in map pixels: line width and label height.
#[derive(Clone, Copy)]
struct AnnotationPen {
  stroke: f32,
  label: f32,
}

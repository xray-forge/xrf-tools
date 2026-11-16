use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_level::DetailModel;
use xrf_math::Vector2d;
use xrf_utils::format_path;

use crate::level::build::level_build_atlas::LevelBuildAtlasPlace;
use crate::level::level_detail_match::LevelDetailMatch;
use crate::object::compile::detail::detail_model_builder::DetailModelBuilder;
use crate::object::detail_parameters::DetailParameters;

/// The detail objects of the object library roots, each compiled as `EDetail` exports it: what a level's
/// detail models are matched against, since `level.details` keeps their geometry but not the objects they came from.
/// The Level Editor keeps its detail objects under `detail\`, so only that folder of each root is read.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LevelDetailLibrary {
  /// Each object by its reference, in the order of their paths.
  sources: Vec<(String, DetailModel)>,
  /// Objects under the roots' detail folders that did not read or compile, and so match nothing.
  pub(crate) unread_objects: usize,
}

impl LevelDetailLibrary {
  /// The folder of an object library root the Level Editor's detail objects stand in.
  pub(crate) const DIRECTORY: &'static str = "detail";

  const OBJECT_EXTENSION: &'static str = "object";

  /// How far a remapped texture coordinate may stand from the stored one, past single-precision rounding.
  const REMAP_TOLERANCE: f32 = 1e-5;

  /// How little a source's coordinates may span on an axis and still give that axis a scale.
  const SPAN_EPSILON: f32 = 1e-6;

  /// A library of `sources`, each a detail object's reference and its model as `EDetail` exports it, in the order
  /// [`Self::find`] tries them.
  #[cfg(test)]
  pub(crate) fn new(sources: Vec<(String, DetailModel)>) -> Self {
    Self {
      sources,
      unread_objects: 0,
    }
  }

  /// The detail objects under each of `roots`' [`Self::DIRECTORY`], a root without one holding none.
  ///
  /// # Errors
  ///
  /// Returns an error when a root is not a directory, or a folder under it cannot be listed.
  pub(crate) fn read_from_roots<T: ByteOrder>(roots: &[PathBuf]) -> XrfResult<Self> {
    let mut library: Self = Self::default();

    for root in roots {
      if !root.is_dir() {
        return Err(XrfError::new_not_found_error(format!(
          "Object library {} is not a directory",
          format_path(root)
        )));
      }

      let mut paths: Vec<PathBuf> = Vec::new();
      let directory: PathBuf = root.join(Self::DIRECTORY);

      if directory.is_dir() {
        Self::list_objects(&directory, &mut paths)?;
      }

      paths.sort();

      for path in paths {
        let compiled: XrfResult<DetailModel> = EditableObjectFile::read_from_path::<T, _>(&path)
          .and_then(|object| DetailModelBuilder::build_model(&object, &DetailParameters::default()))
          .map(|(model, _)| model);

        match compiled {
          Ok(model) => library.sources.push((Self::get_reference(root, &path), model)),
          Err(_) => library.unread_objects += 1,
        }
      }
    }

    Ok(library)
  }

  /// The first object drawing `stored`: the same positions and indices, its texture coordinates `stored`'s before
  /// `MergedTextureRemapUV` moved them into the level's atlas.
  pub(crate) fn find(&self, stored: &DetailModel) -> Option<LevelDetailMatch<'_>> {
    self.sources.iter().find_map(|(reference, source)| {
      let is_same_shape: bool = source.indices == stored.indices
        && source.vertices.len() == stored.vertices.len()
        && source
          .vertices
          .iter()
          .zip(&stored.vertices)
          .all(|(source, stored)| source.position == stored.position);

      if is_same_shape {
        Self::fit(reference, source, stored)
      } else {
        None
      }
    })
  }

  /// The match of `source` where `stored`'s coordinates are its own remapped into an atlas, `scale * source + offset`
  /// on each axis, `u` and `v` swapped where the texture was turned to fit, as [`LevelBuildAtlasPlace::remap`] moves
  /// them: the remap fitted from the extreme coordinates and checked on every vertex, unturned first.
  fn fit<'a>(reference: &'a str, source: &'a DetailModel, stored: &DetailModel) -> Option<LevelDetailMatch<'a>> {
    [false, true].into_iter().find_map(|is_rotated| {
      let pairs = |axis: usize| -> Vec<LevelDetailRemapPair> {
        source
          .vertices
          .iter()
          .zip(&stored.vertices)
          .map(|(source, stored)| {
            let from: [f32; 2] = if is_rotated {
              [source.v, source.u]
            } else {
              [source.u, source.v]
            };

            LevelDetailRemapPair {
              from: from[axis],
              to: [stored.u, stored.v][axis],
            }
          })
          .collect()
      };
      let u: LevelDetailAxisRemap = Self::fit_axis(&pairs(0))?;
      let v: LevelDetailAxisRemap = Self::fit_axis(&pairs(1))?;
      let place: Option<LevelBuildAtlasPlace> = match (u, v) {
        (
          LevelDetailAxisRemap::Line {
            scale: u_scale,
            offset: u_offset,
          },
          LevelDetailAxisRemap::Line {
            scale: v_scale,
            offset: v_offset,
          },
        ) => Some(LevelBuildAtlasPlace {
          offset: Vector2d::new(u_offset, v_offset),
          scale: Vector2d::new(u_scale, v_scale),
          is_rotated,
        }),
        _ => None,
      };

      Some(LevelDetailMatch {
        reference,
        source,
        place,
      })
    })
  }

  /// The one remap carrying every pair, fitted from the pairs of the lowest and highest source coordinate; a source
  /// that spans nothing is carried by a shift alone.
  fn fit_axis(pairs: &[LevelDetailRemapPair]) -> Option<LevelDetailAxisRemap> {
    let Some(first) = pairs.first() else {
      return Some(LevelDetailAxisRemap::Shift { offset: 0.0 });
    };
    let (low, high) = pairs.iter().fold((first, first), |(low, high), pair| {
      (
        if pair.from < low.from { pair } else { low },
        if pair.from > high.from { pair } else { high },
      )
    });
    let span: f32 = high.from - low.from;
    let remap: LevelDetailAxisRemap = if span.abs() < Self::SPAN_EPSILON {
      LevelDetailAxisRemap::Shift {
        offset: low.to - low.from,
      }
    } else {
      let scale: f32 = (high.to - low.to) / span;

      LevelDetailAxisRemap::Line {
        scale,
        offset: low.to - scale * low.from,
      }
    };

    pairs
      .iter()
      .all(|pair| (remap.apply(pair.from) - pair.to).abs() <= Self::REMAP_TOLERANCE)
      .then_some(remap)
  }

  /// `path`'s reference under `root`: its path there without the extension, folders joined by `\`.
  fn get_reference(root: &Path, path: &Path) -> String {
    path
      .strip_prefix(root)
      .unwrap_or(path)
      .with_extension("")
      .components()
      .map(|component| component.as_os_str().to_string_lossy())
      .collect::<Vec<_>>()
      .join("\\")
  }

  fn list_objects(directory: &Path, paths: &mut Vec<PathBuf>) -> XrfResult {
    let entries = std::fs::read_dir(directory)
      .map_err(|error| XrfError::new_read_error(format!("{} was not listed: {error}", format_path(directory))))?;

    for entry in entries {
      let path: PathBuf = entry
        .map_err(|error| XrfError::new_read_error(format!("{} was not listed: {error}", format_path(directory))))?
        .path();

      if path.is_dir() {
        Self::list_objects(&path, paths)?;
      } else if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(Self::OBJECT_EXTENSION))
      {
        paths.push(path);
      }
    }

    Ok(())
  }
}

/// One vertex's coordinate on one axis: `from` the source's, `to` the level's.
struct LevelDetailRemapPair {
  from: f32,
  to: f32,
}

/// How one axis of a source's coordinates moves into the atlas.
#[derive(Clone, Copy)]
enum LevelDetailAxisRemap {
  /// `scale * from + offset`, fitted where the source spans the axis.
  Line { scale: f32, offset: f32 },
  /// `from + offset`, where the source spans nothing on the axis, so no scale can be told.
  Shift { offset: f32 },
}

impl LevelDetailAxisRemap {
  fn apply(self, from: f32) -> f32 {
    match self {
      Self::Line { scale, offset } => scale * from + offset,
      Self::Shift { offset } => from + offset,
    }
  }
}

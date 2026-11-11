use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_level::DetailModel;
use xrf_utils::format_path;

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

  /// The reference of the first object drawing `stored`: the same positions and indices, its texture coordinates
  /// `stored`'s before `MergedTextureRemapUV` moved them into the level's atlas.
  pub(crate) fn find(&self, stored: &DetailModel) -> Option<&str> {
    self
      .sources
      .iter()
      .find(|(_, source)| {
        source.indices == stored.indices
          && source.vertices.len() == stored.vertices.len()
          && source
            .vertices
            .iter()
            .zip(&stored.vertices)
            .all(|(source, stored)| source.position == stored.position)
          && Self::is_remap(source, stored)
      })
      .map(|(reference, _)| reference.as_str())
  }

  /// Whether `stored`'s coordinates are `source`'s remapped into an atlas, `scale * source + offset` on each axis, `u`
  /// and `v` swapped where the texture was turned to fit: the remap fitted from the extreme coordinates and checked on
  /// every vertex.
  fn is_remap(source: &DetailModel, stored: &DetailModel) -> bool {
    [false, true].into_iter().any(|is_rotated| {
      let pairs = |axis: usize| -> Vec<(f32, f32)> {
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

            (from[axis], [stored.u, stored.v][axis])
          })
          .collect()
      };

      [0, 1].into_iter().all(|axis| Self::fits_axis(&pairs(axis)))
    })
  }

  /// Whether one straight line `to = scale * from + offset` carries every pair; a constant source carries any.
  fn fits_axis(pairs: &[(f32, f32)]) -> bool {
    let Some(first) = pairs.first() else {
      return true;
    };
    let (low, high) = pairs.iter().fold((*first, *first), |(low, high), pair| {
      (
        if pair.0 < low.0 { *pair } else { low },
        if pair.0 > high.0 { *pair } else { high },
      )
    });
    let span: f32 = high.0 - low.0;
    let (scale, offset) = if span.abs() < 1e-6 {
      (1.0, low.1 - low.0)
    } else {
      let scale: f32 = (high.1 - low.1) / span;

      (scale, low.1 - scale * low.0)
    };

    pairs
      .iter()
      .all(|(from, to)| (scale * from + offset - to).abs() <= Self::REMAP_TOLERANCE)
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
